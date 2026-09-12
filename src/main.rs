//! `dict` — a Canadian English dictionary for the command line.

mod cache;
mod canadian;
mod cli;
mod markup;
mod model;
mod render;
mod source;
mod wikitext;

use std::process::ExitCode;

use clap::Parser;

use cli::Args;
use model::Entry;
use render::Style;

fn main() -> ExitCode {
    let args = Args::parse();
    let style = Style::new(args.no_color || args.json);

    if args.cache_dir {
        println!("{}", cache::dir().display());
        println!("{} entries cached", cache::count());
        return ExitCode::SUCCESS;
    }

    if args.clear_cache {
        return match cache::clear() {
            Ok(n) => {
                println!("Cleared {n} cached {}.", plural(n, "entry", "entries"));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("dict: could not clear the cache: {e}");
                ExitCode::FAILURE
            }
        };
    }

    let Some(word) = args.word.as_deref().map(str::trim).filter(|w| !w.is_empty()) else {
        eprintln!("dict: no word given\n\nUsage: dict <WORD>\nTry 'dict --help' for more information.");
        return ExitCode::from(2);
    };

    run(&args, &style, word)
}

fn run(args: &Args, style: &Style, word: &str) -> ExitCode {
    // The Canadian layer is consulted first: it is built in, so it answers even
    // with no network and no cache.
    let spelling = canadian::lookup_spelling(word);
    let redirect = !args.exact
        && spelling
            .as_ref()
            .is_some_and(|m| !m.queried_canadian && m.variant.redirect);

    let target = match (&spelling, redirect) {
        (Some(m), true) => m.canadian_form.clone(),
        _ => word.to_string(),
    };
    let canadianism = canadian::lookup_canadianism(&target);

    if args.canadian {
        return print_canadian_only(style, word, spelling.as_ref(), canadianism, redirect);
    }

    let fetched = source::wikitext(&target, args.offline, args.refresh);

    let mut filed_under: Option<String> = None;

    let (entry, from_cache) = match fetched {
        Ok(f) => {
            let mut entry = wikitext::parse(&target, &f.wikitext);
            let mut from_cache = f.from_cache;

            // Wiktionary keeps many Canadian spellings as bare pointers to the
            // American form, so follow one hop to reach the real definitions.
            if let Some(pointer) = entry.spelling_pointer() {
                if !pointer.eq_ignore_ascii_case(&target) {
                    if let Ok(f2) = source::wikitext(&pointer, args.offline, args.refresh) {
                        let mut resolved = wikitext::parse(&target, &f2.wikitext);
                        if !resolved.is_empty() {
                            // The pronunciation on the Canadian spelling's own
                            // page, where there is one, is the better guide.
                            if !entry.pronunciations.is_empty() {
                                resolved.pronunciations =
                                    std::mem::take(&mut entry.pronunciations);
                            }
                            entry = resolved;
                            from_cache = from_cache && f2.from_cache;
                            filed_under = Some(pointer);
                        }
                    }
                }
            }
            // Definitions fetched from Wiktionary use whichever spellings the
            // editors chose; present them in Canadian form unless asked not to.
            if !args.verbatim && !args.exact {
                entry.canadianize();
            }
            (entry, from_cache)
        }
        Err(e) => {
            // Built-in Canadian data still has something to say about the word.
            if spelling.is_some() || canadianism.is_some() {
                if args.json {
                    print_json(style, word, &target, &Entry { word: target.clone(), ..Default::default() }, spelling.as_ref(), canadianism, redirect, false, None);
                    return ExitCode::SUCCESS;
                }
                println!();
                render::print_headword(style, &Entry { word: target.clone(), ..Default::default() });
                println!();
                print_canadian_block(style, spelling.as_ref(), canadianism, redirect);
                eprintln!("dict: no definitions available ({e})");
                return ExitCode::SUCCESS;
            }

            if matches!(e, source::FetchError::NotFound) {
                let suggestions = if args.offline { Vec::new() } else { source::suggestions(&target) };
                render::print_suggestions(style, &target, &suggestions);
            } else {
                eprintln!("dict: {e}");
            }
            return ExitCode::FAILURE;
        }
    };

    if entry.is_empty() {
        if spelling.is_some() || canadianism.is_some() {
            println!();
            render::print_headword(style, &entry);
            println!();
            print_canadian_block(style, spelling.as_ref(), canadianism, redirect);
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "dict: Wiktionary has a page for {target:?}, but no English definitions on it."
        );
        return ExitCode::FAILURE;
    }

    if args.json {
        print_json(style, word, &target, &entry, spelling.as_ref(), canadianism, redirect, from_cache, filed_under.as_deref());
        return ExitCode::SUCCESS;
    }

    println!();
    render::print_headword(style, &entry);

    // The definitions already open with a blank line, so only separate the
    // headword when there are Canadian notes in between.
    if spelling.is_some() || canadianism.is_some() || filed_under.is_some() {
        println!();
        print_canadian_block(style, spelling.as_ref(), canadianism, redirect);
        if let Some(under) = &filed_under {
            render::print_filed_under(style, under);
        }
    }
    render::print_entry(style, &entry, args.all);
    render::print_footer(style, from_cache, !args.verbatim && !args.exact);

    ExitCode::SUCCESS
}

fn print_canadian_block(
    style: &Style,
    spelling: Option<&canadian::SpellingMatch>,
    canadianism: Option<&canadian::Canadianism>,
    redirected: bool,
) {
    if let Some(m) = spelling {
        render::print_spelling_note(style, m, redirected);
    }
    if let Some(c) = canadianism {
        render::print_canadianism(style, c);
    }
}

fn print_canadian_only(
    style: &Style,
    word: &str,
    spelling: Option<&canadian::SpellingMatch>,
    canadianism: Option<&canadian::Canadianism>,
    redirected: bool,
) -> ExitCode {
    if spelling.is_none() && canadianism.is_none() {
        println!("No Canadian spelling or usage note recorded for {word:?}.");
        println!("Canadian and American English spell it the same way.");
        return ExitCode::SUCCESS;
    }
    println!();
    print_canadian_block(style, spelling, canadianism, redirected);
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
fn print_json(
    _style: &Style,
    queried: &str,
    target: &str,
    entry: &Entry,
    spelling: Option<&canadian::SpellingMatch>,
    canadianism: Option<&canadian::Canadianism>,
    redirected: bool,
    from_cache: bool,
    filed_under: Option<&str>,
) {
    use serde_json::{json, Value};

    let sense_json = |s: &model::Sense| -> Value {
        json!({
            "definition": markup::strip_emphasis(&s.text),
            "labels": s.labels,
            "canadian": s.is_canadian(),
            "examples": s.examples.iter().map(|e| markup::strip_emphasis(e)).collect::<Vec<_>>(),
            "synonyms": s.synonyms,
            "antonyms": s.antonyms,
        })
    };

    let sections: Vec<Value> = entry
        .sections
        .iter()
        .map(|section| {
            json!({
                "partOfSpeech": section.part_of_speech,
                "etymologyGroup": section.etymology,
                "senses": section.senses.iter().map(|s| {
                    let mut v = sense_json(s);
                    v["subsenses"] = Value::Array(s.subsenses.iter().map(sense_json).collect());
                    v
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    let pronunciations: Vec<Value> = entry
        .pronunciations
        .iter()
        .map(|p| json!({ "accents": p.accents, "ipa": p.ipa, "canadian": p.is_canadian() }))
        .collect();

    let canadian = json!({
        "spelling": spelling.map(|m| json!({
            "canadian": m.canadian_form,
            "other": m.other_form,
            "otherVariety": m.variant.contrast.other_name(),
            "agreesWith": m.variant.contrast.shared_with(),
            "queriedCanadianForm": m.queried_canadian,
            "note": m.note(),
        })),
        "canadianism": canadianism.map(|c| json!({
            "word": c.word,
            "gloss": c.gloss,
            "region": c.region,
        })),
    });

    let out = json!({
        "queried": queried,
        "word": target,
        "redirected": redirected,
        "filedUnder": filed_under,
        "fromCache": from_cache,
        "pronunciations": pronunciations,
        "canadian": canadian,
        "sections": sections,
        "source": "Wiktionary (CC BY-SA 4.0)",
    });

    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}

fn plural<'a>(n: usize, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 {
        one
    } else {
        many
    }
}
