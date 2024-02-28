use phf::phf_map;
use regex::Regex;

static ESC_PATTERNS: phf::Map<&'static str, &'static str> = phf_map! {
    // r"&" => r"&amp;",
    r"<!--.*-->" => r"",
    r"<" => r"&lt;",
    r">" => r"&gt;"
};

static PATTERNS: phf::Map<&'static str, &'static str> = phf_map! {
    r"^\*[^\*](.*)" => r" • $1",
    r"\*\*(.*)\*\*" => r"<b>$1</b>",
    r"\[(.*)\]\((.*)\)" => r"<a href='$2'>$1</a>",
    r"#(.*)#" => r"<big>$1</big>",
    r"##(.*)##" => r"<big><big>$1</big></big>",
    r"###(.*)###" => r"<big><big><big>$1</big></big></big>",
    r"`([^`]*)`" => "<span foreground='#bbb' background='#181825'><tt>$1</tt></span>",
};

#[derive(Debug)]
pub struct FormattedCode {
    pub string: String,
    pub is_code: bool,
}

pub fn md2pango(input: &str) -> Vec<FormattedCode> {
    let mut final_block = Vec::new();

    let code_block_regex = Regex::new(r"```[\s\S]*?```").unwrap();

    let mut last_end = 0;
    for code_block in code_block_regex.find_iter(&input) {
        final_block.push(general_block_parse(&input[last_end..code_block.start()]));
        final_block.push(code_block_parse(
            &input[code_block.start()..code_block.end()],
        ));
        last_end = code_block.end();
    }

    final_block.push(general_block_parse(&input[last_end..]));

    final_block
}

fn general_block_parse(block: &str) -> FormattedCode {
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
