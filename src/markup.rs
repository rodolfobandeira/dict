//! Turns Wiktionary's wikitext into readable plain text.
//!
//! Wiktionary definitions are wikitext, not prose: links, quote marks and a
//! long tail of templates. This module flattens the constructs that actually
//! appear inside definition lines and drops the rest, rather than trying to be
//! a general MediaWiki parser.

/// Emphasis is preserved through these private-use markers so the terminal
/// renderer can bold it and the JSON writer can strip it.
pub const EMPH_START: char = '\u{1}';
pub const EMPH_END: char = '\u{2}';

/// Render wikitext to text carrying emphasis markers.
pub fn render(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if starts_with(&chars, i, "{{") {
            let (body, next) = take_balanced(&chars, i, "{{", "}}");
            out.push_str(&render_template(&body));
            i = next;
        } else if starts_with(&chars, i, "[[") {
            let (body, next) = take_balanced(&chars, i, "[[", "]]");
            out.push_str(&render_link(&body));
            i = next;
        } else if starts_with(&chars, i, "'''") {
            // Toggle emphasis; unbalanced markers simply cancel at end of line.
            out.push(if count_marker(&out) % 2 == 0 { EMPH_START } else { EMPH_END });
            i += 3;
        } else if starts_with(&chars, i, "''") {
            i += 2;
        } else if chars[i] == '<' {
            i = skip_html(&chars, i, &mut out);
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    tidy(&out)
}

/// Render wikitext to plain text with no emphasis markers.
pub fn render_plain(input: &str) -> String {
    strip_emphasis(&render(input))
}

pub fn strip_emphasis(s: &str) -> String {
    s.chars().filter(|c| *c != EMPH_START && *c != EMPH_END).collect()
}

fn count_marker(s: &str) -> usize {
    s.chars().filter(|c| *c == EMPH_START || *c == EMPH_END).count()
}

fn starts_with(chars: &[char], i: usize, pat: &str) -> bool {
    pat.chars().enumerate().all(|(k, c)| chars.get(i + k) == Some(&c))
}

/// Consume a `{{...}}` or `[[...]]` run, respecting nesting of both kinds, and
/// return its inner text plus the index just past the closing delimiter.
fn take_balanced(chars: &[char], start: usize, open: &str, close: &str) -> (String, usize) {
    let mut depth = 0usize;
    let mut i = start;
    let mut body = String::new();

    while i < chars.len() {
        if starts_with(chars, i, open) {
            depth += 1;
            if depth > 1 {
                body.push_str(open);
            }
            i += open.len();
        } else if starts_with(chars, i, close) {
            depth -= 1;
            i += close.len();
            if depth == 0 {
                return (body, i);
            }
            body.push_str(close);
        } else {
            body.push(chars[i]);
            i += 1;
        }
    }
    // Unterminated: treat the remainder as the body.
    (body, chars.len())
}

/// `[[target|display]]` keeps the display text; `[[target]]` keeps the target.
fn render_link(body: &str) -> String {
    let parts = split_top_level(body);
    let text = parts.last().map(String::as_str).unwrap_or("");
    // Strip a namespace prefix such as `w:` or `Appendix:Glossary#foo`.
    let text = text.split('#').next().unwrap_or(text);
    render(text.trim())
}

/// Drop HTML. `<ref>` and comments take their contents with them; other tags
/// are removed but leave their contents in place.
fn skip_html(chars: &[char], i: usize, out: &mut String) -> usize {
    if starts_with(chars, i, "<!--") {
        return find_after(chars, i, "-->").unwrap_or(chars.len());
    }
    if starts_with(chars, i, "<ref") {
        if let Some(end) = find_after(chars, i, "</ref>") {
            return end;
        }
        // A self-closing `<ref .../>`.
        return find_after(chars, i, ">").unwrap_or(chars.len());
    }
    if starts_with(chars, i, "<br") {
        out.push(' ');
        return find_after(chars, i, ">").unwrap_or(chars.len());
    }
    match find_after(chars, i, ">") {
        // Only treat it as a tag if it looks like one, so `a < b` survives.
        Some(end) if looks_like_tag(chars, i, end) => end,
        _ => {
            out.push('<');
            i + 1
        }
    }
}

fn looks_like_tag(chars: &[char], start: usize, end: usize) -> bool {
    let inner: String = chars[start + 1..end.saturating_sub(1)].iter().collect();
    let name = inner.trim_start_matches('/');
    end - start <= 64
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
}

fn find_after(chars: &[char], from: usize, pat: &str) -> Option<usize> {
    (from..chars.len()).find(|&i| starts_with(chars, i, pat)).map(|i| i + pat.chars().count())
}

/// Split on `|` that is not inside a nested template or link.
pub fn split_top_level(body: &str) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    let mut parts = vec![String::new()];
    let mut depth = 0usize;
    let mut i = 0;

    while i < chars.len() {
        if starts_with(&chars, i, "{{") || starts_with(&chars, i, "[[") {
            depth += 1;
            parts.last_mut().unwrap().push(chars[i]);
            parts.last_mut().unwrap().push(chars[i + 1]);
            i += 2;
        } else if starts_with(&chars, i, "}}") || starts_with(&chars, i, "]]") {
            depth = depth.saturating_sub(1);
            parts.last_mut().unwrap().push(chars[i]);
            parts.last_mut().unwrap().push(chars[i + 1]);
            i += 2;
        } else if chars[i] == '|' && depth == 0 {
            parts.push(String::new());
            i += 1;
        } else {
            parts.last_mut().unwrap().push(chars[i]);
            i += 1;
        }
    }
    parts
}

/// A template split into its name, positional arguments and named arguments.
pub struct Template {
    pub name: String,
    pub positional: Vec<String>,
    pub named: Vec<(String, String)>,
}

impl Template {
    pub fn parse(body: &str) -> Template {
        let mut parts = split_top_level(body).into_iter();
        let name = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
        let mut positional = Vec::new();
        let mut named = Vec::new();

        for part in parts {
            match split_named(&part) {
                Some((k, v)) => named.push((k, v)),
                None => positional.push(part.trim().to_string()),
            }
        }
        Template { name, positional, named }
    }

    pub fn named(&self, key: &str) -> Option<&str> {
        self.named.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn positional(&self, index: usize) -> &str {
        self.positional.get(index).map(String::as_str).unwrap_or("")
    }

    /// Positional arguments with a leading language code removed.
    pub fn args_after_lang(&self) -> &[String] {
        if self.positional.first().is_some_and(|a| is_lang_code(a)) {
            &self.positional[1..]
        } else {
            &self.positional
        }
    }
}

/// `a=Canada` is a named argument; `a = b` inside prose is not, so the key must
/// look like an identifier.
fn split_named(part: &str) -> Option<(String, String)> {
    let eq = part.find('=')?;
    let key = part[..eq].trim();
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some((key.to_ascii_lowercase(), part[eq + 1..].trim().to_string()))
}

fn is_lang_code(arg: &str) -> bool {
    let a = arg.trim();
    (2..=3).contains(&a.chars().count())
        && a.chars().all(|c| c.is_ascii_lowercase())
}

/// Whether a template name belongs to the form-of family, meaning the sense
/// only points at another word rather than defining anything.
pub fn is_form_of_template(name: &str) -> bool {
    form_of_label(name).is_some()
}

/// Human wording for the `... of` family of form-of templates.
fn form_of_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "plural of" => "plural of",
        "past of" | "simple past of" => "past tense of",
        "past participle of" => "past participle of",
        "present participle of" => "present participle of",
        "gerund of" => "gerund of",
        "comparative of" => "comparative of",
        "superlative of" => "superlative of",
        "alternative form of" | "alt form" | "altform" => "alternative form of",
        "alternative spelling of" | "alt sp" | "altsp" => "alternative spelling of",
        "standard spelling of" | "stand sp" => "standard spelling of",
        "synonym of" | "syn of" => "synonym of",
        "abbreviation of" => "abbreviation of",
        "initialism of" => "initialism of",
        "acronym of" => "acronym of",
        "clipping of" => "clipping of",
        "ellipsis of" => "ellipsis of",
        "short for" => "short for",
        "obsolete form of" => "obsolete form of",
        "obsolete spelling of" => "obsolete spelling of",
        "archaic form of" => "archaic form of",
        "misspelling of" => "misspelling of",
        "eye dialect of" => "eye dialect spelling of",
        "nominalization of" => "nominalization of",
        "inflection of" | "infl of" => "inflection of",
        _ => return None,
    })
}

/// Grammar tags used by `{{inflection of}}`, spelled out.
fn grammar_tag(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "ing-form" | "ger" => "present participle",
        "pres" => "present",
        "past" => "past tense",
        "part" | "ptcp" => "participle",
        "pp" | "past|part" => "past participle",
        "s" | "sg" => "singular",
        "p" | "pl" => "plural",
        "1" => "first-person",
        "2" => "second-person",
        "3" => "third-person",
        "comd" => "comparative",
        "supd" => "superlative",
        _ => return None,
    })
}

fn render_template(body: &str) -> String {
    let t = Template::parse(body);
    let args = t.args_after_lang();
    let arg = |i: usize| -> String { args.get(i).map(|s| render(s)).unwrap_or_default() };

    if let Some(label) = form_of_label(&t.name) {
        // `{{infl of|en|compel||ing-form}}`: target, then optional grammar tags.
        let target = arg(0);
        let tags: Vec<&'static str> = args
            .iter()
            .skip(1)
            .filter_map(|a| grammar_tag(a.trim()))
            .collect();
        let target = t.named("t").map(render).unwrap_or(target);
        return if tags.is_empty() {
            format!("{label} {EMPH_START}{target}{EMPH_END}")
        } else {
            format!("{} of {EMPH_START}{target}{EMPH_END}", tags.join(" and "))
        };
    }

    match t.name.as_str() {
        // Plain links to other entries.
        "l" | "m" | "ll" | "link" | "mention" => {
            let display = arg(1);
            if display.is_empty() { arg(0) } else { display }
        }
        // `{{w|Page|Display}}` is an inline link to Wikipedia.
        "w" => {
            let display = render(t.positional(1));
            if display.is_empty() { render(t.positional(0)) } else { display }
        }
        // Sense anchors, sidebar boxes, category markers and editor requests
        // are invisible on Wiktionary, so they must not leak into a definition.
        // `{{senseid|en|cheese}}` would otherwise render as a stray "cheese".
        "senseid" | "anchor" | "wikipedia" | "pedia" | "wp" | "slim-wikipedia"
        | "c" | "topics" | "catlangname" | "cln" | "categorize" | "examples"
        | "rfex" | "rfd" | "rfv" | "rfdef" | "rfquote" | "rfc" | "rfclarify"
        | "attention" | "attn" | "tea room" => String::new(),
        "taxlink" | "vern" | "taxfmt" => render(t.positional(0)),
        // Parenthesised glosses and qualifiers.
        "gloss" | "gl" => format!("({})", arg(0)),
        "q" | "qual" | "qualifier" | "i" | "sense" => {
            let joined: Vec<String> = args.iter().map(|a| render(a)).collect();
            format!("({})", joined.join(", "))
        }
        // Non-gloss definitions are already prose.
        "n-g" | "ng" | "non-gloss" | "non-gloss definition" | "ngd" => arg(0),
        "defdate" | "defdt" => String::new(),
        "unsupported" => arg(0),
        // `{{,}}` and friends are punctuation helpers.
        "," => ",".to_string(),
        "..." => "…".to_string(),
        "nbsp" => " ".to_string(),
        _ => {
            // An unknown template with a single meaningful argument is most
            // likely a wrapper around a word; anything else is metadata.
            if args.len() == 1 && !args[0].is_empty() {
                render(&args[0])
            } else {
                String::new()
            }
        }
    }
}

/// Collapse the whitespace and stray punctuation left behind by dropped
/// templates.
fn tidy(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !last_space && !out.is_empty() {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(c);
            last_space = false;
        }
    }

    let out = out.trim().to_string();
    // A dropped leading template can leave ` , foo` or `; foo`.
    let out = out.trim_start_matches([',', ';', ':']).trim().to_string();
    out.replace(" ,", ",").replace(" .", ".").replace(" ;", ";").replace("( ", "(").replace(" )", ")")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(s: &str) -> String {
        render_plain(s)
    }

    #[test]
    fn keeps_link_display_text() {
        assert_eq!(plain("a [[hat]] with no [[brim]]"), "a hat with no brim");
        assert_eq!(plain("[[wear|worn]] by [[chef]]s"), "worn by chefs");
    }

    #[test]
    fn drops_namespace_and_anchor_from_links() {
        assert_eq!(plain("[[Appendix:Glossary#participle|participle]]"), "participle");
        assert_eq!(plain("[[compel#English]]"), "compel");
    }

    #[test]
    fn renders_nested_templates() {
        assert_eq!(plain("{{l|en|{{l|en|deep}}}}"), "deep");
    }

    #[test]
    fn renders_form_of_templates() {
        assert_eq!(plain("{{plural of|en|toque}}"), "plural of toque");
        assert_eq!(plain("{{infl of|en|compel||ing-form}}"), "present participle of compel");
        assert_eq!(plain("{{standard spelling of|en|from=Commonwealth|color}}"), "standard spelling of color");
    }

    #[test]
    fn parenthesises_glosses_and_qualifiers() {
        assert_eq!(plain("{{gloss|a kind of hat}}"), "(a kind of hat)");
        assert_eq!(plain("{{q|middle portion}} the centre"), "(middle portion) the centre");
    }

    #[test]
    fn drops_metadata_templates() {
        assert_eq!(plain("a word {{defdate|from 1590}}"), "a word");
        assert_eq!(plain("{{rfd|en}}text"), "text");
    }

    #[test]
    fn drops_invisible_anchors_and_category_markers() {
        // A sense anchor names the sense for linking; it is not part of it.
        assert_eq!(
            plain("{{senseid|en|cheese}} A [[dish#Noun|dish]] of fries."),
            "A dish of fries."
        );
        assert_eq!(plain("{{anchor|x}}text"), "text");
        assert_eq!(plain("{{C|en|Foods}}a food"), "a food");
        // The inline Wikipedia link still renders.
        assert_eq!(plain("{{w|Quebec}} cuisine"), "Quebec cuisine");
        assert_eq!(plain("{{w|Quebec|la belle province}}"), "la belle province");
    }

    #[test]
    fn strips_quotes_and_html() {
        assert_eq!(plain("so '''compelling''' that"), "so compelling that");
        assert_eq!(plain("''emphasis'' here"), "emphasis here");
        assert_eq!(plain("one<ref>a source</ref> two"), "one two");
        assert_eq!(plain("one<!-- hidden --> two"), "one two");
    }

    #[test]
    fn marks_emphasis_for_the_renderer() {
        let rendered = render("so '''compelling''' that");
        assert!(rendered.contains(EMPH_START) && rendered.contains(EMPH_END));
        assert_eq!(strip_emphasis(&rendered), "so compelling that");
    }

    #[test]
    fn keeps_a_bare_less_than_sign() {
        assert_eq!(plain("a < b"), "a < b");
    }

    #[test]
    fn splits_named_arguments_only_when_they_look_like_keys() {
        let t = Template::parse("IPA|en|/tuːk/|a=Canada");
        assert_eq!(t.name, "ipa");
        assert_eq!(t.args_after_lang(), ["/tuːk/"]);
        assert_eq!(t.named("a"), Some("Canada"));

        // An equals sign inside prose is not an argument name.
        let t = Template::parse("ux|en|two plus two = four");
        assert_eq!(t.args_after_lang(), ["two plus two = four"]);
    }

    #[test]
    fn survives_an_unterminated_template() {
        assert_eq!(plain("{{l|en|word"), "word");
    }
}
