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

    librarian.iter();

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
    let query = QuerySearch::new("librarian");
    let results = librarian.search(query);
    assert_eq!(results.len(), 1);
    assert_eq!(
        results.iter().next().unwrap().word().unwrap(),
        &library.seeds[2]
    );

    // Search for a sequence
    let query = QuerySearch::new("helloworld").repeating(1);
    let results = librarian.search(query);
    assert_eq!(results.len(), 1);
    assert_eq!(results.iter().next().unwrap().sequence().unwrap().len(), 2);
}

#[test]
fn search_lvl1() {
    let dataset = dataset();
    let library = library_from_dataset(dataset.iter().copied());
    let librarian = Librarian::from(&library);

    let librarian = librarian.search(QuerySearch::new(".").repeating(1));
    assert_eq!(librarian.len(), dataset.len() + dataset.len().pow(2));

    // Search for a word
    let query = QuerySearch::new("librarianworld");
    let results = librarian.search(query);
    assert_eq!(results.len(), 1);
    assert_eq!(results.iter().next().unwrap().sequence().unwrap().len(), 2);

    // Search for a sequence
    let query = QuerySearch::new("helloworld").repeating(1);
    let results = librarian.search(query);
    assert!(results.len() > 1);
    assert!(results.iter().next().unwrap().sequence().unwrap().len() >= 2);
}
