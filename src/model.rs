//! The shape of a dictionary entry, independent of where it came from.

#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub word: String,
    pub pronunciations: Vec<Pronunciation>,
    pub sections: Vec<Section>,
}

impl Entry {
    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(|s| s.senses.is_empty())
    }

    /// Put the parts of speech that actually define the word before those that
    /// only record an inflection. Wiktionary lists `compelling` as a verb form
    /// before the adjective, which buries the senses a reader wants. Ordering
    /// is stable and keyed on the etymology first, so sense groups stay whole.
    pub fn order_sections(&mut self) {
        self.sections
            .sort_by_key(|s| (s.etymology.unwrap_or(0), s.is_inflections_only()));
    }

    /// Rewrite American spellings in the fetched text to Canadian ones.
    pub fn canadianize(&mut self) {
        for section in &mut self.sections {
            for sense in &mut section.senses {
                sense.canadianize();
            }
        }
    }

    /// When every sense merely points at another spelling, the word that
    /// actually carries the definitions. Looking up `colour` lands on such a
    /// page, with the content filed under `color`.
    pub fn spelling_pointer(&self) -> Option<String> {
        let mut target: Option<String> = None;

        for section in &self.sections {
            for sense in &section.senses {
                if !sense.subsenses.is_empty() {
                    return None;
                }
                let pointer = sense.form_of.as_ref()?;
                match &target {
                    None => target = Some(pointer.clone()),
                    Some(t) if t.eq_ignore_ascii_case(pointer) => {}
                    Some(_) => return None,
                }
            }
        }
        target
    }

    /// The Canadian pronunciation if Wiktionary records one, else a general
    /// (unaccented) one, else whatever is first.
    pub fn preferred_pronunciation(&self) -> Option<&Pronunciation> {
        let canadian = self
            .pronunciations
            .iter()
            .find(|p| p.accents.iter().any(|a| is_canadian_accent(a)));
        canadian
            .or_else(|| self.pronunciations.iter().find(|p| p.accents.is_empty()))
            .or_else(|| self.pronunciations.first())
    }
}

/// Wiktionary abbreviates accent tags; spell the common ones out.
fn expand_accent(accent: &str) -> &str {
    match accent.trim() {
        "CA" | "Canada" | "Canadian" => "Canada",
        "US" | "GA" | "GenAm" | "General American" => "US",
        "UK" | "RP" | "Received Pronunciation" => "UK",
        "AU" => "Australia",
        "NZ" => "New Zealand",
        "IE" => "Ireland",
        other => other,
    }
}

fn is_canadian_accent(accent: &str) -> bool {
    let a = accent.to_ascii_lowercase();
    a.contains("canada") || a.contains("canadian") || a == "ca" || a == "cad"
}

#[derive(Debug, Clone, Default)]
pub struct Pronunciation {
    /// Accent tags such as `Canada`, `US`, `General American`.
    pub accents: Vec<String>,
    /// One or more IPA transcriptions, kept in source order.
    pub ipa: Vec<String>,
}

impl Pronunciation {
    pub fn is_canadian(&self) -> bool {
        self.accents.iter().any(|a| is_canadian_accent(a))
    }

    pub fn accent_label(&self) -> Option<String> {
        if self.accents.is_empty() {
            return None;
        }
        let expanded: Vec<&str> = self.accents.iter().map(|a| expand_accent(a)).collect();
        Some(expanded.join(", "))
    }
}

/// One part-of-speech block, e.g. the `Noun` senses under `Etymology 2`.
#[derive(Debug, Clone, Default)]
pub struct Section {
    pub part_of_speech: String,
    /// Present when the word has several etymologies, so senses can be grouped.
    pub etymology: Option<usize>,
    pub senses: Vec<Sense>,
}

impl Section {
    /// True when no sense here defines the word in its own right.
    pub fn is_inflections_only(&self) -> bool {
        !self.senses.is_empty() && self.senses.iter().all(|s| s.inflection_only)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sense {
    /// Usage labels from `{{lb|en|...}}`, e.g. `Canada`, `informal`, `obsolete`.
    pub labels: Vec<String>,
    pub text: String,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub subsenses: Vec<Sense>,
    /// Set when the whole sense is just "standard spelling of X". Wiktionary
    /// files many Canadian spellings this way, with the substance under the
    /// American form.
    pub form_of: Option<String>,
    /// Set when the sense only points at another word, as "present participle
    /// of compel" does, rather than defining anything itself.
    pub inflection_only: bool,
}

/// Labels that mark a sense as not part of ordinary present-day English. These
/// are hidden unless `--all` is given, so the common meaning leads.
const NARROW_LABELS: &[&str] = &[
    "obsolete",
    "archaic",
    "dated",
    "rare",
    "historical",
    "poetic",
    "nonstandard",
    "proscribed",
    "dialectal",
];

impl Sense {
    fn canadianize(&mut self) {
        self.text = crate::canadian::canadianize(&self.text);
        for example in &mut self.examples {
            *example = crate::canadian::canadianize(example);
        }
        for synonym in &mut self.synonyms {
            *synonym = crate::canadian::canadianize(synonym);
        }
        for antonym in &mut self.antonyms {
            *antonym = crate::canadian::canadianize(antonym);
        }
        for sub in &mut self.subsenses {
            sub.canadianize();
        }
    }

    /// True when this sense is explicitly marked Canadian.
    pub fn is_canadian(&self) -> bool {
        self.labels.iter().any(|l| {
            let l = l.to_ascii_lowercase();
            l == "canada" || l == "canadian english" || l.starts_with("canadian")
        })
    }

    /// True when the sense is obsolete, rare, dialectal and so on. Canadian
    /// senses are never narrowed away — they are the point of this tool.
    pub fn is_narrow(&self) -> bool {
        if self.is_canadian() {
            return false;
        }
        self.labels.iter().any(|l| {
            let l = l.to_ascii_lowercase();
            NARROW_LABELS.iter().any(|n| l == *n)
        })
    }
}
