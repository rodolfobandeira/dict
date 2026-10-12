//! Fetching entries from Wiktionary, with the cache in front of it.

use std::fmt;
use std::time::Duration;

use crate::cache;

const USER_AGENT: &str =
    concat!("dict/", env!("CARGO_PKG_VERSION"), " (Canadian English dictionary CLI)");
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub enum FetchError {
    /// Wiktionary has no page for this word.
    NotFound,
    /// The network was unreachable or the request failed.
    Network(String),
    /// `--offline` was given and the word is not cached.
    NotCached,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::NotFound => write!(f, "no Wiktionary entry for this word"),
            FetchError::Network(e) => write!(f, "could not reach Wiktionary: {e}"),
            FetchError::NotCached => {
                write!(f, "not in the offline cache, and --offline was given")
            }
        }
    }
}

pub struct Fetched {
    pub wikitext: String,
    pub from_cache: bool,
}

/// Get the wikitext for a word, preferring the cache unless told otherwise.
pub fn wikitext(word: &str, offline: bool, refresh: bool) -> Result<Fetched, FetchError> {
    if !refresh {
        if let Some(cached) = cache::read(word) {
            // Entries cached by older versions were stored unsanitized.
            return Ok(Fetched { wikitext: sanitize(&cached), from_cache: true });
        }
    }
    if offline {
        return Err(FetchError::NotCached);
    }

    let text = sanitize(&download(word)?);
    // A failed cache write should not fail the lookup.
    let _ = cache::write(word, &text);
    Ok(Fetched { wikitext: text, from_cache: false })
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

fn download(word: &str) -> Result<String, FetchError> {
    let url = format!(
        "https://en.wiktionary.org/w/index.php?title={}&action=raw",
        encode(word)
    );

    match agent().get(&url).call() {
        Ok(mut response) => response
            .body_mut()
            .read_to_string()
            .map_err(|e| FetchError::Network(e.to_string())),
        Err(ureq::Error::StatusCode(404)) => Err(FetchError::NotFound),
        Err(ureq::Error::StatusCode(code)) => {
            Err(FetchError::Network(format!("Wiktionary returned HTTP {code}")))
        }
        Err(e) => Err(FetchError::Network(e.to_string())),
    }
}

/// Ask Wiktionary for titles close to a word the user mistyped.
pub fn suggestions(word: &str) -> Vec<String> {
    let url = format!(
        "https://en.wiktionary.org/w/api.php?action=opensearch&search={}&limit=8&namespace=0&format=json",
        encode(word)
    );

    let Ok(mut response) = agent().get(&url).call() else {
        return Vec::new();
    };
    let Ok(body) = response.body_mut().read_to_string() else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) else {
        return Vec::new();
    };

    // OpenSearch replies with [query, [titles], [descriptions], [urls]].
    json.get(1)
        .and_then(|v| v.as_array())
        .map(|titles| {
            titles
                .iter()
                .filter_map(|t| t.as_str())
                .filter(|t| !t.eq_ignore_ascii_case(word))
                .map(sanitize)
                .collect()
        })
        .unwrap_or_default()
}

/// Remove characters that a terminal would act on rather than print.
///
/// Wiktionary is editable by anyone, so its text must not be able to smuggle
/// escape sequences (colours, cursor movement, OSC title or hyperlink codes)
/// or bidirectional overrides onto the reader's terminal. This also keeps
/// upstream text from forging the emphasis markers `markup` inserts later.
/// Newlines and tabs are kept; the parser needs them.
pub fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|&c| c == '\n' || c == '\t' || !(c.is_control() || is_bidi_control(c)))
        .collect()
}

fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Percent-encode a page title for use in a query string.
fn encode(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for byte in word.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('_'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn sanitize_strips_terminal_escapes() {
        let hostile = "a\u{1b}[2J\u{1b}]0;pwned\u{7}b\u{9b}31mc\u{1}d\u{202E}e\rf";
        assert_eq!(sanitize(hostile), "a[2J]0;pwnedb31mcdef");
    }

    #[test]
    fn sanitize_keeps_ordinary_text() {
        let text = "===Noun===\n# A {{lb|en|Canada}} hat.\tcafé /tuːk/ 🍁";
        assert_eq!(sanitize(text), text);
    }
}
