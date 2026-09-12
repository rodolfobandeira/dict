//! Terminal presentation.

use std::io::IsTerminal;

use crate::canadian::{Canadianism, SpellingMatch};
use crate::markup::{EMPH_END, EMPH_START};
use crate::model::Entry;

const MAPLE: &str = "\u{1F341}";

pub struct Style {
    colour: bool,
    pub width: usize,
}

impl Style {
    pub fn new(force_off: bool) -> Style {
        let colour = !force_off
            && std::env::var_os("NO_COLOR").is_none()
            && std::io::stdout().is_terminal();
        Style { colour, width: terminal_width() }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.colour {
            format!("\u{1b}[{code}m{text}\u{1b}[0m")
        } else {
            text.to_string()
        }
    }

    fn headword(&self, s: &str) -> String {
        self.paint("1;36", s)
    }
    fn pos(&self, s: &str) -> String {
        self.paint("1;33", s)
    }
    fn dim(&self, s: &str) -> String {
        self.paint("2", s)
    }
    fn label(&self, s: &str) -> String {
        self.paint("3;35", s)
    }
    fn maple(&self, s: &str) -> String {
        self.paint("1;31", s)
    }
    fn example(&self, s: &str) -> String {
        self.paint("2;3", s)
    }

    /// Replace emphasis markers with bold, or remove them.
    fn emphasis(&self, s: &str) -> String {
        if self.colour {
            s.replace(EMPH_START, "\u{1b}[1m").replace(EMPH_END, "\u{1b}[22m")
        } else {
            s.chars().filter(|c| *c != EMPH_START && *c != EMPH_END).collect()
        }
    }
}

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<usize>().ok())
        .filter(|w| *w >= 40)
        .unwrap_or(80)
        .min(100)
}

/// Wrap text to `width`, prefixing the first line with `first_indent` and the
/// rest with `rest_indent`. Emphasis markers do not count toward the width.
fn wrap(text: &str, width: usize, first_indent: &str, rest_indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    let mut indent = first_indent;

    for word in text.split_whitespace() {
        let w = display_width(word);
        let budget = width.saturating_sub(indent.chars().count());

        if current.is_empty() {
            current.push_str(word);
            current_width = w;
        } else if current_width + 1 + w <= budget {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + w;
        } else {
            lines.push(format!("{indent}{current}"));
            indent = rest_indent;
            current = word.to_string();
            current_width = w;
        }
    }
    if !current.is_empty() {
        lines.push(format!("{indent}{current}"));
    }
    if lines.is_empty() {
        lines.push(first_indent.to_string());
    }
    lines
}

fn display_width(s: &str) -> usize {
    s.chars().filter(|c| *c != EMPH_START && *c != EMPH_END).count()
}

fn print_wrapped(style: &Style, text: &str, first_indent: &str, rest_indent: &str) {
    for line in wrap(text, style.width, first_indent, rest_indent) {
        println!("{}", style.emphasis(&line));
    }
}

/// The heading line: the word, then its Canadian pronunciation if known.
pub fn print_headword(style: &Style, entry: &Entry) {
    let mut line = style.headword(&entry.word);

    if let Some(p) = entry.preferred_pronunciation() {
        line.push_str("  ");
        line.push_str(&style.dim(&p.ipa.join(", ")));
        if let Some(accent) = p.accent_label() {
            let tag = if p.is_canadian() {
                style.maple(&format!("({accent})"))
            } else {
                style.dim(&format!("({accent})"))
            };
            line.push(' ');
            line.push_str(&tag);
        }
    }
    println!("{line}");
}

/// The Canadian spelling advice for a word, if there is any.
pub fn print_spelling_note(style: &Style, m: &SpellingMatch, redirected: bool) {
    let headline = if !m.variant.redirect {
        // The two forms are different words, or both are current in Canada;
        // the note below carries the distinction.
        format!(
            "Canadian usage: {} and {}.",
            quote(&m.canadian_form),
            quote(&m.other_form),
        )
    } else if m.queried_canadian {
        format!(
            "{} is the Canadian spelling; {} English writes {}.",
            quote(&m.canadian_form),
            m.variant.contrast.other_name(),
            quote(&m.other_form),
        )
    } else if redirected {
        format!(
            "{} is the {} spelling. Canadian English writes {} \u{2014} showing that entry.",
            quote(&m.other_form),
            m.variant.contrast.other_name(),
            quote(&m.canadian_form),
        )
    } else {
        format!(
            "{} is the {} spelling; Canadian English writes {}.",
            quote(&m.other_form),
            m.variant.contrast.other_name(),
            quote(&m.canadian_form),
        )
    };

    print_wrapped(style, &format!("{MAPLE} {headline}"), "", "   ");
    if m.variant.redirect {
        let shared = format!(
            "Canadian English agrees with {} English on this word.",
            m.variant.contrast.shared_with()
        );
        print_wrapped(style, &style.dim(&shared), "   ", "   ");
    }
    if let Some(note) = m.note() {
        print_wrapped(style, &style.dim(note), "   ", "   ");
    }
    println!();
}

pub fn print_canadianism(style: &Style, c: &Canadianism) {
    let mut headline = format!("{MAPLE} Canadianism \u{2014} {}", c.gloss);
    if let Some(region) = c.region {
        headline.push_str(&format!(" Chiefly {region}."));
    }
    print_wrapped(style, &headline, "", "   ");
    println!();
}

/// The definitions themselves.
pub fn print_entry(style: &Style, entry: &Entry, show_all: bool) {
    let mut last_etymology = None;

    for section in &entry.sections {
        let senses: Vec<_> = section
            .senses
            .iter()
            .filter(|s| show_all || !s.is_narrow())
            .collect();
        if senses.is_empty() {
            continue;
        }

        if let Some(group) = section.etymology.filter(|g| Some(*g) != last_etymology) {
            last_etymology = Some(group);
            println!();
            println!(
                "{}",
                style.dim(&format!("  \u{2500}\u{2500} sense group {group} \u{2500}\u{2500}"))
            );
        }

        println!();
        println!("  {}", style.pos(&section.part_of_speech.to_uppercase()));

        for (i, sense) in senses.iter().enumerate() {
            print_sense(style, sense, &format!("{}.", i + 1), "    ");
            for (j, sub) in sense.subsenses.iter().enumerate() {
                if show_all || !sub.is_narrow() {
                    let marker = format!("{}.", ascii_letter(j));
                    print_sense(style, sub, &marker, "        ");
                }
            }
        }
    }
}

fn ascii_letter(index: usize) -> char {
    (b'a' + (index % 26) as u8) as char
}

fn print_sense(style: &Style, sense: &crate::model::Sense, marker: &str, indent: &str) {
    let mut body = String::new();

    if sense.is_canadian() {
        body.push_str(MAPLE);
        body.push(' ');
    }
    if !sense.labels.is_empty() {
        body.push_str(&style.label(&format!("({}) ", sense.labels.join(", "))));
    }
    body.push_str(&sense.text);

    let first = format!("{indent}{marker} ");
    let rest = " ".repeat(first.chars().count());
    print_wrapped(style, &body, &first, &rest);

    let detail_indent = format!("{rest}  ");
    for example in &sense.examples {
        print_wrapped(
            style,
            &style.example(&format!("\u{201c}{example}\u{201d}")),
            &detail_indent,
            &detail_indent,
        );
    }
    if !sense.synonyms.is_empty() {
        let text = format!("synonyms: {}", sense.synonyms.join(", "));
        print_wrapped(style, &style.dim(&text), &detail_indent, &detail_indent);
    }
    if !sense.antonyms.is_empty() {
        let text = format!("antonyms: {}", sense.antonyms.join(", "));
        print_wrapped(style, &style.dim(&text), &detail_indent, &detail_indent);
    }
}

pub fn print_suggestions(style: &Style, word: &str, suggestions: &[String]) {
    eprintln!("dict: no entry for {}", quote(word));
    if !suggestions.is_empty() {
        eprintln!("{}", style.dim("      did you mean:"));
        for s in suggestions.iter().take(8) {
            eprintln!("        {s}");
        }
    }
}

pub fn print_footer(style: &Style, from_cache: bool, canadianized: bool) {
    let mut parts = vec!["Wiktionary (CC BY-SA 4.0)".to_string()];
    if from_cache {
        parts.push("offline cache".to_string());
    }
    if canadianized {
        parts.push("spellings shown in Canadian form".to_string());
    }
    println!();
    println!("{}", style.dim(&format!("  {}", parts.join(" \u{00b7} "))));
}

fn quote(s: &str) -> String {
    format!("\u{201c}{s}\u{201d}")
}

/// Explain that the definitions shown live under a different spelling.
pub fn print_filed_under(style: &Style, target: &str) {
    let note = format!("Wiktionary files these definitions under {}.", quote(target));
    print_wrapped(style, &style.dim(&note), "   ", "   ");
    println!();
}
