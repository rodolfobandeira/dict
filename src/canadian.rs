//! The Canadian English layer: spelling preferences and a glossary of
//! Canadianisms. All of it is compiled into the binary, so it works offline.
//!
//! Canadian spelling is not simply British or simply American. It keeps British
//! `-our` and `-re`, takes American `-ize` and `-yze`, and picks sides word by
//! word elsewhere. The table below records which variety Canadian agrees with
//! for each word, so `dict` can say something more useful than "British".

/// Which other variety spells the word differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contrast {
    /// Canadian agrees with British usage; American English differs.
    American,
    /// Canadian agrees with American usage; British English differs.
    British,
}

impl Contrast {
    pub fn other_name(self) -> &'static str {
        match self {
            Contrast::American => "American",
            Contrast::British => "British",
        }
    }

    pub fn shared_with(self) -> &'static str {
        match self {
            Contrast::American => "British",
            Contrast::British => "American",
        }
    }
}

pub struct Variant {
    pub canadian: &'static str,
    pub other: &'static str,
    pub contrast: Contrast,
    /// Extra guidance, for word pairs where the rule alone would mislead.
    pub note: Option<&'static str>,
    /// Whether `-s`/`-ed`/`-ing` forms may be derived from this pair. Off where
    /// a derived form collides with an unrelated word (`tire` → `tired`).
    pub inflects: bool,
    /// Whether looking up the non-Canadian form should show the Canadian entry
    /// instead. Off for pairs that are two different words in Canadian English
    /// (`cheque` and `check`) and for pairs where both spellings are current in
    /// Canada (`grey` and `gray`), since neither is simply wrong.
    pub redirect: bool,
}

const fn vn(
    canadian: &'static str,
    other: &'static str,
    contrast: Contrast,
    note: &'static str,
) -> Variant {
    Variant { canadian, other, contrast, note: Some(note), inflects: true, redirect: true }
}

/// As `vn`, but without derived forms.
const fn vf(
    canadian: &'static str,
    other: &'static str,
    contrast: Contrast,
    note: &'static str,
) -> Variant {
    Variant { canadian, other, contrast, note: Some(note), inflects: false, redirect: true }
}

/// A pair that is not simply a misspelling: either the two forms are different
/// words in Canadian English, or both are current here. Reported, never
/// substituted.
const fn vs(
    canadian: &'static str,
    other: &'static str,
    contrast: Contrast,
    note: &'static str,
) -> Variant {
    Variant { canadian, other, contrast, note: Some(note), inflects: false, redirect: false }
}

use Contrast::{American, British};

const OUR_RULE: &str = "Canadian English keeps the British -our ending, but drops the u before the suffixes -ous, -ial, -ify and -ation: humour but humorous, honour but honorary, vigour but vigorous.";
const RE_RULE: &str = "Canadian English keeps the British -re ending.";
const IZE_RULE: &str = "Canadian English uses -ize and -yze like American English, not British -ise and -yse.";
const DOUBLE_RULE: &str = "Canadian English doubles a final l before a suffix, whether or not the last syllable is stressed.";

pub const VARIANTS: &[Variant] = &[
    // -our, kept from British English.
    vn("armour", "armor", American, OUR_RULE),
    vn("behaviour", "behavior", American, OUR_RULE),
    vn("candour", "candor", American, OUR_RULE),
    vn("clamour", "clamor", American, OUR_RULE),
    vn("colour", "color", American, OUR_RULE),
    vn("demeanour", "demeanor", American, OUR_RULE),
    vn("endeavour", "endeavor", American, OUR_RULE),
    vn("favour", "favor", American, OUR_RULE),
    vn("fervour", "fervor", American, OUR_RULE),
    vn("flavour", "flavor", American, OUR_RULE),
    vn("harbour", "harbor", American, OUR_RULE),
    vn("honour", "honor", American, OUR_RULE),
    vn("humour", "humor", American, OUR_RULE),
    vn("labour", "labor", American, OUR_RULE),
    vn("neighbour", "neighbor", American, OUR_RULE),
    vn("odour", "odor", American, OUR_RULE),
    vn("parlour", "parlor", American, OUR_RULE),
    vn("rancour", "rancor", American, OUR_RULE),
    vn("rigour", "rigor", American, OUR_RULE),
    vn("rumour", "rumor", American, OUR_RULE),
    vn("saviour", "savior", American, OUR_RULE),
    vn("savour", "savor", American, OUR_RULE),
    vn("splendour", "splendor", American, OUR_RULE),
    vn("valour", "valor", American, OUR_RULE),
    vn("vapour", "vapor", American, OUR_RULE),
    vn("vigour", "vigor", American, OUR_RULE),
    vs("glamour", "glamor", American, "Both spellings are American; glamour is the usual form in Canada and, increasingly, everywhere. The adjective is glamorous on all sides."),

    // -re, kept from British English.
    vn("calibre", "caliber", American, RE_RULE),
    vn("centre", "center", American, RE_RULE),
    vn("fibre", "fiber", American, RE_RULE),
    vn("litre", "liter", American, RE_RULE),
    vn("lustre", "luster", American, RE_RULE),
    vn("manoeuvre", "maneuver", American, RE_RULE),
    vn("meagre", "meager", American, RE_RULE),
    vn("mitre", "miter", American, RE_RULE),
    vn("ochre", "ocher", American, RE_RULE),
    vn("sabre", "saber", American, RE_RULE),
    vn("sceptre", "scepter", American, RE_RULE),
    vn("sombre", "somber", American, RE_RULE),
    vn("spectre", "specter", American, RE_RULE),
    vn("theatre", "theater", American, RE_RULE),
    vs("metre", "meter", American, "A metre is the unit of length; a meter is a device that measures. Canadian English keeps both words, spelled differently."),
    vf("kilometre", "kilometer", American, RE_RULE),
    vf("centimetre", "centimeter", American, RE_RULE),
    vf("millimetre", "millimeter", American, RE_RULE),

    // -ce nouns beside -se verbs.
    vf("defence", "defense", American, "The adjective is defensive on both sides of the border."),
    vf("offence", "offense", American, "The adjective is offensive on both sides of the border."),
    vf("pretence", "pretense", American, no_note()),
    vs("licence", "license", American, "Canadian English spells the noun licence and the verb license: a driver's licence, but licensed to drive."),
    vs("practise", "practice", American, "Canadian English spells the verb practise and the noun practice: practise your slapshot, but hockey practice. American English uses practice for both."),

    // Doubled final l before a suffix.
    vn("cancelled", "canceled", American, DOUBLE_RULE),
    vn("cancelling", "canceling", American, DOUBLE_RULE),
    vn("counselled", "counseled", American, DOUBLE_RULE),
    vn("counsellor", "counselor", American, DOUBLE_RULE),
    vn("fuelled", "fueled", American, DOUBLE_RULE),
    vn("jeweller", "jeweler", American, DOUBLE_RULE),
    vn("jewellery", "jewelry", American, DOUBLE_RULE),
    vn("labelled", "labeled", American, DOUBLE_RULE),
    vn("labelling", "labeling", American, DOUBLE_RULE),
    vn("levelled", "leveled", American, DOUBLE_RULE),
    vn("marvellous", "marvelous", American, DOUBLE_RULE),
    vn("modelled", "modeled", American, DOUBLE_RULE),
    vn("quarrelled", "quarreled", American, DOUBLE_RULE),
    vn("signalled", "signaled", American, DOUBLE_RULE),
    vn("totalled", "totaled", American, DOUBLE_RULE),
    vn("travelled", "traveled", American, DOUBLE_RULE),
    vn("traveller", "traveler", American, DOUBLE_RULE),
    vn("travelling", "traveling", American, DOUBLE_RULE),
    vs("enrolment", "enrollment", American, "Canadian style guides prefer enrolment with one l, though enrollment is widely seen."),
    vs("fulfil", "fulfill", American, "Canadian usage is divided; fulfil follows British practice and fulfill American. Both are accepted in Canada."),
    vs("skilful", "skillful", American, "Canadian usage is divided, with skilful the traditional form."),
    vs("instalment", "installment", American, "Canadian style guides prefer instalment, though installment is common."),

    // -ize and -yze, shared with American English.
    vn("analyze", "analyse", British, IZE_RULE),
    vn("apologize", "apologise", British, IZE_RULE),
    vn("catalyze", "catalyse", British, IZE_RULE),
    vn("criticize", "criticise", British, IZE_RULE),
    vn("emphasize", "emphasise", British, IZE_RULE),
    vn("memorize", "memorise", British, IZE_RULE),
    vn("organize", "organise", British, IZE_RULE),
    vn("paralyze", "paralyse", British, IZE_RULE),
    vn("realize", "realise", British, IZE_RULE),
    vn("recognize", "recognise", British, IZE_RULE),

    // British forms Canadian keeps, where no general rule covers them.
    vs("cheque", "check", American, "A cheque is the bank instrument; check covers every other sense, including the tick mark and the verb."),
    vs("storey", "story", American, "A storey is a floor of a building, plural storeys; a story is a narrative. American English uses story for both."),
    vs("grey", "gray", American, "Both are current in Canada, with grey the more common."),
    vn("catalogue", "catalog", American, no_note()),
    vn("dialogue", "dialog", American, "Canadian English writes dialogue, except in computing, where dialog box is standard."),
    vn("moustache", "mustache", American, no_note()),
    vn("pyjamas", "pajamas", American, no_note()),
    vn("smoulder", "smolder", American, no_note()),
    vs("mould", "mold", American, "Canadian usage is divided; mould is traditional and mold is common, especially in industry."),
    vs("axe", "ax", American, no_note()),
    vs("doughnut", "donut", American, "Doughnut is the formal spelling; donut is very common in Canada, not least on signs."),
    vs("sulphur", "sulfur", American, "Canadian scientific writing now follows the international standard sulfur; sulphur remains common in general use."),

    // American forms Canadian takes.
    vf("tire", "tyre", British, "Canadian English spells the wheel covering tire, as American English does."),
    vf("curb", "kerb", British, "Canadian English uses curb for the edge of a road as well as for the verb."),
    vf("aluminum", "aluminium", British, "Canadian English says and spells aluminum, following American usage."),
    vf("jail", "gaol", British, no_note()),
    vf("program", "programme", British, "Canadian English writes program in nearly every sense, including broadcasting."),
    vf("specialty", "speciality", British, no_note()),
    vf("airplane", "aeroplane", British, no_note()),
    vf("skeptic", "sceptic", British, "Canadian English follows American usage here, against its usual British leanings."),
    vf("cozy", "cosy", British, no_note()),
    vs("plow", "plough", British, "Canadian English generally writes plow, as in snowplow, though plough appears in older and literary writing."),
];

/// Lets `vf` stay a const fn while some entries have no extra note.
const fn no_note() -> &'static str {
    ""
}

pub struct SpellingMatch {
    pub variant: &'static Variant,
    /// The Canadian form of the word as queried, inflection included.
    pub canadian_form: String,
    /// The non-Canadian form of the word as queried.
    pub other_form: String,
    /// Whether the query was already the Canadian spelling.
    pub queried_canadian: bool,
}

impl SpellingMatch {
    pub fn note(&self) -> Option<&'static str> {
        self.variant.note.filter(|n| !n.is_empty())
    }
}

/// Suffixes that may be derived from a base form.
const SUFFIXES: &[&str] = &["s", "es", "ed", "d", "ing", "er", "ers", "ly"];

/// Append a suffix, dropping a silent final `e` before a vowel.
fn inflect(base: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return base.to_string();
    }
    let starts_with_vowel = suffix.starts_with(['a', 'e', 'i', 'o', 'u']);
    if starts_with_vowel && base.ends_with('e') {
        format!("{}{}", &base[..base.len() - 1], suffix)
    } else {
        format!("{base}{suffix}")
    }
}

/// Find the spelling advice for a word, matching base and derived forms.
pub fn lookup_spelling(word: &str) -> Option<SpellingMatch> {
    let w = word.trim().to_ascii_lowercase();

    for variant in VARIANTS {
        let suffixes: &[&str] = if variant.inflects { SUFFIXES } else { &[] };

        if w == variant.canadian {
            return Some(built(variant, "", true));
        }
        if w == variant.other {
            return Some(built(variant, "", false));
        }
        for suffix in suffixes {
            if w == inflect(variant.canadian, suffix) {
                return Some(built(variant, suffix, true));
            }
            if w == inflect(variant.other, suffix) {
                return Some(built(variant, suffix, false));
            }
        }
    }
    None
}

fn built(variant: &'static Variant, suffix: &str, queried_canadian: bool) -> SpellingMatch {
    SpellingMatch {
        variant,
        canadian_form: inflect(variant.canadian, suffix),
        other_form: inflect(variant.other, suffix),
        queried_canadian,
        }
}

/// A word or sense that is distinctively Canadian.
pub struct Canadianism {
    pub word: &'static str,
    pub gloss: &'static str,
    /// Where in Canada the word is used, when it is not nationwide.
    pub region: Option<&'static str>,
}

const fn c(word: &'static str, gloss: &'static str) -> Canadianism {
    Canadianism { word, gloss, region: None }
}

const fn cr(word: &'static str, gloss: &'static str, region: &'static str) -> Canadianism {
    Canadianism { word, gloss, region: Some(region) }
}

pub const CANADIANISMS: &[Canadianism] = &[
    c("allophone", "A Canadian whose first language is neither English nor French."),
    c("anglophone", "A person whose first language is English, used especially in contrast with francophone."),
    c("bachelor", "A one-room apartment, the Canadian equivalent of a studio."),
    cr("bunny hug", "A hooded sweatshirt.", "Saskatchewan"),
    c("butter tart", "A small pastry tart filled with butter, sugar and egg, sometimes with raisins or pecans."),
    c("Caesar", "A cocktail of vodka, Clamato, hot sauce and Worcestershire, served with a celery stick."),
    c("Canuck", "A Canadian. Mildly informal, and not usually an insult in Canada."),
    c("chesterfield", "A sofa. Once the ordinary Canadian word, now old-fashioned; couch has largely replaced it."),
    cr("chinook", "A warm dry wind that descends the eastern slopes of the Rockies and can raise winter temperatures sharply.", "Alberta and southern British Columbia"),
    c("concession road", "A rural road following the survey grid of a township."),
    cr("dep", "A corner store, from the French dépanneur.", "Quebec"),
    c("double-double", "A coffee with two creams and two sugars."),
    c("duotang", "A card-stock folder with metal fasteners, used by schoolchildren."),
    c("eavestrough", "A roof gutter."),
    c("eh", "A tag appended to a statement to invite agreement or check that the listener is following, as in \u{201c}Cold out, eh?\u{201d}"),
    c("francophone", "A person whose first language is French."),
    c("garburator", "An electric garbage disposal unit fitted in a kitchen sink."),
    c("give'r", "To go at something with full effort."),
    c("gong show", "A chaotic, badly run situation."),
    c("homo milk", "Homogenized whole milk, 3.25% butterfat."),
    c("housecoat", "A dressing gown or bathrobe."),
    c("hydro", "Household electricity, and the utility that supplies it: the hydro bill, hydro lines."),
    c("keener", "Someone conspicuously eager to please a teacher or boss."),
    c("klick", "A kilometre."),
    c("loonie", "The Canadian one-dollar coin, named for the loon on its reverse."),
    c("mickey", "A 375 mL bottle of spirits, shaped to fit a pocket."),
    c("Mountie", "A member of the Royal Canadian Mounted Police."),
    c("muskeg", "Northern bog: waterlogged ground thick with moss and peat."),
    c("Nanaimo bar", "A no-bake layered dessert bar of crumb base, custard icing and chocolate, named for Nanaimo, British Columbia."),
    c("parkade", "A multi-storey parking garage."),
    c("pencil crayon", "A coloured pencil."),
    c("pogey", "Employment insurance benefits."),
    c("pop", "A sweet carbonated soft drink. The usual Canadian word, where Britain says fizzy drink and parts of the United States say soda."),
    c("portage", "To carry a canoe and gear overland between waterways; the route so travelled."),
    c("poutine", "French fries topped with cheese curds and gravy, originally from Quebec."),
    c("riding", "A federal or provincial electoral district."),
    c("runners", "Running shoes; sneakers."),
    cr("screech", "A strong dark rum long associated with the province.", "Newfoundland and Labrador"),
    c("serviette", "A paper table napkin."),
    cr("skookum", "Impressive, powerful or excellent, from Chinook Jargon.", "British Columbia"),
    c("snowbird", "A Canadian who spends the winter in the southern United States."),
    cr("sook", "A person who whines or sulks; also sooky as an adjective.", "Atlantic Canada"),
    c("stagette", "A party for a bride before her wedding; a bachelorette party."),
    c("toonie", "The Canadian two-dollar coin, named after the loonie."),
    c("toque", "A close-fitting knitted winter hat, the central garment of Canadian winter. Also spelled tuque."),
    c("tuque", "A close-fitting knitted winter hat. Also spelled toque."),
    c("two-four", "A case of twenty-four bottles or cans of beer."),
    c("washroom", "A room with a toilet, public or domestic. The ordinary Canadian word where Britain says loo and the United States says restroom or bathroom."),
    c("whitener", "Non-dairy creamer for coffee."),
    c("zed", "The name of the letter Z. Canadians say zed, not the American zee."),
];

/// Find a Canadianism, tolerating case and a plural `-s`.
pub fn lookup_canadianism(word: &str) -> Option<&'static Canadianism> {
    let w = word.trim().to_ascii_lowercase();
    CANADIANISMS
        .iter()
        .find(|c| c.word.to_ascii_lowercase() == w)
        .or_else(|| {
            let singular = w.strip_suffix('s')?;
            CANADIANISMS
                .iter()
                .find(|c| c.word.to_ascii_lowercase() == singular)
        })
}

// ---------------------------------------------------------------------------
// Canadianizing fetched text
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::OnceLock;

/// Map of American spellings to their Canadian equivalents, inflections
/// included.
///
/// Only rule-governed pairs take part. Pairs where the two spellings are
/// separate words in Canadian English — `story` and `storey`, `check` and
/// `cheque`, `practice` and `practise` — are marked `inflects: false` and
/// excluded, because choosing between them needs the sense, not the spelling.
fn substitutions() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for variant in VARIANTS {
            if variant.contrast != Contrast::American || !variant.inflects || !variant.redirect {
                continue;
            }
            for suffix in std::iter::once(&"").chain(SUFFIXES.iter()) {
                let from = inflect(variant.other, suffix);
                let to = inflect(variant.canadian, suffix);
                if from != to {
                    map.insert(from, to);
                }
            }
        }
        map
    })
}

/// Rewrite unambiguous American spellings in Wiktionary text to their Canadian
/// forms, preserving capitalization.
pub fn canadianize(text: &str) -> String {
    let map = substitutions();
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();

    let flush = |word: &mut String, out: &mut String| {
        if word.is_empty() {
            return;
        }
        match map.get(&word.to_ascii_lowercase()) {
            Some(replacement) => out.push_str(&match_case(word, replacement)),
            None => out.push_str(word),
        }
        word.clear();
    };

    for c in text.chars() {
        if c.is_alphabetic() {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Give `replacement` the capitalization pattern of `original`.
fn match_case(original: &str, replacement: &str) -> String {
    let uppercase_count = original.chars().filter(|c| c.is_uppercase()).count();

    if uppercase_count == 0 {
        replacement.to_string()
    } else if uppercase_count == original.chars().count() {
        replacement.to_uppercase()
    } else if original.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = replacement.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    } else {
        replacement.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_both_sides_of_a_pair() {
        let m = lookup_spelling("colour").expect("colour");
        assert!(m.queried_canadian);
        assert_eq!(m.other_form, "color");

        let m = lookup_spelling("color").expect("color");
        assert!(!m.queried_canadian);
        assert_eq!(m.canadian_form, "colour");
        assert_eq!(m.variant.contrast, Contrast::American);
    }

    #[test]
    fn records_which_variety_canada_agrees_with() {
        // Canada follows Britain on -our, and America on -ize.
        assert_eq!(lookup_spelling("labour").unwrap().variant.contrast, Contrast::American);
        assert_eq!(lookup_spelling("organize").unwrap().variant.contrast, Contrast::British);
    }

    #[test]
    fn matches_derived_forms() {
        let m = lookup_spelling("neighbours").expect("neighbours");
        assert_eq!(m.canadian_form, "neighbours");
        assert_eq!(m.other_form, "neighbors");

        let m = lookup_spelling("favored").expect("favored");
        assert!(!m.queried_canadian);
        assert_eq!(m.canadian_form, "favoured");

        // A final silent e is dropped before a vowel: centre + ed -> centred.
        let m = lookup_spelling("centred").expect("centred");
        assert_eq!(m.other_form, "centered");
    }

    #[test]
    fn does_not_derive_forms_that_collide_with_other_words() {
        // `tired` is not the past tense of the wheel covering.
        assert!(lookup_spelling("tired").is_none());
        // `storeys` is fine, but `stories` must not be claimed as American.
        assert!(lookup_spelling("stories").is_none());
    }

    #[test]
    fn semantic_pairs_are_reported_but_never_substituted() {
        for word in ["practice", "licence", "cheque", "storey", "metre", "grey"] {
            let m = lookup_spelling(word).unwrap_or_else(|| panic!("{word} should be listed"));
            assert!(!m.variant.redirect, "{word} must not redirect");
        }
    }

    #[test]
    fn practise_is_recorded_as_the_verb() {
        let m = lookup_spelling("practise").expect("practise");
        assert_eq!(m.variant.canadian, "practise");
        assert_eq!(m.variant.other, "practice");
        assert!(m.note().unwrap().contains("verb practise"));
    }

    #[test]
    fn canadianizes_unambiguous_spellings_only() {
        assert_eq!(canadianize("the color of the center"), "the colour of the centre");
        assert_eq!(canadianize("he traveled and labeled it"), "he travelled and labelled it");
        // Ambiguous pairs are left alone: a story is not a storey, a meter is
        // not a metre, and gray is current in Canada.
        assert_eq!(canadianize("a story about a gray meter"), "a story about a gray meter");
        assert_eq!(canadianize("cash a check"), "cash a check");
    }

    #[test]
    fn canadianize_preserves_capitalization_and_punctuation() {
        assert_eq!(canadianize("Color, COLOR and color."), "Colour, COLOUR and colour.");
        assert_eq!(canadianize("discoloration"), "discoloration");
    }

    #[test]
    fn finds_canadianisms_including_plurals() {
        assert!(lookup_canadianism("toque").is_some());
        assert!(lookup_canadianism("Toque").is_some());
        assert_eq!(lookup_canadianism("toonies").unwrap().word, "toonie");
        assert!(lookup_canadianism("sandwich").is_none());
    }

    #[test]
    fn regional_canadianisms_say_where() {
        assert_eq!(lookup_canadianism("bunny hug").unwrap().region, Some("Saskatchewan"));
        assert_eq!(lookup_canadianism("toque").unwrap().region, None);
    }

    /// The spelling table is hand-written, so guard it against the mistakes
    /// that are easy to make when editing it.
    #[test]
    fn the_spelling_table_is_consistent() {
        let mut seen: Vec<&str> = Vec::new();

        for v in VARIANTS {
            assert_ne!(v.canadian, v.other, "{} is listed against itself", v.canadian);
            assert!(!v.canadian.is_empty() && !v.other.is_empty());
            assert!(
                v.canadian.chars().all(|c| c.is_ascii_lowercase() || c == ' '),
                "{} should be lowercase",
                v.canadian
            );

            for form in [v.canadian, v.other] {
                assert!(
                    !seen.contains(&form),
                    "{form} appears twice in the spelling table"
                );
                seen.push(form);
            }
        }
    }

    /// Every substitution must be reversible: applying it to the Canadian form
    /// must be a no-op, or the pass would corrupt text it has already fixed.
    #[test]
    fn canadianizing_is_idempotent() {
        for v in VARIANTS {
            let once = canadianize(v.canadian);
            assert_eq!(once, canadianize(&once), "{} is not stable", v.canadian);
            assert_eq!(once, v.canadian, "{} was rewritten", v.canadian);
        }
    }

    #[test]
    fn the_glossary_is_consistent() {
        let mut seen: Vec<String> = Vec::new();
        for c in CANADIANISMS {
            let key = c.word.to_ascii_lowercase();
            assert!(!seen.contains(&key), "{} appears twice in the glossary", c.word);
            assert!(
                c.gloss.ends_with(['.', '?', '!', '\u{201d}']),
                "the gloss for {} should end in terminal punctuation",
                c.word
            );
            seen.push(key);
        }
    }
}
