use phf::{phf_map, phf_ordered_map};
use regex::Regex;

use super::config::{Config, Theming};

static ESC_PATTERNS: phf::Map<&'static str, &'static str> = phf_map! {
    r"&([^gl]t;)|&" => r"&amp;$1",
    r"<!--.*-->" => r"",
    r"<" => r"&lt;",
    r">" => r"&gt;"
};

static PATTERNS: phf::OrderedMap<&'static str, &'static str> = phf_ordered_map! {
    r"^[-\*] (.*)" => r" • $1",
    r"\*\*\*(.*?)\*\*\*" => r"<b><i>$1</i></b>",
    r"\*\*(.*?)\*\*" => r"<b>$1</b>",
    r"\*(.+?)\*" => r"<i>$1</i>",
    r"~~(.*?)~~" => r"<s>$1</s>",
    r"\[(.*)\]\((.*)\)" => r"<a href='$2'>$1</a>",
    r"^###### (.*)" => r"<big>$1</big>",
    r"^##### (.*)" => r"<big><big>$1</big></big>",
    r"^#### (.*)" => r"<big><big><big>$1</big></big></big>",
    r"^### (.*)" => r"<big><big><big><big>$1</big></big></big></big>",
    r"^## (.*)" => r"<big><big><big><big><big>$1</big></big></big></big></big>",
    r"^# (.*)" => r"<big><big><big><big><big><big>$1</big></big></big></big></big></big>",
};

#[derive(Debug)]
pub struct FormattedCode {
    pub string: String,
    pub is_code: bool,
}

pub fn md2pango(input: &str, config: &Config) -> Vec<FormattedCode> {
    let mut final_block = Vec::new();

    let code_block_regex = Regex::new(r"```[\s\S]*?```").unwrap();

    let themes = &config.theming;
    let mut last_end = 0;
    for code_block in code_block_regex.find_iter(&input) {
        final_block.push(general_block_parse(
            &input[last_end..code_block.start()],
            &themes,
        ));
        final_block.push(code_block_parse(
            &input[code_block.start()..code_block.end()],
        ));
        last_end = code_block.end();
    }

    final_block.push(general_block_parse(&input[last_end..], &themes));

    final_block
}

fn general_block_parse(block: &str, themes: &Theming) -> FormattedCode {
    let mut pango_str = String::new();
    for line in block.split("\n") {
        let mut line_with_pango = String::from(line);

        for pattern in &ESC_PATTERNS {
            let re = Regex::new(pattern.0).expect("error.");
            line_with_pango = re.replace_all(&line_with_pango, *pattern.1).to_string();
        }

        for pattern in &PATTERNS {
            let re = Regex::new(pattern.0).expect("error.");
            line_with_pango = re.replace_all(&line_with_pango, *pattern.1).to_string();
        }

        let re = Regex::new("^&gt;(.*)").expect("error.");
        line_with_pango = re
            .replace_all(
                &line_with_pango,
                format!(
                    "<span foreground='{}'>╏</span><span foreground='{}'> $1</span>",
                    themes.quote_indicator, themes.quote_foreground
                ),
            )
            .to_string();
        let re = Regex::new(r"`([^`]*)`").expect("error.");
        line_with_pango = re
            .replace_all(
                &line_with_pango,
                format!(
                    "<span foreground='{}' background='{}'><tt>$1</tt></span>",
                    themes.code_foreground, themes.code_background
                ),
            )
            .to_string();

        pango_str.push_str(&line_with_pango);
        pango_str.push_str("\n");
    }
    FormattedCode {
        string: pango_str.trim().to_string(),
        is_code: false,
    }
}

fn code_block_parse(block: &str) -> FormattedCode {
    let re = Regex::new("```(.*)").unwrap();
    let result = re.replace_all(block, "");
    FormattedCode {
        string: result.trim().to_string(),
        is_code: true,
    }
}
