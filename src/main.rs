mod models;
mod parser;

use parser::{cache::Cache, config::Config, md2pango::md2pango};

use gdk::gio;
use gtk::{
    glib::{self, Propagation, *},
    prelude::*,
    Application, ApplicationWindow, Box, Button, ComboBoxText, Entry, Label, ListStore,
    ScrolledWindow,
};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use serde_json::json;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}

struct UI {}

impl UI {
    fn build_ui(app: &Application, config: Config) {
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

            window.set_layer_shell_margin(Edge::Left, config.general.layer_margin_left);
            window.set_layer_shell_margin(Edge::Right, config.general.layer_margin_right);
            window.set_layer_shell_margin(Edge::Top, config.general.layer_margin_top);
            window.set_layer_shell_margin(Edge::Bottom, config.general.layer_margin_bottom);

            window.set_layer(Layer::Overlay);
        }

        window.style_context().add_class("main-window");

        let anchors = [
            (Edge::Left, true),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, true),
        ];

        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }

        // Main UI layout
        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(false)
            .build();

        let chat_box_layout = Box::new(gtk::Orientation::Vertical, 0);

        scroll.add(&chat_box_layout);

        let entry = Entry::builder().placeholder_text("Enter Text").build();
        entry.style_context().add_class("entry");

        let send_button = Button::builder().label("󱅥").build();
        send_button.style_context().add_class("send-button");

        let entry_box_horizontal = Box::new(gtk::Orientation::Horizontal, 0);
        entry_box_horizontal.pack_start(&entry, true, true, 0);
        entry_box_horizontal.pack_start(&send_button, false, false, 0);

        let truncate_button = Button::builder().label("󱅯").build();
        truncate_button.style_context().add_class("truncate-chat");

        let model_combobox = ComboBoxText::new();
        model_combobox.style_context().add_class("model-combobox");
        let model_list = ListStore::new(&[String::static_type()]);
        model_list.set(&model_list.append(), &[(0, &"Gemini")]);
        model_list.set(&model_list.append(), &[(0, &"Cohere")]);
        model_combobox.set_model(Some(&model_list));
        model_combobox.set_active(Some(0));

        let control_area = Box::new(gtk::Orientation::Vertical, 0);
        control_area.style_context().add_class("control-area");
        let control_area_horizontal = Box::new(gtk::Orientation::Horizontal, 0);

        control_area.pack_start(&entry_box_horizontal, true, true, 0);
        control_area.pack_start(&control_area_horizontal, false, false, 0);

        control_area_horizontal.pack_start(&model_combobox, true, true, 0);
        control_area_horizontal.pack_start(&truncate_button, false, false, 0);

        let main_box = Box::new(gtk::Orientation::Vertical, 0);
        main_box.pack_start(&scroll, true, true, 0);
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

        let contains_history = Self::update(&chat_box_layout);
        model_combobox.set_sensitive(!contains_history);

        let (sender, receiver) = async_channel::bounded(1);

        // Event Handlers.
        let send_button_clone = send_button.clone();
        window.connect_key_press_event(move |window, event| {
            let state = match event.keyval() {
                gdk::keys::constants::Escape => {
                    window.close();
                    Propagation::Stop
                }
                gdk::keys::constants::Return => {
                    send_button_clone.emit_clicked();
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            };
            let adj = scroll.vadjustment();
            adj.set_value(adj.upper());
            window.show_all();
            state
        });

        truncate_button.connect_clicked(
            clone!( @weak chat_box_layout, @weak model_combobox => move |_| {
                Cache::truncate();
                model_combobox.set_sensitive(true);
                for child in chat_box_layout.children() {
                    chat_box_layout.remove(&child);
                }
            }),
        );

        send_button.connect_clicked(
            clone!(@weak entry, @weak chat_box_layout, @weak window => move |button| {
                let gemini_config = config.clone();
                let entry_text = entry.text();
                let selected_model = model_combobox.active_text().unwrap().to_string();

                if !entry_text.is_empty() {

                    let answer_box = Box::new(gtk::Orientation::Vertical, 0);

                    let new_question_label = Self::new_label(entry_text.as_str(), true, false);

                    answer_box.pack_start(&new_question_label, false, false, 0);
                    answer_box.set_halign(gtk::Align::End);
                    answer_box.style_context().add_class("label-user");

                    chat_box_layout
                        .pack_start(&answer_box, false, false, 0);
                    entry.delete_text(0, -1);
                    entry.set_sensitive(false);
                    button.set_sensitive(false);
                    model_combobox.set_sensitive(false);
                    window.show_all();

                    runtime().spawn(clone!(@strong sender => async move {
                        let response = models::select_model(&selected_model, &entry_text, gemini_config).await;
                        sender.send(response).await.expect("The channel needs to be open.");
                    }));
                }
            }),
        );

        // Handles the api call to the llm and adds Label widget.
        glib::spawn_future_local(
            clone!(@weak chat_box_layout, @weak window, @weak entry => async move {
                while let Ok(response) = receiver.recv().await {
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

                    let answer_box = Box::new(gtk::Orientation::Vertical, 0);

                    for block in md2pango(&label_content) {
                        let label_model = Self::new_label(&block.string, false, block.is_code);
                        answer_box.pack_start(&label_model, false, false, 0);
                        answer_box.set_halign(gtk::Align::Start);
                        answer_box.style_context().add_class("label-model");
                    }
                    chat_box_layout.pack_start(&answer_box, false, false, 0);
                    window.show_all();
                }
            }),
        );

        window.show_all();
    }

    pub fn update(chat_area: &Box) -> bool {
        let chats = &Cache::read();
        if chats["chat"] != json!([]) {
            for chat in chats["chat"].as_array().unwrap() {
                let answer = chat["text"].as_str().unwrap();
                let answer_box = Box::new(gtk::Orientation::Vertical, 0);

                if chat["role"] == "user" {
                    let label_user = Self::new_label(answer, true, false);
                    answer_box.pack_start(&label_user, false, false, 0);
                    answer_box.set_halign(gtk::Align::End);
                    answer_box.style_context().add_class("label-user");
                } else {
                    for block in md2pango(answer) {
                        let label_model = Self::new_label(&block.string, false, block.is_code);
                        answer_box.pack_start(&label_model, true, true, 0);
                        answer_box.set_halign(gtk::Align::Start);
                        answer_box.style_context().add_class("label-model");
                    }
                };

                chat_area.pack_start(&answer_box, false, false, 0);
            }
            true
        } else {
            false
        }
    }

    fn new_label(content: &str, is_user: bool, is_code: bool) -> Label {
        let answer_label = Label::new(None);
        answer_label.set_selectable(true);
        answer_label.set_wrap(true);
        if is_user {
            answer_label.set_justify(gtk::Justification::Right);
            answer_label.set_text(content);
        } else {
            answer_label.set_justify(gtk::Justification::Fill);
            if is_code {
                answer_label.set_text(content);
                answer_label.style_context().add_class("label-model-code");
            } else {
                answer_label.set_markup(content);
            }
        }
        answer_label
    }
}

#[tokio::main]
async fn main() {
    let config = Config::new();
    let app = Application::builder()
        .application_id("org.gtk.converse")
        .build();

    // only allows single instance of program.
    app.register(None::<&gio::Cancellable>).unwrap();
    if app.is_remote() {
        return;
    }

    app.connect_activate(move |app| {
        UI::build_ui(app, config.clone());
    });
    app.run();
}
