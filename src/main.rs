mod cli;
use clap::Parser;
use grumpr::librarian::{Gram, Librarian, Library, query};
use itertools::Itertools;
use std::{collections::HashMap, io::BufRead, process::ExitCode};
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
    dbg!("Loaded library");
    let mut librarian: Librarian = (&library).into();
    dbg!(librarian.len());
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
    cmd_n: Option<cli::CmdN>,
) -> Result<(), Box<dyn std::error::Error>> {
    use cli::CmdN;

    match cmd_n {
        Some(cmd_n) => match cmd_n {
            CmdN::Show(opts) => {
                todo!("{:#?}", opts);
            }
        },
        None => {
            // If no final command is specified, we just print the results
            librarian.into_iter().for_each(|gram| match gram {
                Gram::Word(word) => println!("{}", word.root),
                Gram::Sequence(words) => {
                    println!("{}", words.into_iter().map(|w| &w.root).join(" "))
                }
            });
        }
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
