mod models;
mod parser;

use models::get_models;
use parser::{
    cache::{Cache, PATH},
    config::Config,
    md2pango::md2pango,
};

use gdk::{gio, keys::constants as keys, ModifierType};
use gtk::{
    glib::{self, Propagation, *},
    prelude::*,
    Application, ApplicationWindow, Button, ComboBoxText, Entry, Label, ListStore, ScrolledWindow,
};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use serde_json::json;
use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, OnceLock},
    usize,
};
use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}

#[derive(Clone)]
struct Tabs {
    tab: gtk::Box,
    id: usize,
    file: PathBuf,
    model: Option<String>,
}

impl Tabs {
    fn get_tab_from_id(id: usize, tabs: &Vec<Tabs>) -> Option<gtk::Box> {
        for tab in tabs {
            if tab.id == id {
                return Some(tab.tab.clone());
            }
        }
        None
    }
}

struct UI {
    tabs: Vec<Tabs>,
    tab_count: usize,
    model_count: u32,
}

impl UI {
    fn build_ui(app: &Application, config: &Arc<Config>) {
        let ui = Rc::new(RefCell::new(UI {
            tabs: Vec::new(),
            tab_count: 0,
            model_count: 0,
        }));
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(480)
            .default_height(1000)
            .title("converse")
            .build();

        if config.general.use_gtk_layer {
            window.init_layer_shell();
            window.set_namespace("converse");
            window.set_keyboard_interactivity(true);
            window.auto_exclusive_zone_enable();

            window.set_layer(Layer::Overlay);
        }

        window.style_context().add_class("main-window");

        let anchors = [
            (Edge::Left, true, config.general.layer_margin_left),
            (Edge::Right, true, config.general.layer_margin_right),
            (Edge::Top, true, config.general.layer_margin_top),
            (Edge::Bottom, true, config.general.layer_margin_bottom),
        ];

        for (anchor, state, margin_size) in anchors {
            window.set_anchor(anchor, state);
            window.set_layer_shell_margin(anchor, margin_size);
        }

        // Main UI layout

        let notebook = gtk::Notebook::builder()
            .scrollable(true)
            .tab_pos(gtk::PositionType::Top)
            .build();
        notebook.style_context().add_class("tab-page");

        let entry = Entry::builder().placeholder_text("Enter Text").build();
        entry.style_context().add_class("entry");

        let sent_icon =
            gtk::Image::from_icon_name(Some("document-send-symbolic"), gtk::IconSize::Dnd);
        let send_button = Button::builder().image(&sent_icon).build();
        send_button.style_context().add_class("send-button");

        let entry_box_horizontal = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        entry_box_horizontal.pack_start(&entry, true, true, 0);
        entry_box_horizontal.pack_start(&send_button, false, false, 0);

        let reset_icon = gtk::Image::from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Dnd);
        let add_tab_button = Button::builder().image(&reset_icon).build();
        add_tab_button.style_context().add_class("truncate-chat");

        let model_combobox = ComboBoxText::new();
        model_combobox.style_context().add_class("model-combobox");
        let model_list = ListStore::new(&[String::static_type()]);
        for model in get_models(&config) {
            model_list.set(&model_list.append(), &[(0, &model)]);
            ui.borrow_mut().model_count += 1;
        }
        model_combobox.set_model(Some(&model_list));
        model_combobox.set_active(Some(0));

        let control_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
        control_area.style_context().add_class("control-area");
        let control_area_horizontal = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        control_area.pack_start(&entry_box_horizontal, true, true, 0);
        control_area.pack_start(&control_area_horizontal, false, false, 0);

        control_area_horizontal.pack_start(&model_combobox, true, true, 0);
        control_area_horizontal.pack_start(&add_tab_button, false, false, 0);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.pack_start(&notebook, true, true, 0);
        main_box.pack_start(&control_area, false, false, 0);

        window.add(&main_box);

        let provider = gtk::CssProvider::new();
        if let Err(_) =
            provider.load_from_path(&format!("{}/.config/converse/style.css", env!("HOME")))
        {
            eprintln!("No theme file found. Using defaults.");
            provider
                .load_from_data(include_bytes!("../res/style.css"))
                .unwrap();
        }
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Failed to get GDK screen for CSS provider!"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        entry.grab_focus();

        let (sender, receiver) = async_channel::bounded(1);

        // Event Handlers.

        // Key bindings
        window.connect_key_press_event(
            clone!(@weak send_button, @weak notebook, @weak entry, @weak model_combobox, @weak ui => @default-return Propagation::Proceed, move |window, event| {
            let modifier = if event.state().is_empty() {
                None
            } else {
                Some(event.state())
            };
            let state = match (event.keyval(), modifier) {
                (keys::Escape, None) => {
                    window.close();
                    Propagation::Stop
                }

                (keys::Return, None) => {
                    send_button.emit_clicked();
                    Propagation::Stop
                }

                (keys::j, Some(ModifierType::CONTROL_MASK)) | (keys::n, Some(ModifierType::CONTROL_MASK)) => {
                    if let Some(page_number) = notebook.current_page() {
                        let current_page = &ui.borrow().tabs[page_number as usize ].tab;
                        let scroll: ScrolledWindow = current_page.parent().unwrap().parent().unwrap().downcast().unwrap();
                        let adj = scroll.vadjustment();
                        adj.set_value(adj.value()+40.0);
                    }
                    Propagation::Stop
                }

                (keys::k, Some(ModifierType::CONTROL_MASK)) | (keys::p, Some(ModifierType::CONTROL_MASK)) => {
                    if let Some(page_number) = notebook.current_page() {
                        let current_page = &ui.borrow().tabs[page_number as usize ].tab;
                        let scroll: ScrolledWindow = current_page.parent().unwrap().parent().unwrap().downcast().unwrap();
                        let adj = scroll.vadjustment();
                        adj.set_value(adj.value()-40.0);
                    }
                    Propagation::Stop
                }

                (keys::_1, Some(ModifierType::MOD1_MASK))
                | (keys::_2, Some(ModifierType::MOD1_MASK))
                | (keys::_3, Some(ModifierType::MOD1_MASK))
                | (keys::_4, Some(ModifierType::MOD1_MASK))
                | (keys::_5, Some(ModifierType::MOD1_MASK))
                | (keys::_6, Some(ModifierType::MOD1_MASK))
                | (keys::_7, Some(ModifierType::MOD1_MASK))
                | (keys::_8, Some(ModifierType::MOD1_MASK))
                | (keys::_9, Some(ModifierType::MOD1_MASK)) => {
                    notebook.set_current_page(Some(
                        event.keyval().name().unwrap().parse::<u32>().unwrap() - 1,
                    ));
                    Propagation::Stop
                }

                (keys::l, Some(ModifierType::CONTROL_MASK)) => {
                    entry.grab_focus();
                    Propagation::Stop
                }

                (keys::Tab, None) => {
                    if model_combobox.is_sensitive() {
                        let next = model_combobox.active().map(|x| {if x == ui.borrow().model_count -1 {0} else {x+1}});
                        model_combobox.set_active(next);
                    }
                    Propagation::Stop
                }

                (keys::ISO_Left_Tab, Some(ModifierType::SHIFT_MASK)) => {
                    if model_combobox.is_sensitive() {
                        let previous = model_combobox.active().map(|x| {if x == 0 {ui.borrow().model_count -1} else {x-1}});
                        model_combobox.set_active(previous);
                    }
                    Propagation::Stop
                }

                (keys::t, Some(ModifierType::CONTROL_MASK)) => {
                    Self::new_page(&ui, &notebook, None);
                    Propagation::Stop
                }

                (keys::w, Some(ModifierType::CONTROL_MASK)) => {
                    if let Some(page_num) = notebook.current_page() {
                        notebook.remove_page(Some(page_num));
                        fs::remove_file(ui.borrow().tabs[page_num as usize].file.clone())
                            .ok();
                        ui.borrow_mut().tabs.remove(page_num as usize);
                    };
                    Propagation::Stop
                }

                _ => Propagation::Proceed,
            };
            window.show_all();
            state
        }));

        // Adds another tab.
        add_tab_button.connect_clicked(
            clone!( @weak notebook, @weak model_combobox, @weak ui => move |_| {
                Self::new_page(&ui, &notebook, None);
                notebook.show_all();
            }),
        );

        // Sends responses.
        send_button.connect_clicked(
            clone!(@weak entry, @weak notebook, @weak window, @weak model_combobox, @weak ui, @strong config => move |button| {
                let config = config.clone();
                let entry_text = entry.text();
                let selected_model = model_combobox.active_text().unwrap().to_string();

                if !entry_text.is_empty() {

                    let answer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

                    let new_question_label = Self::new_label(entry_text.as_str(), true, false);

                    answer_box.pack_start(&new_question_label, false, false, 0);
                    answer_box.set_halign(gtk::Align::End);
                    answer_box.style_context().add_class("label-user");


                    let page_number = notebook.current_page().unwrap_or_else(|| {
                        Self::new_page(&ui, &notebook, None);
                        0
                    });
                    ui.borrow_mut().tabs[page_number as usize].model = Some(selected_model.clone());
                    let current_page = &ui.borrow().tabs[page_number as usize ];
                    current_page.tab.pack_start(&answer_box, false, false, 0);
                    let current_page_id = current_page.id.clone();
                    let file = current_page.clone().file;
                    entry.delete_text(0, -1);
                    entry.set_sensitive(false);
                    button.set_sensitive(false);
                    model_combobox.set_sensitive(false);
                    window.show_all();

                    runtime().spawn(clone!(@strong sender => async move {
                        let response = models::select_model(&selected_model, &entry_text, config, file).await;
                        sender.send((response, current_page_id)).await.expect("The channel needs to be open.");
                    }));
                }
            }),
        );

        // Handles tab switching.
        let inhibit_notebook = notebook.connect_switch_page(
            clone!(@weak ui, @weak config => move |notebook, _, page| {
                if notebook.children().len() != 0 {
                    if let Some(file) = &ui.borrow().tabs.get(page as usize) {
                        if let Some(model) = file.model.clone() {
                            let index = get_models(&config)
                                .iter()
                                .position(|r| r == &model)
                                .unwrap_or_default();
                            model_combobox.set_active(Some(index as u32));
                            model_combobox.set_sensitive(false);
                        } else {
                            model_combobox.set_active(Some(0));
                            model_combobox.set_sensitive(true);
                        }
                    }
                }
            }),
        );

        // Handles the api call to the llm and adds Label widget.
        glib::spawn_future_local(
            clone!(@weak notebook, @weak window, @weak entry, @weak ui, @weak config => async move {
                while let Ok((response, current_page_id)) = receiver.recv().await {
                    entry.set_sensitive(true);
                    send_button.set_sensitive(true);
                    let label_content = if let Ok(response) = response {
                        if response.status.is_success() {
                            response.answer
                        } else {
                            response.status.to_string()
                        }
                    }
                    else {
                        "Could not connect to a server.".to_string()
                    };

                    let answer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

                    for block in md2pango(&label_content, &config) {
                        Self::model_response_format(block, &answer_box);
                        answer_box.set_halign(gtk::Align::Start);
                        answer_box.style_context().add_class("label-model");
                    }
                    if let Some(tab) = Tabs::get_tab_from_id(current_page_id, &ui.borrow().tabs) {
                        tab.pack_start(&answer_box, false, false, 0);
                    }
                    window.show_all();
                }
            }),
        );

        let file_list = Cache::read_all();
        if file_list.len() != 0 {
            for file in file_list {
                Self::update(&ui, &notebook, &config, Some(file), &inhibit_notebook);
            }
        } else {
            Self::update(&ui, &notebook, &config, None, &inhibit_notebook);
        }

        window.show_all();
    }

    // Reads history and creates tab accordingly when first opened.
    pub fn update(
        ui: &Rc<RefCell<UI>>,
        notebook: &gtk::Notebook,
        config: &Config,
        dir_file: Option<PathBuf>,
        inhibit_notebook: &SignalHandlerId,
    ) -> Option<String> {
        notebook.block_signal(inhibit_notebook);
        let (chat_box_layout, chats) = Self::new_page(ui, notebook, dir_file.clone());
        if chats["chat"] != json!([]) {
            for chat in chats["chat"].as_array().unwrap() {
                let answer = chat["text"].as_str().unwrap();
                let answer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

                if chat["role"] == "user" {
                    let label_user = Self::new_label(answer, true, false);
                    answer_box.pack_start(&label_user, false, false, 0);
                    answer_box.set_halign(gtk::Align::End);
                    answer_box.style_context().add_class("label-user");
                } else {
                    for block in md2pango(answer, &config) {
                        Self::model_response_format(block, &answer_box);
                        answer_box.set_halign(gtk::Align::Start);
                        answer_box.style_context().add_class("label-model");
                    }
                };

                chat_box_layout.pack_start(&answer_box, false, false, 0);
            }
        }
        notebook.unblock_signal(&inhibit_notebook);
        None
    }

    // Used to create a new tab page.
    fn new_page(
        ui: &Rc<RefCell<UI>>,
        notebook: &gtk::Notebook,
        file: Option<PathBuf>,
    ) -> (gtk::Box, serde_json::Value) {
        let chat_box_layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(false)
            .build();

        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let tab_label =
            gtk::Label::new(Some(&format!("Session {}", notebook.children().len() + 1)));
        let close_image =
            gtk::Image::from_icon_name(Some("window-close-symbolic"), gtk::IconSize::Button);
        let close_button = gtk::Button::new();
        close_button.set_relief(gtk::ReliefStyle::None);
        close_button.set_image(Some(&close_image));
        tab.pack_start(&tab_label, true, true, 0);
        tab.pack_end(&close_button, true, true, 0);
        tab.show_all();

        scroll.add(&chat_box_layout);
        notebook.insert_page(&scroll, Some(&tab), None);

        close_button.connect_clicked(clone!(@weak notebook, @strong ui => move |_| {
            let index = notebook.page_num(&scroll).expect("Couldn't get page_num from notebook");
            fs::remove_file(ui.borrow().tabs[index as usize].file.clone()).ok();
            ui.borrow_mut().tabs.remove(index as usize);
            notebook.remove_page(Some(index));
        }));

        ui.borrow_mut().tab_count += 1;
        let tab_id = ui.borrow().tab_count;
        let file =
            file.unwrap_or_else(|| PathBuf::from(format!("{}/{}-history.json", PATH, real_time())));
        let chats = Cache::read(&file);
        let model = if let Some(model) = chats["model"].as_str() {
            Some(model.to_string())
        } else {
            None
        };
        ui.borrow_mut().tabs.push(Tabs {
            tab: chat_box_layout.clone(),
            id: tab_id,
            file,
            model,
        });
        (chat_box_layout, chats)
    }

    // Creates a label that contains user dialogs or model code/non-code responses.
    fn new_label(content: &str, is_user: bool, is_code: bool) -> Label {
        let answer_label = Label::new(None);
        answer_label.set_selectable(true);
        answer_label.set_wrap(true);
        if is_user {
            answer_label.set_justify(gtk::Justification::Right);
            answer_label.set_text(content);
        } else {
            answer_label.set_justify(gtk::Justification::Left);
            if is_code {
                answer_label.set_text(content);
                answer_label.set_halign(gtk::Align::Fill);
            } else {
                answer_label.set_markup(content);
                answer_label.set_halign(gtk::Align::Start);
            }
        }
        answer_label
    }

    // Formats model responses based on code and non code block segments.
    fn model_response_format(block: parser::md2pango::FormattedCode, answer_box: &gtk::Box) {
        let label_model = Self::new_label(&block.string, false, block.is_code);
        if block.is_code {
            let copy_image =
                gtk::Image::from_icon_name(Some("edit-copy-symbolic"), gtk::IconSize::Button);
            let copy_button = Button::builder()
                .image(&copy_image)
                .halign(gtk::Align::End)
                .tooltip_text("Copy")
                .build();
            let code_block = gtk::Box::new(gtk::Orientation::Vertical, 0);
            copy_button.style_context().add_class("code-copy");
            code_block.style_context().add_class("label-model-code");

            code_block.add(&copy_button);
            code_block.add(&label_model);
            answer_box.pack_start(&code_block, true, true, 0);

            copy_button.connect_clicked(move |_| {
                let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                clipboard.set_text(&block.string);
            });
        } else {
            answer_box.pack_start(&label_model, true, true, 0);
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(Config::new());
    let app = Application::builder()
        .application_id("com.github.vishruth-thimmaiah.converse")
        .build();

    // only allows single instance of program.
    app.register(None::<&gio::Cancellable>).unwrap();
    if app.is_remote() {
        return;
    }

    app.connect_activate(move |app| {
        UI::build_ui(app, &config);
    });
    app.run();
}
