mod cli;
use clap::Parser;
use grumpr::librarian::{Gram, Librarian, Library, Stats, query};
use itertools::Itertools;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
    process::ExitCode,
};
use unicode_segmentation::UnicodeSegmentation;

fn main() -> ExitCode {
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    use cli::*;
    let cli = Cli::parse();

    let (library, cmd_i) = process_cmd_0(cli.cmd)?;
    let mut librarian: Librarian = (&library).into();
    let cmd_n = process_cmd_i(&mut librarian, cmd_i)?;
    process_cmd_n(librarian, cmd_n)?;

    Ok(())
}

fn process_cmd_0(
    cmd: cli::Cmd0,
) -> Result<(Library, Option<cli::CmdI>), Box<dyn std::error::Error>> {
    use cli::Cmd0;

    Ok(match cmd {
        Cmd0::Library(opts) => (get_library(Some(opts.inner))?, opts.next.map(|cmd| *cmd)),
        Cmd0::Other(cmd) => (get_library(None)?, Some(cmd)),
    })
}

fn process_cmd_i(
    librarian: &mut Librarian,
    mut cmd_i: Option<cli::CmdI>,
) -> Result<Option<cli::CmdN>, Box<dyn std::error::Error>> {
    use cli::CmdI;

    while let Some(cmd) = cmd_i {
        cmd_i = match cmd {
            CmdI::Filter(opts) => {
                // The order these are applied matters but I don't document this, oh well.

                if let Some(mut file) = opts.inner.wordlist {
                    let reader = std::io::BufReader::new(file.reader());
                    let lines = reader.lines().collect::<Result<Vec<_>, std::io::Error>>()?;
                    let list = lines.iter().map(|s| s.as_str());
                    *librarian = if opts.inner.negate {
                        librarian.blacklist(list)
                    } else {
                        librarian.whitelist(list)
                    };
                }

                if opts.inner.count > 1 {
                    *librarian = if opts.inner.negate {
                        librarian.filter(|seed| seed.count < opts.inner.count as u64)
                    } else {
                        librarian.filter(|seed| seed.count >= opts.inner.count as u64)
                    };
                }
                if let Some(top) = opts.inner.top {
                    let seed = librarian
                        .iter()
                        .flat_map(|gram| gram.seeds())
                        .sorted_by(|lhs, rhs| rhs.count.cmp(&lhs.count))
                        .dedup_by(|lhs, rhs| lhs.index.eq(&rhs.index))
                        .nth(top);
                    if let Some(seed) = seed {
                        let count = seed.count;
                        *librarian = if opts.inner.negate {
                            librarian.filter(|seed| seed.count <= count)
                        } else {
                            librarian.filter(|seed| seed.count > count)
                        };
                    }
                }

                opts.next
            }
            CmdI::Match(opts) => {
                let query =
                    query::Match::new(&opts.inner.pattern).depth(opts.inner.depth.depth - 1);
                *librarian = librarian.search(&query).unwrap();
                opts.next
            }
            CmdI::Anna(opts) => {
                let query = query::Anagram::new(&opts.inner.pattern)
                    .partial(opts.inner.partial)
                    .wildcards(opts.inner.wildcards)
                    .depth(opts.inner.depth.depth - 1);
                *librarian = librarian.anagrams(&query).unwrap();
                opts.next
            }
            CmdI::Fuzzy(opts) => {
                let max_edits = opts.inner.max.unwrap_or(opts.inner.pattern.len() as u8);
                if opts.inner.edits.is_empty() {
                    // Find the nearest match
                    let query = query::Nearest::new(&opts.inner.pattern, max_edits);
                    *librarian = librarian.nearest(&query)?.0;
                } else {
                    // Find matches with the specified edit distances
                    let query =
                        query::Distance::new(&opts.inner.pattern, opts.inner.edits).strict(true);
                    *librarian = librarian.distance(&query)?;
                }

                opts.next
            }
            CmdI::Has(opts) => {
                let query = query::Has::new(&opts.inner.characters);
                *librarian = librarian.has(&query).unwrap();

                opts.next
            }
            CmdI::Final(final_cmd) => {
                return Ok(Some(final_cmd));
            }
        }
        .take()
        .map(|cmd| *cmd);
    }

    Ok(None)
}

fn process_cmd_n(
    librarian: Librarian,
    mut cmd_n: Option<cli::CmdN>,
) -> Result<(), Box<dyn std::error::Error>> {
    use cli::CmdN;

    // If no command was specified, default to showing the results
    if cmd_n.is_none() {
        cmd_n = Some(CmdN::Show(cli::ReClap::new(cli::OptsShow {
            title: true,
            rank: false,
            index: true,
            count: false,
            frequency: true,
        })));
    }

    while let Some(cmd) = cmd_n {
        cmd_n = match cmd {
            CmdN::Show(opts) => {
                let mut stdout = std::io::stdout().lock();
                if opts.inner.title {
                    writeln!(stdout, "{}", ShowHeader { opts: &opts.inner })?;
                }
                let total = if opts.inner.frequency {
                    librarian.iter().map(|gram| gram.count_mean()).sum()
                } else {
                    0
                };

                // TODO: Expose to cli how to sort the results
                // Also limit the number of results
                let grams = librarian
                    .iter()
                    .enumerate()
                    .sorted_by(|(_, lhs), (_, rhs)| lhs.cmp_by_count_mean(rhs))
                    .rev();

                // TODO: Format the results nicely in a table with padding
                for (index, gram) in grams {
                    let show_gram = ShowGram {
                        gram,
                        total,
                        rank: index,
                        opts: &opts.inner,
                    };
                    writeln!(stdout, "{}", show_gram)?;
                }

                opts.next
            }
            CmdN::Write(opts) => {
                todo!("Write librarian to file {:#?}", opts);
            }
            CmdN::Stats(opts) => {
                let stats = librarian.stats();
                match opts.inner.format {
                    cli::StatFormat::Human => {
                        let show = ShowStats::from(stats);
                        println!("{show}");
                    }
                    cli::StatFormat::Json => {
                        let stdout = std::io::stdout().lock();
                        serde_json::to_writer_pretty(stdout, &stats)?;
                    }
                }
                // TODO: Page count, etc.

                opts.next
            }
        }
        .take()
        .map(|cmd| *cmd);
    }

    Ok(())
}

fn get_library(opts: Option<cli::OptsLibrary>) -> Result<Library, Box<dyn std::error::Error>> {
    use cli::{BuiltinOrFile, LibraryFormat};
    let mut opts = opts.unwrap_or_default();

    if opts.build {
        if matches!(opts.file, BuiltinOrFile::Builtin(_)) {
            return Err("Built-in libraries cannot be built".into());
        }

        let file = opts.file.reader();
        library_build(file, opts.threshold, opts.ignore_case)
    } else {
        let format = match &opts.file {
            BuiltinOrFile::Builtin(_) => {
                // All builtins are TSV
                // This catches where the user doesn't specify a format
                // but they specify a built-in library
                matches!(opts.format, None | Some(LibraryFormat::TSV))
                    .then_some(LibraryFormat::TSV)
                    .ok_or(<Box<dyn std::error::Error>>::from(
                        "Built-in libraries must be in TSV format",
                    ))?
                // TODO: Look at embedding the built-in in a faster format to read /
                // mmap it directly as a Library?
            }
            BuiltinOrFile::File(file) if file.is_local() => {
                // If the file is local, we can determine the format from the file extension
                match file.path().extension().and_then(|s| s.to_str()) {
                    Some("tsv") => LibraryFormat::TSV,
                    Some("csv") => LibraryFormat::CSV,
                    _ => {
                        return Err("Unable to determine library format from file extension".into());
                    }
                }
            }
            // If the file is not local, we cannot determine the format
            // and we require the user to specify it
            BuiltinOrFile::File(_) => opts.format.ok_or(<Box<dyn std::error::Error>>::from(
                "Unable to determine library format from file",
            ))?,
        };

        library_parse(opts.file.reader(), format)
    }
}

fn library_build(
    file: impl std::io::BufRead,
    threshold: u64,
    ignore_case: bool,
) -> Result<Library, Box<dyn std::error::Error>> {
    let mut counter = HashMap::<String, u64>::new();

    for line in file.lines() {
        let line = line?;
        for word in line.unicode_words() {
            let word = if ignore_case {
                word.to_lowercase()
            } else {
                word.to_string()
            };
            *counter.entry(word).or_default() += 1;
        }
    }

    Ok(counter
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .collect())
}

fn library_parse(
    file: impl std::io::Read,
    format: cli::LibraryFormat,
) -> Result<Library, Box<dyn std::error::Error>> {
    use cli::LibraryFormat;

    #[derive(Debug, serde::Deserialize)]
    struct GramRecord {
        pub root: String,
        pub count: u64,
    }

    Ok(match format {
        LibraryFormat::TSV => {
            let mut parser = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b'\t')
                .from_reader(file);
            parser
                .deserialize()
                .map(|res: Result<GramRecord, _>| res.map(|rec| (rec.root, rec.count)))
                .collect::<csv::Result<Library>>()?
        }
        LibraryFormat::CSV => {
            let mut parser = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(file);
            parser
                .deserialize()
                .map(|res: Result<GramRecord, _>| res.map(|rec| (rec.root, rec.count)))
                .collect::<csv::Result<Library>>()?
        }
    })
}

#[derive(Debug, Clone)]
struct ShowHeader<'a> {
    opts: &'a cli::OptsShow,
}

impl<'a> std::fmt::Display for ShowHeader<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NGram\t")?;
        if self.opts.rank {
            write!(f, "Rank\t")?;
        }
        if self.opts.index {
            write!(f, "Index\t")?;
        }
        if self.opts.frequency {
            write!(f, "Frequency\t")?;
        }
        if self.opts.count {
            write!(f, "Count\t")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ShowGram<'a, 'l> {
    gram: Gram<'l>,
    rank: usize,
    total: u64,
    opts: &'a cli::OptsShow,
}

impl<'a, 'l> std::fmt::Display for ShowGram<'a, 'l> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.gram {
            Gram::Word(seed) => {
                write!(f, "{}\t", seed.root)?;
                if self.opts.rank {
                    write!(f, "{}\t", self.rank)?;
                }
                if self.opts.index {
                    write!(f, "{}\t", seed.index)?;
                }
                if self.opts.frequency {
                    write!(f, "{:.5}%\t", seed.count as f64 / self.total as f64 * 100.0)?;
                }
                if self.opts.count {
                    write!(f, "{}\t", seed.count)?;
                }
            }
            Gram::Sequence(seeds) => {
                write!(f, "{}", seeds.into_iter().map(|w| &w.root).join(" "))?;
                if self.opts.rank {
                    write!(f, "\t{}", self.rank)?;
                }
                if self.opts.index {
                    write!(f, "\t{}", seeds.iter().map(|s| s.index).join(","))?;
                }
                if self.opts.frequency {
                    write!(
                        f,
                        "\t{:.5}%",
                        seeds.iter().map(|s| s.count).sum::<u64>() as f64
                            / seeds.len() as f64
                            / self.total as f64
                            * 100.0
                    )?;
                }
                if self.opts.count {
                    write!(
                        f,
                        "\t{}",
                        seeds.iter().map(|s| s.count).sum::<u64>() / seeds.len() as u64
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct ShowStats(Stats);

impl std::fmt::Display for ShowStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "Seeds       \t{}\t{}",
            self.0.seeds, self.0.occurrences.seeds
        )?;
        writeln!(
            f,
            "NGrams      \t{}\t{}",
            self.0.ngrams, self.0.occurrences.ngrams
        )?;
        writeln!(f, "NGram Seeds \t{}", self.0.ngram_seeds)?;
        writeln!(
            f,
            "Chars Seeds \t{}\t{}",
            self.0.chars_seeds, self.0.occurrences.chars_seeds
        )?;
        writeln!(
            f,
            "Chars NGrams\t{}\t{}",
            self.0.chars_ngrams, self.0.occurrences.chars_ngrams
        )?;

        Ok(())
    }
}

impl ShowStats {
    fn new(stats: Stats) -> Self {
        Self(stats)
    }
}

impl From<Stats> for ShowStats {
    fn from(value: Stats) -> Self {
        Self::new(value)
    }
}
