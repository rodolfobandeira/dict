//! Parses the English section of a Wiktionary page into an [`Entry`].
//!
//! Wiktionary's REST definition endpoint returns rendered HTML, but it expands
//! usage labels away — which loses exactly the `{{lb|en|Canada}}` markers and
//! `a=Canada` pronunciations this dictionary is built around. So we read the
//! raw wikitext and parse it ourselves.

use crate::markup::{self, Template};
use crate::model::{Entry, Pronunciation, Section, Sense};

/// Headings that introduce definitions.
const PARTS_OF_SPEECH: &[&str] = &[
    "Noun",
    "Proper noun",
    "Verb",
    "Adjective",
    "Adverb",
    "Pronoun",
    "Preposition",
    "Postposition",
    "Conjunction",
    "Interjection",
    "Determiner",
    "Article",
    "Numeral",
    "Number",
    "Particle",
    "Phrase",
    "Prepositional phrase",
    "Proverb",
    "Idiom",
    "Contraction",
    "Abbreviation",
    "Initialism",
    "Acronym",
    "Symbol",
    "Letter",
    "Prefix",
    "Suffix",
    "Infix",
    "Interfix",
    "Counter",
    "Ideophone",
    "Participle",
];

pub fn parse(word: &str, wikitext: &str) -> Entry {
    let mut entry = Entry { word: word.to_string(), ..Default::default() };

    let Some(english) = english_section(wikitext) else {
        return entry;
    };

    let mut etymology: Option<usize> = None;
    let mut in_pronunciation = false;
    let mut current: Option<Section> = None;

    for line in english.lines() {
        let line = line.trim_end();

        if let Some((level, title)) = heading(line) {
            let _ = level;
            if let Some(section) = current.take() {
                push_section(&mut entry, section);
            }
            in_pronunciation = false;

            if let Some(n) = etymology_index(&title) {
                etymology = Some(n);
            } else if title.eq_ignore_ascii_case("Pronunciation") {
                in_pronunciation = true;
            } else if let Some(pos) = part_of_speech(&title) {
                current = Some(Section {
                    part_of_speech: pos.to_string(),
                    etymology,
                    senses: Vec::new(),
                });
            }
            continue;
        }

        if in_pronunciation {
            collect_pronunciations(line, &mut entry.pronunciations);
            continue;
        }

        if let Some(section) = current.as_mut() {
            consume_definition_line(line, section);
        }
    }

    if let Some(section) = current.take() {
        push_section(&mut entry, section);
    }

    // A word with a single etymology does not need the grouping shown.
    let mut groups: Vec<usize> = entry.sections.iter().filter_map(|s| s.etymology).collect();
    groups.dedup();
    if groups.len() <= 1 {
        for section in &mut entry.sections {
            section.etymology = None;
        }
    }

    entry.order_sections();
    entry
}

fn push_section(entry: &mut Entry, section: Section) {
    if !section.senses.is_empty() {
        entry.sections.push(section);
    }
}

/// Slice out `==English==` up to the next language heading.
fn english_section(wikitext: &str) -> Option<&str> {
    let mut start = None;
    let mut lines = wikitext.char_indices().peekable();
    let _ = &mut lines;

    let mut offset = 0usize;
    for line in wikitext.split_inclusive('\n') {
        let trimmed = line.trim();
        match start {
            None => {
                if trimmed == "==English==" {
                    start = Some(offset + line.len());
                }
            }
            Some(begin) => {
                // Any other level-2 heading ends the English section.
                if trimmed.starts_with("==")
                    && !trimmed.starts_with("===")
                    && trimmed.ends_with("==")
                    && trimmed.len() > 4
                {
                    return Some(&wikitext[begin..offset]);
                }
            }
        }
        offset += line.len();
    }

    start.map(|begin| &wikitext[begin..])
}

/// `===Noun===` becomes `(3, "Noun")`.
fn heading(line: &str) -> Option<(usize, String)> {
    let line = line.trim();
    if !line.starts_with('=') || !line.ends_with('=') {
        return None;
    }
    let level = line.chars().take_while(|c| *c == '=').count();
    let trailing = line.chars().rev().take_while(|c| *c == '=').count();
    if level < 2 || level != trailing || line.len() <= level * 2 {
        return None;
    }
    let title = line[level..line.len() - level].trim().to_string();
    Some((level, title))
}

fn etymology_index(title: &str) -> Option<usize> {
    let rest = title.strip_prefix("Etymology").or_else(|| title.strip_prefix("etymology"))?;
    let rest = rest.trim();
    if rest.is_empty() {
        Some(1)
    } else {
        rest.parse().ok()
    }
}

fn part_of_speech(title: &str) -> Option<&'static str> {
    // Headings are occasionally numbered, e.g. `Noun 2`.
    let base = title.trim_end_matches(|c: char| c.is_ascii_digit() || c.is_whitespace());
    PARTS_OF_SPEECH
        .iter()
        .find(|pos| pos.eq_ignore_ascii_case(base))
        .copied()
}

/// Pull IPA transcriptions and their accent tags out of a `* {{IPA|...}}` line.
fn collect_pronunciations(line: &str, out: &mut Vec<Pronunciation>) {
    let line = line.trim_start_matches(['*', ' ', '\t']);
    if !line.starts_with("{{") {
        return;
    }

    let mut accents: Vec<String> = Vec::new();

    for body in top_level_templates(line) {
        let t = Template::parse(&body);
        match t.name.as_str() {
            // `{{a|Canada}}` tags the rest of the line.
            "a" | "accent" | "aa" => {
                accents.extend(t.positional.iter().map(|s| s.trim().to_string()));
            }
            "ipa" | "ipachar" => {
                let mut p = Pronunciation { accents: accents.clone(), ipa: Vec::new() };
                if let Some(a) = t.named("a") {
                    p.accents.extend(a.split(',').map(|s| s.trim().to_string()));
                }
                for arg in t.args_after_lang() {
                    let ipa = arg.trim();
                    if ipa.starts_with('/') || ipa.starts_with('[') {
                        p.ipa.push(markup::render_plain(ipa));
                    }
                }
                p.accents.retain(|a| !a.is_empty());
                if !p.ipa.is_empty() {
                    out.push(p);
                }
            }
            _ => {}
        }
    }
}

/// The bodies of every `{{...}}` at the top level of a line.
fn top_level_templates(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars.get(i) == Some(&'{') && chars.get(i + 1) == Some(&'{') {
            let mut depth = 0usize;
            let start = i + 2;
            while i < chars.len() {
                if chars.get(i) == Some(&'{') && chars.get(i + 1) == Some(&'{') {
                    depth += 1;
                    i += 2;
                } else if chars.get(i) == Some(&'}') && chars.get(i + 1) == Some(&'}') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        found.push(chars[start..i - 2].iter().collect());
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    found
}

/// What a `#`-prefixed line carries.
enum LineKind {
    /// A definition at the given nesting depth.
    Definition(usize),
    /// An example, synonym or antonym attached to the sense at this depth.
    Annotation(usize),
    /// A quotation, which we skip as too long for terminal output.
    Quotation,
}

fn classify(line: &str) -> Option<(LineKind, &str)> {
    if !line.starts_with('#') {
        return None;
    }
    let depth = line.chars().take_while(|c| *c == '#').count();
    let rest = &line[depth..];

    if rest.starts_with('*') {
        Some((LineKind::Quotation, ""))
    } else if let Some(body) = rest.strip_prefix(':') {
        Some((LineKind::Annotation(depth), body.trim()))
    } else {
        Some((LineKind::Definition(depth), rest.trim()))
    }
}

fn consume_definition_line(line: &str, section: &mut Section) {
    let Some((kind, body)) = classify(line.trim()) else {
        return;
    };

    match kind {
        LineKind::Quotation => {}
        LineKind::Definition(depth) => {
            let sense = build_sense(body);
            if depth <= 1 {
                section.senses.push(sense);
            } else if let Some(parent) = section.senses.last_mut() {
                parent.subsenses.push(sense);
            } else {
                section.senses.push(sense);
            }
        }
        LineKind::Annotation(depth) => {
            // An annotation belongs to the most recent sense at its depth,
            // falling back to the parent when there is no subsense yet.
            if let Some(last) = section.senses.last_mut() {
                let target = if depth > 1 && !last.subsenses.is_empty() {
                    last.subsenses.last_mut().unwrap()
                } else {
                    last
                };
                apply_annotation(body, target);
            }
        }
    }
}

/// Split leading `{{lb|en|...}}` labels off a definition and render the rest.
fn build_sense(body: &str) -> Sense {
    let mut sense = Sense::default();
    let mut rest = body.trim();

    while rest.starts_with("{{") {
        let chars: Vec<char> = rest.chars().collect();
        let (template, consumed) = balanced_template(&chars);
        let t = Template::parse(&template);
        if matches!(t.name.as_str(), "lb" | "label" | "lbl" | "tlb") {
            sense.labels.extend(
                t.args_after_lang()
                    .iter()
                    .map(|a| markup::render_plain(a))
                    .filter(|a| !a.is_empty() && a != "_" && a != "and" && a != "or"),
            );
            rest = rest[consumed..].trim_start();
        } else {
            break;
        }
    }

    sense.form_of = sole_template(rest)
        .filter(|t| SPELLING_POINTERS.contains(&t.name.as_str()))
        .map(|t| {
            t.args_after_lang()
                .first()
                .map(|a| markup::render_plain(a))
                .unwrap_or_default()
        })
        .filter(|target| !target.is_empty());
    sense.inflection_only =
        sole_template(rest).is_some_and(|t| markup::is_form_of_template(&t.name));
    sense.text = markup::render(rest);
    sense
}

/// Templates that make an entry a pointer to another spelling rather than a
/// definition in its own right.
const SPELLING_POINTERS: &[&str] = &[
    "standard spelling of",
    "stand sp",
    "alternative spelling of",
    "alt sp",
    "altsp",
    "alternative form of",
    "alt form",
    "altform",
];

/// When a definition consists of exactly one template, that template. A
/// definition with prose around the template defines something itself, and so
/// is neither a pointer nor a bare inflection.
fn sole_template(body: &str) -> Option<Template> {
    let body = body.trim();
    if !body.starts_with("{{") {
        return None;
    }
    let chars: Vec<char> = body.chars().collect();
    let (template, consumed) = balanced_template(&chars);

    if !body[consumed..].trim().trim_end_matches('.').is_empty() {
        return None;
    }
    Some(Template::parse(&template))
}

/// Length in bytes of the balanced `{{...}}` starting at index 0, with its body.
fn balanced_template(chars: &[char]) -> (String, usize) {
    let mut depth = 0usize;
    let mut i = 0;
    let mut bytes = 0usize;
    let mut body_start = 0usize;

    while i < chars.len() {
        if chars.get(i) == Some(&'{') && chars.get(i + 1) == Some(&'{') {
            depth += 1;
            if depth == 1 {
                body_start = i + 2;
            }
            bytes += chars[i].len_utf8() + chars[i + 1].len_utf8();
            i += 2;
        } else if chars.get(i) == Some(&'}') && chars.get(i + 1) == Some(&'}') {
            depth -= 1;
            bytes += chars[i].len_utf8() + chars[i + 1].len_utf8();
            i += 2;
            if depth == 0 {
                return (chars[body_start..i - 2].iter().collect(), bytes);
            }
        } else {
            bytes += chars[i].len_utf8();
            i += 1;
        }
    }
    (String::new(), bytes)
}

/// A `#:` line is a usage example, a synonym list or an antonym list.
fn apply_annotation(body: &str, sense: &mut Sense) {
    let body = body.trim();

    if body.starts_with("{{") {
        let templates = top_level_templates(body);
        let mut handled = false;

        for tb in &templates {
            let t = Template::parse(tb);
            match t.name.as_str() {
                "syn" | "synonyms" => {
                    sense.synonyms.extend(collect_terms(&t));
                    handled = true;
                }
                "ant" | "antonyms" => {
                    sense.antonyms.extend(collect_terms(&t));
                    handled = true;
                }
                "ux" | "usex" | "uxi" | "ux-lite" | "eg" => {
                    let text = markup::render(t.args_after_lang().first().map(String::as_str).unwrap_or(""));
                    if !text.is_empty() {
                        sense.examples.push(text);
                    }
                    handled = true;
                }
                _ => {}
            }
        }
        if handled {
            return;
        }
    }

    let text = markup::render(body);
    if !text.is_empty() {
        sense.examples.push(text);
    }
}

fn collect_terms(t: &Template) -> Vec<String> {
    t.args_after_lang()
        .iter()
        .map(|a| markup::render_plain(a))
        // `{{syn|en|ride#Noun}}` targets a section of another page; the reader
        // wants the word, not the anchor.
        .map(|a| a.split('#').next().unwrap_or(&a).trim().to_string())
        .filter(|a| !a.is_empty() && !a.starts_with("Thesaurus:"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOQUE: &str = r#"{{also|toqué}}
==English==

===Etymology 1===
From {{der|en|frm|toque}}.

====Pronunciation====
* {{IPA|en|/təʊk/|a=UK}}
* {{IPA|en|/toʊk/|a=US}}

====Noun====
{{en-noun}}

# A type of [[hat]] with no [[brim]].
#* {{quote-book|en|year=1824|text=Black velvet '''toques''' are ornamented.}}
# {{lb|en|specifically}} A [[tall]] [[white]] hat worn by [[chef]]s.
## A very tall one.
##: {{ux|en|The '''toque''' towered over the kitchen.}}
# {{lb|en|historical}} An African money of account.

===Etymology 2===

====Pronunciation====
* {{IPA|en|/tuːk/|/tjuːk/|a=Canada}}

====Noun====
{{en-noun}}

# {{lb|en|Canada}} A [[knitted]] [[hat]], often [[woollen]].
#: {{syn|en|beanie|watch cap}}
#: {{ant|en|sunhat}}
#: {{ux|en|Wear a '''toque''', it is minus thirty.}}

==French==

===Noun===
# a French sense that must not appear
"#;

    fn parsed() -> Entry {
        parse("toque", TOQUE)
    }

    #[test]
    fn stops_at_the_next_language() {
        let entry = parsed();
        let all: String = entry
            .sections
            .iter()
            .flat_map(|s| s.senses.iter())
            .map(|s| s.text.clone())
            .collect();
        assert!(!all.contains("French sense"));
    }

    #[test]
    fn groups_senses_by_etymology() {
        let entry = parsed();
        assert_eq!(entry.sections.len(), 2);
        assert_eq!(entry.sections[0].etymology, Some(1));
        assert_eq!(entry.sections[1].etymology, Some(2));
        assert_eq!(entry.sections[0].part_of_speech, "Noun");
    }

    #[test]
    fn reads_labels_and_marks_canadian_senses() {
        let entry = parsed();
        let canadian = &entry.sections[1].senses[0];
        assert_eq!(canadian.labels, ["Canada"]);
        assert!(canadian.is_canadian());

        let specific = &entry.sections[0].senses[1];
        assert_eq!(specific.labels, ["specifically"]);
        assert!(!specific.is_canadian());
        assert_eq!(specific.text, "A tall white hat worn by chefs.");
    }

    #[test]
    fn skips_quotations_but_keeps_usage_examples() {
        let entry = parsed();
        assert!(entry.sections[0].senses[0].examples.is_empty());
        assert_eq!(
            entry.sections[1].senses[0].examples,
            ["Wear a \u{1}toque\u{2}, it is minus thirty."]
        );
    }

    #[test]
    fn reads_synonyms_and_antonyms() {
        let entry = parsed();
        let sense = &entry.sections[1].senses[0];
        assert_eq!(sense.synonyms, ["beanie", "watch cap"]);
        assert_eq!(sense.antonyms, ["sunhat"]);
    }

    #[test]
    fn attaches_subsenses_and_their_examples() {
        let entry = parsed();
        let parent = &entry.sections[0].senses[1];
        assert_eq!(parent.subsenses.len(), 1);
        assert_eq!(parent.subsenses[0].text, "A very tall one.");
        assert_eq!(parent.subsenses[0].examples.len(), 1);
        // The example on the subsense must not leak onto the parent.
        assert!(parent.examples.is_empty());
    }

    #[test]
    fn prefers_the_canadian_pronunciation() {
        let entry = parsed();
        let p = entry.preferred_pronunciation().expect("a pronunciation");
        assert!(p.is_canadian());
        assert_eq!(p.ipa, ["/tuːk/", "/tjuːk/"]);
    }

    #[test]
    fn hides_narrow_senses_but_never_canadian_ones() {
        let entry = parsed();
        assert!(entry.sections[0].senses[2].is_narrow()); // historical
        assert!(!entry.sections[1].senses[0].is_narrow()); // Canada
    }

    #[test]
    fn detects_a_spelling_pointer_page() {
        let colour = parse(
            "colour",
            "==English==\n\n===Noun===\n# {{standard spelling of|en|from=Commonwealth|color}}.\n\
             \n===Verb===\n# {{stand sp|en|color}}.\n",
        );
        assert_eq!(colour.spelling_pointer().as_deref(), Some("color"));
    }

    #[test]
    fn a_page_with_real_senses_is_not_a_pointer() {
        // `defence` carries its own definitions, so it must not be followed.
        let defence = parse(
            "defence",
            "==English==\n\n===Noun===\n# The action of [[defend]]ing.\n\
             # {{alternative spelling of|en|defense}}.\n",
        );
        assert_eq!(defence.spelling_pointer(), None);
        assert!(parsed().spelling_pointer().is_none());
    }

    #[test]
    fn an_entry_without_an_english_section_is_empty() {
        let entry = parse("nada", "==Spanish==\n\n===Noun===\n# nothing\n");
        assert!(entry.is_empty());
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    #[test]
    fn synonyms_drop_section_anchors_and_thesaurus_links() {
        let entry = parse(
            "riding",
            "==English==\n\n===Noun===\n# A path cut through woodland.\n\
             #: {{syn|en|royd#Noun|thwaite#Noun|Thesaurus:path}}\n",
        );
        assert_eq!(entry.sections[0].senses[0].synonyms, ["royd", "thwaite"]);
    }
}

#[cfg(test)]
mod ordering_tests {
    use super::*;

    const COMPELLING: &str = "==English==\n\n===Verb===\n{{head|en|verb form}}\n\n\
        # {{infl of|en|compel||ing-form}}\n\n\
        ===Adjective===\n{{en-adj}}\n\n# very [[interesting]]\n";

    #[test]
    fn definitions_come_before_bare_inflections() {
        let entry = parse("compelling", COMPELLING);
        assert_eq!(entry.sections[0].part_of_speech, "Adjective");
        assert_eq!(entry.sections[1].part_of_speech, "Verb");
        assert!(entry.sections[1].is_inflections_only());
        assert!(!entry.sections[0].is_inflections_only());
    }

    #[test]
    fn a_sense_with_prose_around_a_template_still_defines_something() {
        let entry = parse(
            "x",
            "==English==\n\n===Noun===\n# a kind of {{l|en|thing}} found nearby\n",
        );
        assert!(!entry.sections[0].senses[0].inflection_only);
        assert!(entry.spelling_pointer().is_none());
    }

    #[test]
    fn sense_groups_are_not_broken_up_by_reordering() {
        let entry = parse(
            "toque",
            "==English==\n\n===Etymology 1===\n\n====Verb====\n# {{plural of|en|toc}}\n\n\
             ====Noun====\n# A hat.\n\n===Etymology 2===\n\n====Noun====\n# A knitted hat.\n",
        );
        let groups: Vec<_> = entry.sections.iter().map(|s| s.etymology).collect();
        assert_eq!(groups, [Some(1), Some(1), Some(2)]);
        // Within group 1, the real definition now leads.
        assert_eq!(entry.sections[0].part_of_speech, "Noun");
    }
}
