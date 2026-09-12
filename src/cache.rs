//! An on-disk cache of raw Wiktionary wikitext.
//!
//! Caching the upstream wikitext rather than a parsed entry means a word looked
//! up once stays available offline, and that improvements to the parser apply
//! to everything already cached.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("dict")
}

/// Map a word to a filename, escaping anything that is not a plain ASCII
/// letter, digit, hyphen or apostrophe so that entries like `a/b` or `café`
/// cannot escape the cache directory.
fn file_name(word: &str) -> String {
    let mut name = String::with_capacity(word.len() + 8);
    for c in word.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            name.push(c.to_ascii_lowercase());
        } else {
            for b in c.to_string().as_bytes() {
                name.push_str(&format!("%{b:02x}"));
            }
        }
    }
    name.push_str(".wiki");
    name
}

fn path(word: &str) -> PathBuf {
    dir().join(file_name(word))
}

pub fn read(word: &str) -> Option<String> {
    fs::read_to_string(path(word)).ok().filter(|s| !s.trim().is_empty())
}

pub fn write(word: &str, wikitext: &str) -> io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    // Write to a temporary file first so an interrupted run cannot leave a
    // truncated entry behind.
    let target = dir.join(file_name(word));
    let temp = target.with_extension("wiki.tmp");
    fs::write(&temp, wikitext)?;
    fs::rename(&temp, &target)
}

/// Remove every cached entry, returning how many were deleted.
pub fn clear() -> io::Result<usize> {
    let dir = dir();
    if !dir.exists() {
        return Ok(0);
    }
    let mut removed = 0;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if is_cache_file(&entry.path()) {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn is_cache_file(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".wiki") || n.ends_with(".wiki.tmp"))
}

/// How many entries are cached.
pub fn count() -> usize {
    fs::read_dir(dir())
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.ends_with(".wiki"))
                })
                .count()
        })
        .unwrap_or(0)
}
