use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

use itertools::Itertools;
use serde::de::DeserializeOwned;

use crate::gram::{Book, Root};

pub trait Extractor {
    fn extract(self) -> Root;
}

pub fn parse<Record: DeserializeOwned + Extractor>(reader: impl Read) -> Book {
    let mut book = Book::new();

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(reader);
    for (index, record) in rdr.deserialize::<Record>().flatten().enumerate() {
        let mut root = record.extract();
        root.string = root.string.to_lowercase();
        root.index = index;
        book.dataset.push(root);
    }
    book
}

fn normalise(input: &str) -> String {
    // use unicode_categories::UnicodeCategories;
    input.to_lowercase()
}

pub fn generate(text: impl IntoIterator<Item = String>, threshold: u64) -> Book {
    let mut map: HashMap<String, u64> = HashMap::new();
    for ngram in text {
        if ngram.contains(char::is_alphabetic) {
            *map.entry(normalise(&ngram)).or_default() += 1;
        }
    }
    Book {
        dataset: map
            .into_iter()
            .sorted_by_cached_key(|(_, count)| *count)
            .rev()
            .enumerate()
            .map(|(index, (string, count))| Root {
                string,
                count,
                index,
            })
            .filter(|element| element.count > threshold)
            .collect(),
    }
}

pub fn extract_raw(rdr: impl Read) -> impl Iterator<Item = String> {
    use std::io::BufRead;
    use unicode_segmentation::UnicodeSegmentation;
    let rdr = BufReader::new(rdr);
    rdr.lines().flat_map(|line| {
        line.unwrap()
            .split_word_bounds()
            .map(|word| word.to_string())
            .collect::<Vec<_>>()
    })
}

pub fn ngrams(rdr: impl Read) -> Book {
    #[derive(Debug, serde::Deserialize)]
    struct Record {
        pub name: String,
        pub count: u64,
    }

    impl Extractor for Record {
        fn extract(self) -> Root {
            Root::new(self.name, self.count)
        }
    }

    let rdr = BufReader::new(rdr);
    parse::<Record>(rdr)
}

pub mod file {
    use super::Book;
    use std::{collections::HashSet, fs::File, io::BufReader, path::Path};

    pub fn raw(path: impl AsRef<Path>) -> Result<Book, std::io::Error> {
        File::open(path).map(super::ngrams)
    }

    pub fn filter(path: impl AsRef<Path>) -> Result<HashSet<String>, std::io::Error> {
        use std::io::Read;
        let mut rdr = BufReader::new(File::open(path)?);
        let mut buf = Default::default();
        rdr.read_to_string(&mut buf);
        Ok(buf
            .split_whitespace()
            .map(|s| s.into())
            .collect::<std::collections::HashSet<String>>())
    }
}

pub mod find {
    use std::{fs, path::PathBuf};

    pub fn search_paths() -> Vec<PathBuf> {
        let mut out: Vec<_> = Default::default();
        for mut path in vec![
            std::env::current_exe()
                .expect("Program has location")
                .parent()
                .unwrap()
                .to_owned(),
            std::env::current_dir().expect("Has a current working dir"),
        ] {
            out.push(path.clone());
            path.push("corpus");
            out.push(path);
        }

        out
    }

    fn get_rel_paths(path: PathBuf, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        fs::read_dir(path).map(|rdr| {
            {
                rdr.into_iter()
                    .flat_map(|entry| -> Result<PathBuf, std::io::Error> {
                        let entry = entry?;
                        entry.path().canonicalize()
                    })
                    .filter(|p| match p.extension() {
                        Some(ext) if ext == extension => true,
                        _ => false,
                    })
            }
            .collect::<Vec<_>>()
        })
    }

    pub fn available_files(extension: &str) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Default::default();

        for path in search_paths() {
            if let Ok(mut local) = get_rel_paths(path, extension) {
                paths.append(&mut local);
            }
        }

        paths
    }
}
