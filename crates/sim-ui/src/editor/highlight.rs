use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, TextStyle};

/// VS Code Dark+ inspired palette.
const DEFAULT: Color32 = Color32::from_rgb(212, 212, 212);
const KEYWORD: Color32 = Color32::from_rgb(197, 134, 192);
const TYPE: Color32 = Color32::from_rgb(78, 201, 176);
const BUILTIN: Color32 = Color32::from_rgb(86, 156, 214);
const FUNCTION: Color32 = Color32::from_rgb(220, 220, 170);
const NUMBER: Color32 = Color32::from_rgb(181, 206, 168);
const STRING: Color32 = Color32::from_rgb(206, 145, 120);
const COMMENT: Color32 = Color32::from_rgb(106, 153, 85);
const PREPROCESSOR: Color32 = Color32::from_rgb(197, 134, 192);

/// Control-flow keywords.
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "break"
            | "continue"
            | "return"
            | "goto"
    )
}

/// Built-in types and qualifiers.
fn is_type(word: &str) -> bool {
    matches!(
        word,
        "void"
            | "int"
            | "float"
            | "double"
            | "char"
            | "bool"
            | "byte"
            | "word"
            | "long"
            | "short"
            | "unsigned"
            | "signed"
            | "const"
            | "static"
            | "volatile"
            | "struct"
            | "class"
            | "enum"
            | "union"
            | "public"
            | "private"
            | "protected"
            | "new"
            | "delete"
            | "sizeof"
            | "typedef"
            | "namespace"
            | "template"
            | "typename"
            | "auto"
            | "String"
    )
}

/// Arduino constants and literals.
fn is_builtin(word: &str) -> bool {
    matches!(
        word,
        "HIGH"
            | "LOW"
            | "INPUT"
            | "OUTPUT"
            | "INPUT_PULLUP"
            | "LED_BUILTIN"
            | "true"
            | "false"
            | "NULL"
            | "nullptr"
    ) || matches!(word, "A0" | "A1" | "A2" | "A3" | "A4" | "A5")
}

/// Builds a highlighted layout job for the given source text.
pub fn highlight(text: &str, style: &egui::Style) -> LayoutJob {
    let size = style
        .text_styles
        .get(&TextStyle::Monospace)
        .map(|font| font.size)
        .unwrap_or(13.0);
    let font = FontId::monospace(size);
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;

    let mut index = 0;
    let mut at_line_start = true;
    while index < text.len() {
        let rest = &text[index..];
        let ch = rest.chars().next().unwrap_or('\n');

        if ch == '\n' {
            append(&mut job, "\n", DEFAULT, &font);
            index += ch.len_utf8();
            at_line_start = true;
            continue;
        }
        if ch == ' ' || ch == '\t' {
            let end = rest
                .find(|candidate: char| candidate != ' ' && candidate != '\t')
                .map(|offset| index + offset)
                .unwrap_or(text.len());
            append(&mut job, &text[index..end], DEFAULT, &font);
            index = end;
            continue;
        }
        if rest.starts_with("//") {
            let end = rest
                .find('\n')
                .map(|offset| index + offset)
                .unwrap_or(text.len());
            append(&mut job, &text[index..end], COMMENT, &font);
            index = end;
            at_line_start = false;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix("/*") {
            let end = stripped
                .find("*/")
                .map(|offset| index + 2 + offset + 2)
                .unwrap_or(text.len());
            append(&mut job, &text[index..end], COMMENT, &font);
            index = end;
            at_line_start = false;
            continue;
        }
        if ch == '#' && at_line_start {
            let end = rest
                .find('\n')
                .map(|offset| index + offset)
                .unwrap_or(text.len());
            append(&mut job, &text[index..end], PREPROCESSOR, &font);
            index = end;
            at_line_start = false;
            continue;
        }
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut end = index + ch.len_utf8();
            let mut escaped = false;
            for (offset, candidate) in text[end..].char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if candidate == '\\' {
                    escaped = true;
                    continue;
                }
                if candidate == quote {
                    end = end + offset + candidate.len_utf8();
                    break;
                }
                if candidate == '\n' {
                    break;
                }
            }
            append(&mut job, &text[index..end.max(index + 1)], STRING, &font);
            index = end.max(index + 1);
            at_line_start = false;
            continue;
        }
        if ch.is_ascii_digit() {
            let end = scan_while(rest, |candidate| {
                candidate.is_ascii_alphanumeric() || candidate == '.' || candidate == '_'
            });
            append(&mut job, &text[index..index + end], NUMBER, &font);
            index += end;
            at_line_start = false;
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let end = scan_while(rest, |candidate| {
                candidate.is_ascii_alphanumeric() || candidate == '_'
            });
            let word = &text[index..index + end];
            let color = classify(word, &text[index + end..]);
            append(&mut job, word, color, &font);
            index += end;
            at_line_start = false;
            continue;
        }

        append(
            &mut job,
            &text[index..index + ch.len_utf8()],
            DEFAULT,
            &font,
        );
        index += ch.len_utf8();
        at_line_start = false;
    }

    job
}

/// Picks a color for an identifier, treating `name(` as a function call.
fn classify(word: &str, after: &str) -> Color32 {
    if is_keyword(word) {
        KEYWORD
    } else if is_type(word) {
        TYPE
    } else if is_builtin(word) {
        BUILTIN
    } else if after.trim_start().starts_with('(') {
        FUNCTION
    } else {
        DEFAULT
    }
}

/// Consumes characters from `text` while `predicate` holds, returning the byte length.
fn scan_while(text: &str, predicate: impl Fn(char) -> bool) -> usize {
    let mut length = 0;
    for candidate in text.chars() {
        if !predicate(candidate) {
            break;
        }
        length += candidate.len_utf8();
    }
    length
}

/// Appends a colored span to the layout job.
fn append(job: &mut LayoutJob, text: &str, color: Color32, font: &FontId) {
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font.clone(),
            color,
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colors(text: &str) -> Vec<(String, Color32)> {
        let job = highlight(text, &egui::Style::default());
        job.sections
            .iter()
            .map(|section| {
                (
                    job.text[section.byte_range.clone()].to_string(),
                    section.format.color,
                )
            })
            .collect()
    }

    #[test]
    fn test_comment_string_number_and_keyword() {
        let spans = colors("if (x) { // note\n  Serial.print(\"Hi\", 42); }");
        assert!(
            spans
                .iter()
                .any(|(text, color)| text == "if" && *color == KEYWORD)
        );
        assert!(
            spans
                .iter()
                .any(|(text, color)| text == "// note" && *color == COMMENT)
        );
        assert!(
            spans
                .iter()
                .any(|(text, color)| text == "\"Hi\"" && *color == STRING)
        );
        assert!(
            spans
                .iter()
                .any(|(text, color)| text == "42" && *color == NUMBER)
        );
    }

    #[test]
    fn test_preprocessor_and_function() {
        let spans = colors("#include <Wire.h>\nsetup();");
        assert!(
            spans
                .iter()
                .any(|(text, color)| text.starts_with("#include") && *color == PREPROCESSOR)
        );
        assert!(
            spans
                .iter()
                .any(|(text, color)| text == "setup" && *color == FUNCTION)
        );
    }
}
