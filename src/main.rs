use std::{fs::File, path::PathBuf, process::ExitCode};

use clap::{builder::TypedValueParser, ArgMatches};
use grumpr::{
    dataset,
    gram::{self, Book, Corpus, Display},
};
use indoc::indoc;
use itertools::Itertools;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("Unable to load the corpus - {0}")]
    Corpus(#[from] std::io::Error),
    #[error("Corpus is empty")]
    EmptyCorpus,
    #[error("The ngram '{0}' is not in the corpus")]
    NotInCorpus(String),
    #[error("The ngram '{0}' has no anagrams in the corpus")]
    NoAnagrams(String),
    #[error("The pattern '{0}' is not contained within the corpus")]
    NoMatches(String),
    #[error("The regex pattern is invalid: {0}")]
    RegexPattern(#[from] regex::Error),
    #[error("The rank '{0}' is too large for the corpus of size '{1}'")]
    Rank(usize, usize),
    #[error("The position '{1}' stated for character '0' is beyond the length of the desired word '{2}'")]
    CharLocation(char, usize, usize),
}

fn main() -> ExitCode {
    match entry() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_corpus(corpus: Corpus, matches: &clap::ArgMatches, display: Display) -> Result<(), Error> {
    if corpus.is_empty() {
        Err(Error::EmptyCorpus)
    } else {
        let corpus = if let Some(limit) = matches.get_one::<usize>("ngrams_count") {
            corpus.truncate(*limit)
        } else {
            corpus
        };

        println!(
            "{}",
            corpus.display(Display {
                rank: matches.get_flag("rank_rank"),
                count: matches.get_flag("rank_count"),
                frequency: matches.get_flag("rank_freq"),
                ..display
            })
        );
        Ok(())
    }
}

fn book_or_make(cli: &ArgMatches) -> Result<Book, std::io::Error> {
    if let Some(filename) = cli.get_one::<PathBuf>("book_file") {
        Ok(dataset::generate(
            dataset::extract_raw(File::open(filename)?),
            *cli.get_one::<u64>("corpus_threshold").expect("required"),
        ))
    } else {
        dataset::file::raw(cli.get_one::<PathBuf>("corpus").expect("required"))
    }
}

fn run_with_corpus(
    cli: &ArgMatches,
    f: impl FnOnce(Corpus) -> Result<(), Error>,
) -> Result<(), Error> {
    let book = book_or_make(cli)?;
    let corpus = book.corpus();

    let corpus = if let Some(filename) = cli.get_one::<PathBuf>("filter") {
        let filter = dataset::file::filter(filename)?;

        corpus
            .iter()
            .filter(|&gram| filter.contains(&gram.root.string))
            .collect()
    } else {
        corpus
    };

    let corpus = if let Some(corpus_size) = cli.get_one::<usize>("corpus_size") {
        corpus.truncate(*corpus_size)
    } else {
        corpus
    };

    f(corpus)
}

fn entry() -> Result<(), Error> {
    let cli = cli().get_matches();

    match cli.subcommand() {
        Some(("rank", command)) => run_with_corpus(&cli, |corpus| {
            let ngram = command
                .get_one::<String>("ngram")
                .expect("ngram is required");
            if let Some(gram) = corpus.find(ngram) {
                println!(
                    "{}",
                    gram.display(Display {
                        rank: command.get_flag("rank_rank"),
                        count: command.get_flag("rank_count"),
                        frequency: command.get_flag("rank_freq"),
                        ..Default::default()
                    })
                );
                Ok(())
            } else {
                Err(Error::NotInCorpus(ngram.clone()))
            }
        }),
        Some(("spot", command)) => run_with_corpus(&cli, |corpus| {
            let begin = *command
                .get_one::<usize>("gram_index")
                .expect("required argument");
            let count = corpus.grams.len();
            if begin >= count {
                return Err(Error::Rank(begin, count));
            }

            let end = begin + command.get_one::<usize>("ngrams_count").expect("default");
            let slice = &corpus.grams[begin..end.min(count)];

            {
                let flags = Display {
                    string: true,
                    rank: command.get_flag("rank_rank"),
                    count: command.get_flag("rank_count"),
                    frequency: command.get_flag("rank_freq"),
                    ..Default::default()
                };
                let mut f = std::io::stdout().lock();
                for gram in slice {
                    use std::io::Write;
                    let _ = writeln!(f, "{}", gram.display(flags));
                }
            };

            Ok(())
        }),
        Some(("anna", command)) => run_with_corpus(&cli, |corpus| {
            let ngram = command
                .get_one::<String>("ngram")
                .expect("ngram is required");

            let pattern = command.get_one::<String>("pattern");

            let corpus = corpus.anagrams(ngram);

            match pattern {
                Some(pattern) if !corpus.is_empty() => print_corpus(
                    corpus.wildcard(Regex::new(pattern)?),
                    command,
                    Display {
                        string: true,
                        ..Default::default()
                    },
                )
                .map_err(|_| Error::NoMatches(pattern.to_owned())),
                _ => print_corpus(
                    corpus,
                    command,
                    Display {
                        string: true,
                        ..Default::default()
                    },
                )
                .map_err(|_| Error::NoAnagrams(ngram.clone())),
            }
        }),
        Some(("fuzzy", command)) => run_with_corpus(&cli, |corpus| {
            let ngram = command
                .get_one::<String>("ngram")
                .expect("ngram is required");

            print_corpus(
                corpus.fuzzy_find(
                    ngram,
                    command
                        .get_one::<usize>("ngrams_count")
                        .copied()
                        .unwrap_or(corpus.len()),
                ),
                command,
                Display {
                    string: true,
                    ..Default::default()
                },
            )
            .map_err(|_| Error::NotInCorpus(ngram.clone()))
        }),
        Some(("levenshtein", command)) => {
            let ngrams: Vec<String> = command
                .get_many("ngram")
                .expect("ngrams are required")
                .cloned()
                .collect_vec();
            let dist = gram::levenshtein(&ngrams[0], &ngrams[1]);
            println!("{}", dist);
            Ok(())
        }
        Some(("match", command)) => run_with_corpus(&cli, |corpus| {
            let pattern = command
                .get_one::<String>("pattern")
                .expect("pattern is required");

            print_corpus(
                corpus.wildcard(Regex::new(pattern)?),
                command,
                Display {
                    string: true,
                    ..Default::default()
                },
            )
            .map_err(|_| Error::NoMatches(pattern.to_owned()))
        }),
        Some(("stat", command)) => run_with_corpus(&cli, |corpus| {
            let summary = command.get_one::<String>("summary").expect("has default");
            let extra_info = (summary != "none" && !corpus.is_empty()).then(|| {
                let grams_unique = corpus.len() as u64;
                let grams_total = corpus.grams.iter().map(|g| g.root.count).sum::<u64>();
                let chars = corpus
                    .grams
                    .iter()
                    .map(|g| g.string.chars().count() as u64)
                    .sum::<u64>();
                let chars_total = chars * grams_total / grams_unique;
                let ws_total = chars_total + grams_total - 1;
                format!(
                    "Grams: Unique({}) Total({})\nChars: Unique({}) Total({}) TotalWS({})\nPerGram: Unique({:.5}) Total({:.5})\nMisc : Pages({:.2})",
                    grams_unique,
                    grams_total,
                    chars,
                    chars_total,
                    ws_total,
                    chars as f64 / grams_unique as f64,
                    chars_total as f64 / grams_total as f64,
                    ws_total as f64 / 3000.0
                )
            });
            let res = if summary != "only" {
                print_corpus(
                    corpus,
                    command,
                    Display {
                        string: true,
                        ..Default::default()
                    },
                )
            } else {
                Ok(())
            };
            if let Some(extra_info) = extra_info {
                let nl = if summary != "only" { "\n" } else { "" };
                println!("{nl}{extra_info}");
            }
            res
        }),
        Some(("wordle", command)) => run_with_corpus(&cli, |corpus| {
            fn turn_char_pos_into_pos_map<'a>(
                length: usize,
                it: impl Iterator<Item = &'a (char, usize)>,
            ) -> Result<Vec<String>, Error> {
                let mut pos_map = vec![String::new(); length];
                for (char, pos) in it.copied() {
                    pos_map
                        .get_mut(pos - 1)
                        .ok_or_else(|| Error::CharLocation(char, pos, length))?
                        .push(char);
                }
                Ok(pos_map)
            }
            let length = command
                .get_one::<u16>("length")
                .expect("default")
                .to_owned() as usize;

            let pattern_not = command
                .get_many::<String>("not")
                .map(|mut strings| strings.join(""))
                .unwrap_or_default();
            let pattern_has = command
                .get_many::<(char, usize)>("has")
                .map(|pairs| turn_char_pos_into_pos_map(length, pairs))
                .unwrap_or_else(|| Ok(vec![String::new(); length]))?;
            let pattern_is = command
                .get_many::<(char, usize)>("is")
                .map(|pairs| turn_char_pos_into_pos_map(length, pairs))
                .unwrap_or_else(|| Ok(vec![String::new(); length]))?;

            let all_has = pattern_has
                .iter()
                .flat_map(|s| s.chars())
                .sorted_unstable()
                .dedup()
                .collect_vec();

            let pattern = (0..length)
                .map(|i| {
                    if !pattern_is[i].is_empty() {
                        pattern_is[i].clone()
                    } else if !pattern_has[i].is_empty() {
                        format!("[^{}]", pattern_not.clone() + &pattern_has[i])
                    } else {
                        format!("[^{}]", pattern_not)
                    }
                })
                .join("");
            let pattern = format!("^{pattern}$");

            let corpus = corpus.wildcard(Regex::new(&pattern)?);
            let corpus = corpus
                .iter()
                .filter(|&gram| {
                    for &c in &all_has {
                        if !gram.string.contains(c) {
                            return false;
                        }
                    }
                    return true;
                })
                .collect();

            print_corpus(
                corpus,
                command,
                Display {
                    string: true,
                    ..Default::default()
                },
            )
            .map_err(|_| Error::NoMatches(pattern))
        }),
        Some(("corpus", command)) => {
            let text_iterator = command
                .get_many::<PathBuf>("input_file")
                .expect("required")
                .flat_map(|filename| File::open(filename).map(dataset::extract_raw))
                .flatten();

            let threshold = command
                .get_one::<u64>("corpus_threshold")
                .expect("required");

            let book = dataset::generate(text_iterator, *threshold);

            fn write_book(book: Book, mut f: impl std::io::Write) -> Result<(), std::io::Error> {
                for root in book.dataset {
                    writeln!(f, "{}\t{}", root.string, root.count)?;
                }
                Ok(())
            }

            if let Some(filename) = command.get_one::<PathBuf>("output_file") {
                let file = File::create(filename)?;
                write_book(book, file)
            } else {
                let io = std::io::stdout();
                let handle = io.lock();
                write_book(book, handle)
            }
            .map_err(|e| e.into())
        }
        _ => unreachable!("A subcommand is required!"),
    }
}

fn cli() -> clap::Command {
    use clap::*;
    use std::str::FromStr;

    fn parse_key_val<T: FromStr, U: FromStr>(input: &str) -> Result<(T, U), String>
    where
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
        <U as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let idx = input.trim().find('=');
        if let Some(idx) = idx {
            let char = input[..idx].parse().map_err(|e: T::Err| e.to_string())?;
            let location = input[idx + 1..]
                .parse()
                .map_err(|e: U::Err| e.to_string())?;
            Ok((char, location))
        } else {
            Err(format!("invalid KEY=value: no `=` found in `{}`", input))
        }
    }

    fn parse_char_pos(input: &str) -> Result<(char, usize), String> {
        match parse_key_val::<char, usize>(input) {
            Ok((char, location)) => {
                if location < 1 {
                    return Err(format!(
                        "invalid position '{location}': must be greater than 0"
                    ));
                }
                if !char.is_alphabetic() {
                    return Err(format!("invalid character '{char}': must be alphabetic"));
                }
                Ok((char, location))
            }
            e => e,
        }
    }

    // .arg(clap::Arg::new("corpus").short('c').long("corpus"))
    let rank_rank = Arg::new("rank_rank")
        .short('r')
        .long("rank")
        .value_name("SHOW")
        .action(ArgAction::Set)
        .value_parser(value_parser!(bool))
        .default_value("true")
        .default_missing_value("false")
        .num_args(0..=1)
        .require_equals(true)
        .help("Display the Rank of the ngrams");

    let rank_freq = Arg::new("rank_freq")
        .short('f')
        .long("freq")
        .value_name("SHOW")
        .action(ArgAction::Set)
        .value_parser(value_parser!(bool))
        .default_value("true")
        .default_missing_value("false")
        .num_args(0..=1)
        .require_equals(true)
        .help("Display the Frequency of the ngrams");

    let rank_count = Arg::new("rank_count")
        .short('c')
        .long("count")
        .value_name("SHOW")
        .action(ArgAction::Set)
        .value_parser(value_parser!(bool))
        .default_value("false")
        .default_missing_value("true")
        .num_args(0..=1)
        .require_equals(true)
        .help("Display the number of occurrences of the ngrams");

    let results_size = Arg::new("ngrams_count")
        .short('s')
        .long("size")
        .value_name("COUNT")
        .action(ArgAction::Set)
        .value_parser(value_parser!(usize))
        .help("Maximum number of ngrams to respond with");

    let corpus_threshold = Arg::new("corpus_threshold")
        .short('t')
        .long("threshold")
        .value_name("COUNT")
        .action(ArgAction::Set)
        .default_value("0")
        .value_parser(value_parser!(u64))
        .help("Number of occurrences an ngram requires before it is put in the corpus");

    let output_file = Arg::new("output_file")
        .short('o')
        .long("output")
        .action(ArgAction::Set)
        .value_name("FILE")
        .value_parser(value_parser!(PathBuf))
        .help("File to output the corpus data to. Defaults to stdout");

    command!()
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::new("corpus")
            .short('c')
            .long("corpus")
            .value_name("NAME")
            .action(ArgAction::Set)
            .value_parser(PossiblePathValueParser(grumpr::dataset::find::available_files("corpus"), "corpus"))
            .default_value("google")
            .help("Select a corpus to use. It can also be a file path to a corpus file."))
        .arg(Arg::new("corpus_size")
            .short('s')
            .long("size")
            .value_name("SIZE")
            .action(ArgAction::Set)
            .value_parser(value_parser!(usize))
            .help("Limit the number of ngrams in the corpus to the <SIZE> most frequent"))
        .arg(Arg::new("filter")
            .short('f')
            .long("filter")
            .value_name("NAME")
            .action(ArgAction::Set)
            .value_parser(PossiblePathValueParser(grumpr::dataset::find::available_files("filter"), "filter"))
            .help("TODO: ADD HELP"))
        .arg(Arg::new("book_file")
            .short('b')
            .long("book")
            .value_name("FILE")
            .action(ArgAction::Set)
            .value_parser(value_parser!(PathBuf))
            .help("Temporarily generate a corpus from a text input file and use it"))
        .arg(corpus_threshold.clone().requires("book_file"))
        .subcommand_required(true)
        .subcommand(Command::new("rank")
            .arg(Arg::new("ngram")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .help("The ngram to rank in the corpus"))
            .arg(rank_rank.clone()
                .help("Display the Rank of the <ngram>"))
            .arg(rank_freq.clone()
                .help("Display the Frequency of the <ngram>"))
            .arg(rank_count.clone()
                .help("Display the number of occurrences of the <ngram>"))
            .about("Returns a words rank in the corpus")
            .long_about(indoc! {
                "Returns the (zero-based) cardinality of an ngram with regards to the number of occurrences in the corpus.
                It also returns the frequency of the ngram relative to the rest of the corpus.
                The program will exit with a non-zero exit-code if the ngram is not found in the corpus"}))
        .subcommand(Command::new("spot")
            .arg(Arg::new("gram_index")
                .required(true)
                .action(ArgAction::Set)
                .value_name("RANK")
                .value_parser(value_parser!(usize))
                .help("A 0-based rank"))
            .arg(rank_rank.clone())
            .arg(rank_freq.clone())
            .arg(rank_count.clone())
            .arg(results_size.clone().default_value("1"))
            .about("Finds the nth most common ngram in the corpus.")
            .long_about(indoc! {"
                Finds the nth most common ngram in the corpus.
                The given rank is 0-based."}))
        .subcommand(Command::new("anna")
            .arg(Arg::new("ngram")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .help("The ngram to find anagrams of in the corpus"))
            .arg(Arg::new("pattern")
                .long("where")
                .value_name("PATTERN")
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .help("The regex pattern to match against ngrams in the corpus"))
            .arg(rank_rank.clone())
            .arg(rank_freq.clone())
            .arg(rank_count.clone())
            .arg(results_size.clone())
            .about("Finds anagrams of the given word")
            .long_about(indoc! {"
                Finds anagrams of the ngram within the corpus.
                The anagrams are returned in order of their rank.
                The ngram does not have to be in the corpus in the first place."}))
        .subcommand(Command::new("fuzzy")
            .arg(Arg::new("ngram")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .help("The ngram to search for in the corpus"))
            .arg(rank_rank.clone())
            .arg(rank_freq.clone())
            .arg(rank_count.clone())
            .arg(results_size.clone())
            .about("Find words similar to the given ngram")
            .long_about(indoc! {"
                Finds words similar to the ngram within the corpus.
                The words are returned in order of their levenshtein distance.
                The ngram does not have to be in the corpus in the first place."}))
        .subcommand(Command::new("levenshtein")
            .alias("lev")
            .arg(Arg::new("ngram")
                .required(true)
                .action(ArgAction::Set)
                .num_args(2)
                .value_parser(value_parser!(String))
                .help("The ngrams to compute the distance between"))
            .about("Compute the levenshtein distance between two ngrams")
            .long_about(indoc! {"
                Compute the levenshtein distance between two arbitrary ngrams.
                The ngrams do not have to be in the corpus in the first place."}))
        .subcommand(Command::new("match")
            .arg(Arg::new("pattern")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .help("The regex pattern to match against ngrams in the corpus"))
            .arg(rank_rank.clone())
            .arg(rank_freq.clone())
            .arg(rank_count.clone())
            .arg(results_size.clone())
            .about("Finds ngrams that match the regex pattern")
            .long_about(indoc! {"
                Finds ngrams in the corpus that match the regex pattern.
                The ngrams are returned in order of their rank."}))
        .subcommand(Command::new("wordle")
            .arg(rank_rank.clone())
            .arg(rank_freq.clone())
            .arg(rank_count.clone())
            .arg(results_size.clone())
            .arg(Arg::new("not")
                .long("not")
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("LETTER")
                .value_parser(value_parser!(String))
                .help("List of letters that are certainly not in the final word"))
            .arg(Arg::new("has")
                .long("has")
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("LETTER=POSITION")
                .value_parser(builder::ValueParser::new(parse_char_pos))
                .help("List of letters that have been found in the final word at a specific position, but that position is incorrect"))
            .arg(Arg::new("is")
                .long("is")
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("LETTER=POSITION")
                .value_parser(builder::ValueParser::new(parse_char_pos))
                .help("List of letters that are at exactly the correct position in the final word"))
            .arg(Arg::new("length")
                .short('l')
                .long("length")
                .value_parser(value_parser!(u16).range(1..))
                .default_value("5")
                .help("Length of the target ngram"))
            .about("Try to solve a wordle-esk puzzle"))
        .subcommand(Command::new("stat")
            .arg(rank_rank)
            .arg(rank_freq)
            .arg(rank_count)
            .arg(results_size)
            .arg(Arg::new("summary")
                .short('i')
                .long("info")
                .action(ArgAction::Set)
                .num_args(0..=1)
                .value_parser(["all", "only", "none"])
                .default_value("all")
                .default_missing_value("only"))
            .about("Display information about the corpus itself"))
        .subcommand(Command::new("corpus")
            .arg(output_file)
            .arg(corpus_threshold)
            .arg(Arg::new("input_file")
                .required(true)
                .action(ArgAction::Set)
                .value_name("FILE")
                .num_args(1..)
                .value_parser(value_parser!(PathBuf))
                .help("Text files to create a corpus from"))
            .about("Create a new corpus")
            .long_about(indoc! {"
                Creates a new corpus file from given input documents"}))
}

#[derive(Debug, Clone)]
struct PossiblePathValueParser(Vec<PathBuf>, &'static str);

impl TypedValueParser for PossiblePathValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = clap::builder::PathBufValueParser::new();
        let path = inner.parse_ref(cmd, arg, value)?;
        for mut search in dataset::find::search_paths() {
            search.push(&path);

            let find = |path: Option<PathBuf>| -> Option<&PathBuf> {
                path.map(|path| self.0.iter().find(|&p| p.starts_with(&path)))
                    .flatten()
            };

            if let Some(path) = find(search.canonicalize().ok()) {
                return Ok(path.clone());
            }
            search.set_extension(self.1);
            if let Some(path) = find(search.canonicalize().ok()) {
                return Ok(path.clone());
            }
        }
        let mut err = clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd);
        err.insert(
            clap::error::ContextKind::ValidValue,
            clap::error::ContextValue::Strings(
                self.0
                    .iter()
                    .map(|p| p.to_string_lossy().into())
                    .collect::<Vec<_>>(),
            ),
        );
        Err(err)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(self.0.iter().map(|p| {
            p.with_extension("")
                .file_name()
                .expect("Path will only be in the vector if it was correctly identified earlier")
                .to_string_lossy()
                .to_string()
                .into()
        })))
    }
}
