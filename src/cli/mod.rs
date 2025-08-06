mod enumfile;
pub use enumfile::BuiltinOrFile;
mod builtins;
mod reclap;
use clap::{Args, Parser, Subcommand, ValueEnum};
pub use reclap::ReClap;

use crate::cli::builtins::impl_builtin_file;

/// Simple program to greet a person
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd0,
}

#[derive(Debug, Subcommand)]
#[command(subcommand_precedence_over_arg = true)]
pub enum Cmd0 {
    /// Choose the initial library to work with.
    Library(ReClap<OptsLibrary, CmdI>),
    #[command(flatten)]
    Other(CmdI),
}

#[derive(Debug, Subcommand)]
#[command(subcommand_precedence_over_arg = true)]
pub enum CmdI {
    /// Filter the library
    Filter(ReClap<OptsFilter, Self>),
    /// Match a regex pattern against the library.
    Match(ReClap<OptsMatch, Self>),
    /// Search for anagrams of a given pattern.
    Anna(ReClap<OptsAnna, Self>),
    /// Perform a fuzzy match against the corpus.
    Fuzzy(ReClap<OptsFuzzy, Self>),
    /// Filter to words containing at least all of the given letters.
    Has(ReClap<OptsHas, Self>),
    #[command(flatten)]
    /// Final command to execute.
    Final(CmdN),
}

#[derive(Debug, Subcommand)]
#[command(subcommand_precedence_over_arg = true)]
pub enum CmdN {
    /// Print the results.
    Show(ReClap<OptsShow, Self>),
    /// Write the library to a file.
    Write(ReClap<OptsWrite, Self>),
    /// Display statistics about the library.
    Stats(ReClap<OptsStats, Self>),
}

#[derive(Debug, Args)]
pub struct Depth {
    /// Depth of the search.
    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,
}

#[derive(Debug, Args)]
pub struct OptsMatch {
    /// Regex pattern to match against the library.
    pub pattern: String,

    #[command(flatten)]
    pub depth: Depth,
}

#[derive(Debug, Args)]
pub struct OptsAnna {
    /// Characters of the anagram to search for.
    pub pattern: String,
    /// Number of wildcards (unknowns) in the anagram.
    #[arg(short, long = "wild", default_value_t = 0)]
    pub wildcards: usize,
    /// Find partial anagrams.
    /// These are anagrams that can be formed from a subset of the letters.
    #[arg(short, long, default_value_t = false)]
    pub partial: bool,

    #[command(flatten)]
    pub depth: Depth,
}

#[derive(Debug, Args)]
pub struct OptsFuzzy {
    /// String to perform a fuzzy match against.
    pub pattern: String,

    /// Maximum number of edits (insertions, deletions, substitutions) allowed.
    /// Edits == Levenshtein distance.
    ///
    /// When unspecified, the default is to find the nearest matches.
    #[arg(short, long, value_delimiter = ',')]
    pub edits: Vec<u8>,

    /// Maximum distance to consider a match.
    /// When unspecified, the max is equal to the length of the pattern.
    #[arg(short, long, conflicts_with = "edits")]
    pub max: Option<u8>,

    #[command(flatten)]
    pub depth: Depth,
}

#[derive(Debug, Args)]
pub struct OptsFilter {
    /// Negate the filter (i.e., remove instead of keep).
    #[arg(short, long, default_value_t = false)]
    pub negate: bool,
    /// File to list of words to remove from the library.
    pub wordlist: Option<BuiltinOrFile<BuiltinsFilter>>,
    /// Keep the top N words by popularity.
    #[arg(short, long)]
    pub top: Option<usize>,
    /// Keep words that have occurred at least N times.
    #[arg(short, long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Debug, Args)]
pub struct OptsHas {
    /// Letters that must be present in the words.
    pub characters: String,
}

#[derive(Debug, Default, Args)]
pub struct OptsLibrary {
    /// Path to the library file.
    pub file: BuiltinOrFile<BuiltinsLibrary>,
    /// Format of the library file.
    #[arg(short, long)]
    pub format: Option<LibraryFormat>,
    /// Build the library from a string of words.
    #[arg(short, long, conflicts_with = "format")]
    pub build: bool,
    /// Minimum count of occurrences for a word to be included in the library.
    #[arg(short, long, default_value_t = 1, requires = "build")]
    pub threshold: u64,
    /// Ignore case when building the library.
    /// This will convert all words to lowercase.
    #[arg(short, long, default_value_t = false, requires = "build")]
    pub ignore_case: bool,
}

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum LibraryFormat {
    CSV,
    TSV,
}

#[derive(Debug, Default, Args)]
pub struct OptsShow {
    /// Add a header to the output denoting the columns.
    #[arg(short, long)]
    pub title: bool,
    /// Rank of the word in the local library.
    #[arg(short, long)]
    pub rank: bool,
    /// Global index of the word in the library.
    #[arg(short, long)]
    pub index: bool,
    /// Occurrences of the word in the global library.
    #[arg(short, long)]
    pub count: bool,
    /// Frequency of the word in the local library.
    #[arg(short, long)]
    pub frequency: bool,
}

#[derive(Debug, Args)]
pub struct OptsWrite {
    /// TODO: unimplemented
    pub unimplemented: String,
}

#[derive(Debug, Args)]
pub struct OptsStats {
    #[arg(short, long, value_enum, default_value_t = StatFormat::Human)]
    pub format: StatFormat,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, ValueEnum)]
pub enum StatFormat {
    /// Print the stats in a human-readable format.
    #[default]
    Human,
    /// Print the stats in a machine-readable format (JSON).
    Json,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, ValueEnum)]
pub enum BuiltinsLibrary {
    /// Google ngram corpus.
    #[default]
    Google,
}

impl_builtin_file!(
    BuiltinsLibrary,
    Google => "corpus/google.tsv"
);

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum BuiltinsFilter {
    /// Scrabble words.
    Scrabble,
}

impl_builtin_file!(
    BuiltinsFilter,
    Scrabble => "corpus/scrabble.tsv"
);
