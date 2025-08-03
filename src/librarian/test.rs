use super::*;

fn dataset() -> Vec<&'static str> {
    vec![
        "hello",
        "world",
        "librarian",
        "gram",
        "rust",
        "regex",
        "search",
        "test",
        "seed",
        "library",
        "pear",
        "pears",
        "spear",
    ]
}

fn library_from_dataset<'a>(it: impl IntoIterator<Item = &'a str>) -> Library {
    Library::from_iter(
        it.into_iter()
            .enumerate()
            .map(|(i, s)| (s.to_owned(), i as u64)),
    )
}

#[test]
fn make_library() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    assert_eq!(library.seeds.len(), dataset.len());

    // The base librarian should have the same number of grams as the library seeds
    let librarian = Librarian::from(&library);
    assert_eq!(librarian.len(), dataset.len());
}

#[test]
fn librarian_iter() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    for (i, gram) in librarian.iter().enumerate() {
        assert!(gram.word().is_some());
        assert_eq!(gram.word().unwrap(), &library.seeds[i]);
    }

    for (i, gram) in librarian.into_iter().enumerate() {
        assert!(gram.word().is_some());
        assert_eq!(gram.word().unwrap(), &library.seeds[i]);
    }
}

#[test]
fn search_lvl0() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    // Search for a word
    let query = query::Match::new("librarian");
    let results = librarian.search(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results.iter().next().unwrap().word().unwrap(),
        &library.seeds[2]
    );

    // Search for a sequence
    let query = query::Match::new("helloworld").depth(1);
    let results = librarian.search(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results.iter().next().unwrap().sequence().unwrap().len(), 2);
}

#[test]
fn search_lvl1() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    let librarian = librarian.search(&query::Match::new(".").depth(1)).unwrap();
    assert_eq!(librarian.len(), dataset.len() + dataset.len().pow(2));

    // Search for a word
    let query = query::Match::new("librarianworld");
    let results = librarian.search(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results.iter().next().unwrap().sequence().unwrap().len(), 2);

    // Search for a sequence
    let query = query::Match::new("helloworld").depth(1);
    let results = librarian.search(&query).unwrap();
    assert!(results.len() > 1);
    assert!(results.iter().next().unwrap().sequence().unwrap().len() >= 2);
}

#[test]
fn anagrams() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    // Search for anagrams
    let query = query::Anagram::new("stur");
    let results = librarian.anagrams(&query).unwrap();
    assert_eq!(results.len(), 1);
    for gram in results {
        assert!(gram.word().is_some());
        assert_eq!(gram.word().unwrap().root, "rust");
    }

    let query = query::Anagram::new("reap");
    let results = librarian.anagrams(&query).unwrap();
    assert_eq!(results.len(), 1);
    for gram in results {
        assert!(gram.word().is_some());
        assert_eq!(gram.word().unwrap().root, "pear");
    }

    let query = query::Anagram::new("pears").partial(true);
    let results = librarian.anagrams(&query).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn nearest() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    // Search for nearest words
    let query = query::Nearest::new("librar", 5);
    let (results, distance) = librarian.nearest(&query).unwrap();
    assert_eq!(distance, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(
        results.iter().next().unwrap().word().unwrap().root,
        "library"
    );
}

#[test]
fn distance() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    let query = query::Distance::new("librar", [3]);
    let results = librarian.distance(&query).unwrap();
    assert_eq!(results.len(), 2);
    // librarian & library
    // library has a distance of 3 from "librar" as you can add & delete in pairs

    let query = query::Distance::new("librar", [3]).strict(true);
    let results = librarian.distance(&query).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn has() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    // Search for words that have certain characters
    let query = query::Has::new("eex");
    let results = librarian.has(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results.iter().next().unwrap().word().unwrap().root, "regex");
}
