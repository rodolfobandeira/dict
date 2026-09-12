use clap::Parser;

/// A Canadian English dictionary for the command line.
///
/// Definitions come from Wiktionary and are cached on disk, so any word you
/// have looked up once stays available offline. Canadian spelling guidance and
/// the built-in glossary of Canadianisms always work offline.
#[derive(Parser, Debug)]
#[command(name = "dict", version, about, long_about)]
pub struct Args {
    /// The word to look up.
    #[arg(value_name = "WORD")]
    pub word: Option<String>,

    /// Never touch the network; use only the cache and built-in Canadian data.
    #[arg(short, long)]
    pub offline: bool,

    /// Ignore the cache and re-fetch the entry.
    #[arg(short, long, conflicts_with = "offline")]
    pub refresh: bool,

    /// Look up the word exactly as typed, without redirecting American
    /// spellings to their Canadian form.
    #[arg(short = 'x', long)]
    pub exact: bool,

    /// Show only the Canadian spelling and usage notes, not the definitions.
    #[arg(short, long)]
    pub canadian: bool,

    /// Show every sense, including rare, obsolete and dialectal ones.
    #[arg(short, long)]
    pub all: bool,

    /// Leave Wiktionary's spellings untouched instead of converting them to
    /// their Canadian forms.
    #[arg(long)]
    pub verbatim: bool,

    /// Emit the entry as JSON.
    #[arg(long)]
    pub json: bool,

    /// Disable coloured output (also honoured via NO_COLOR).
    #[arg(long)]
    pub no_color: bool,

    /// Print where the on-disk cache lives.
    #[arg(long, exclusive = true)]
    pub cache_dir: bool,

    /// Delete every cached entry.
    #[arg(long, exclusive = true)]
    pub clear_cache: bool,
}
