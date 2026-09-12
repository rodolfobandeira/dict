# dict

A Canadian English dictionary for the command line.

```
$ dict compelling

compelling  /kəmˈpɛlɪŋ/

  ADJECTIVE
    1. very interesting; able to capture and hold one's attention
         “The novel was so compelling that I couldn't put it down.”
         synonyms: gripping
    2. capable of causing someone to believe or agree
         “He made a compelling argument.”
         synonyms: convincing
    3. strong and forceful; that causes one to feel like they must do something
         “I would need a very compelling reason to leave my job.”
         synonyms: urgent, pressing

  NOUN
    1. An act of compulsion; an obliging somebody to do something.

  VERB
    1. present participle of compel

  Wiktionary (CC BY-SA 4.0) · spellings shown in Canadian form
```

Wiktionary lists `compelling` as a verb form first, which buries the senses a
reader actually wants; parts of speech that only record an inflection are moved
below the ones that define the word.

## What makes it Canadian

Canadian English is not British English and not American English. It keeps
British `-our` and `-re`, takes American `-ize` and `-yze`, and picks sides word
by word everywhere else. `dict` knows which, and says so:

```
$ dict colour

colour  /ˈkʌl.ə/

🍁 “colour” is the Canadian spelling; American English writes “color”.
   Canadian English agrees with British English on this word.
   Canadian English keeps the British -our ending, but drops the u before the
   suffixes -ous, -ial, -ify and -ation: humour but humorous, honour but
   honorary, vigour but vigorous.
```

Type the American spelling and you land on the Canadian entry:

```
$ dict color

colour  /ˈkʌl.ə/

🍁 “color” is the American spelling. Canadian English writes “colour” —
   showing that entry.
```

Words that are two different words in Canadian English are reported, never
silently swapped:

```
$ dict --canadian cheque

🍁 Canadian usage: “cheque” and “check”.
   A cheque is the bank instrument; check covers every other sense, including
   the tick mark and the verb.
```

Distinctly Canadian words carry a glossary entry of their own, and Canadian
senses and pronunciations are flagged in the definitions:

```
$ dict toque

toque  /tuːk/, /tjuːk/ (Canada)

🍁 Canadianism — A close-fitting knitted winter hat, the central garment of
   Canadian winter. Also spelled tuque.

  ── sense group 2 ──

  NOUN
    1. 🍁 (Canada) A knitted hat, usually conical but of varying shape, often
       woollen, and sometimes topped by a pom-pom or tassel.
         synonyms: beanie, knit cap, stocking cap, watch cap
```

Definitions fetched from Wiktionary are shown with Canadian spellings, so the
entry for `colour` does not talk about `color`. Only rule-governed pairs are
converted; `story`/`storey`, `check`/`cheque` and `gray`/`grey` are left alone
because choosing between them needs the sense, not the spelling. Pass
`--verbatim` to see Wiktionary's own spellings.

## Offline

Definitions come from Wiktionary over the network and are cached under
`~/.cache/dict`, so any word looked up once stays available offline. The
Canadian spelling table and the glossary of Canadianisms are compiled into the
binary and never need the network at all.

```
$ dict --offline toque      # served from the cache
$ dict --cache-dir          # where the cache lives, and how full it is
$ dict --clear-cache
```

## Installing

Needs a Rust toolchain (1.74 or newer). The install script builds the release
binary and puts it on your PATH, so `dict` works from any directory:

```
./install.sh
```

It installs to `~/.local/bin` by default, tells you if that directory is not on
your PATH (and prints the line to add for your shell), and warns if another
`dict` earlier on your PATH would shadow it. It edits no shell config of its
own.

```
./install.sh --prefix /usr/local/bin   # somewhere else
./install.sh --uninstall               # remove it again
./install.sh --uninstall --purge       # and delete the cached entries
```

Or use cargo directly, if `~/.cargo/bin` is on your PATH:

```
cargo install --path .
cargo build --release    # or just build in place, at target/release/dict
```

## Usage

```
dict <WORD>

  -o, --offline      Never touch the network; use only the cache and built-in data
  -r, --refresh      Ignore the cache and re-fetch the entry
  -x, --exact        Look up the word as typed, without redirecting to the Canadian form
  -c, --canadian     Show only the Canadian spelling and usage notes
  -a, --all          Show every sense, including rare, obsolete and dialectal ones
      --verbatim     Leave Wiktionary's spellings untouched
      --json         Emit the entry as JSON
      --no-color     Disable colour (also honoured via NO_COLOR)
      --cache-dir    Print where the cache lives
      --clear-cache  Delete every cached entry
```

`--json` gives the whole entry, Canadian notes included, for scripting:

```
$ dict --json toque | jq '.canadian.canadianism.gloss'
"A close-fitting knitted winter hat, the central garment of Canadian winter. Also spelled tuque."
```

## How it works

Wiktionary's REST definition endpoint returns rendered HTML, but it expands
usage labels away — losing exactly the `{{lb|en|Canada}}` markers and
`a=Canada` pronunciations this tool is built around. So `dict` reads the raw
wikitext and parses it itself (`src/wikitext.rs`, `src/markup.rs`), which also
means the cache holds upstream wikitext: improvements to the parser apply to
everything already cached.

Many Canadian spellings are filed on Wiktionary as bare pointers — the page for
`colour` says only "standard spelling of color". `dict` detects those and
follows one hop to the real definitions, then reports where they came from.

| file | what it does |
| --- | --- |
| `src/canadian.rs` | the spelling table, the glossary, and the Canadianizing pass |
| `src/wikitext.rs` | parses the English section of a Wiktionary page |
| `src/markup.rs` | flattens wikitext links and templates to readable text |
| `src/model.rs` | the shape of an entry, independent of its source |
| `src/source.rs` | fetching, with the cache in front |
| `src/render.rs` | terminal output |

```
cargo test     # 40 tests, including integrity checks on the curated word lists
```

## Licence

The code is MIT. Definitions come from
[Wiktionary](https://en.wiktionary.org) and are licensed
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); `dict` says so
in the footer of every entry.
