use std::io::BufRead;

use grumpr::{Librarian, Library, Seed, librarian::query};
use include_flate::flate;
use itertools::Itertools;
use serde::Deserialize;
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn initialise() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Match {
    pub pattern: String,
    pub depth: usize,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Filter {
    pub id: Option<FilterID>,
    pub top: Option<usize>,
    pub invert: bool,
    pub count: usize,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Anagram {
    pub pattern: String,
    pub depth: usize,
    pub partial: bool,
    pub wildcards: usize,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Has {
    pub characters: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Command {
    Match(Match),
    Filter(Filter),
    Anagram(Anagram),
    Has(Has),
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NGrams {
    words: Vec<Vec<Seed>>,
}

#[wasm_bindgen]
pub fn process(library: Option<LibraryID>, commands: Box<[JsValue]>) -> Result<NGrams, String> {
    let library: Library = library.unwrap_or_default().into();
    let commands = commands
        .into_iter()
        .map(|js| serde_wasm_bindgen::from_value(js))
        .collect::<Result<Vec<Command>, _>>()
        .map_err(|err| err.to_string())?;
    let librarian = process_impl((&library).into(), commands).map_err(|err| err.to_string())?;

    // Convert into JS compatible format
    Ok(NGrams {
        words: librarian
            .into_iter()
            .map(|gram| gram.seeds().into_iter().cloned().collect())
            .collect(),
    })
}

fn process_impl<'l>(
    mut librarian: Librarian<'l>,
    commands: impl IntoIterator<Item = Command>,
) -> Result<Librarian<'l>, Box<dyn std::error::Error>> {
    for command in commands {
        match command {
            Command::Filter(filter) => {
                if let Some(filter_id) = filter.id {
                    let words = filter_id
                        .reader()
                        .lines()
                        .collect::<Result<Vec<_>, std::io::Error>>()?;
                    let list = words.iter().map(|s| s.as_str());
                    librarian = if filter.invert {
                        librarian.blacklist(list)
                    } else {
                        librarian.whitelist(list)
                    };
                }

                if filter.count > 1 {
                    librarian = if filter.invert {
                        librarian.filter(|seed| seed.count < filter.count as u64)
                    } else {
                        librarian.filter(|seed| seed.count >= filter.count as u64)
                    };
                }

                if let Some(top) = filter.top {
                    let seed = librarian
                        .iter()
                        .flat_map(|gram| gram.seeds())
                        .sorted_by(|lhs, rhs| rhs.count.cmp(&lhs.count))
                        .dedup_by(|lhs, rhs| lhs.index.eq(&rhs.index))
                        .nth(top);
                    if let Some(seed) = seed {
                        let count = seed.count;
                        librarian = if filter.invert {
                            librarian.filter(|seed| seed.count <= count)
                        } else {
                            librarian.filter(|seed| seed.count > count)
                        };
                    }
                }
            }
            Command::Match(Match { pattern, depth }) => {
                let query = query::Match::new(&pattern).depth(depth);
                librarian = librarian.search(&query)?;
            }
            Command::Anagram(Anagram {
                pattern,
                depth,
                partial,
                wildcards,
            }) => {
                let query = query::Anagram::new(&pattern)
                    .depth(depth)
                    .partial(partial)
                    .wildcards(wildcards);
                librarian = librarian.anagrams(&query)?;
            }
            Command::Has(Has { characters }) => {
                let query = query::Has::new(&characters);
                librarian = librarian.has(&query)?;
            }
        }
    }

    Ok(librarian)
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum LibraryID {
    #[default]
    Google,
}

// #[wasm_bindgen]
impl LibraryID {
    pub fn variants() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Deserialize)]
pub enum FilterID {
    Scrabble,
}

// #[wasm_bindgen]
impl FilterID {
    pub fn variants() -> Vec<Self> {
        Self::iter().collect()
    }
}

trait IncludeReader {
    fn reader(&self) -> std::io::Cursor<&'static [u8]>;
}

macro_rules! impl_include_reader {
    ($name:ident, $(($variant:ident, $path:literal)),*) => {
        impl IncludeReader for $name {
            fn reader(&self) -> std::io::Cursor<&'static [u8]> {
                match self {
                    $(Self::$variant => {
                        flate!(static DATA: [u8] from $path);
                        std::io::Cursor::new(DATA.as_slice())
                    })*
                }
            }
        }
    };
    () => {};
}

impl_include_reader!(LibraryID, (Google, "../corpus/google.tsv"));
impl_include_reader!(FilterID, (Scrabble, "../corpus/scrabble.tsv"));

impl From<LibraryID> for Library {
    fn from(id: LibraryID) -> Self {
        let reader = match id {
            LibraryID::Google => LibraryID::Google.reader(),
        };
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(reader);

        #[derive(Debug, Deserialize)]
        struct Record {
            word: String,
            count: u64,
        }

        rdr.deserialize()
            .filter_map(|result| {
                let Record { word, count } = result.ok()?;
                Some((word, count))
            })
            .collect()
    }
}
