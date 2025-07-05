use itertools::Itertools;

use super::*;

#[test]
fn single() {
    let mut trie = Trie::new();
    let key = "abcde".to_owned();
    let value = 42;
    trie.insert(key.to_owned(), value);
    assert_eq!(trie.get(&key), Some(&value));
}

#[test]
fn multiple_unique() {
    let mut trie = Trie::new();
    let keys = vec!["abc", "def", "ghi"];
    let values = vec![1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key.to_owned(), *value);
    }

    for (&key, value) in keys.iter().zip(values.iter()) {
        assert_eq!(trie.get(key), Some(value));
    }
}

#[test]
fn multiple_overlapping() {
    let mut trie = Trie::new();
    let keys = vec!["abc", "abcd", "abcde"];
    let values = vec![1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key.to_owned(), *value);
    }

    for (&key, value) in keys.iter().zip(values.iter()) {
        assert_eq!(trie.get(key), Some(value));
    }
}

#[test]
fn iter_all() {
    let mut trie = Trie::new();
    let keys = vec!["abc", "def", "ghi"];
    let values = vec![1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key.to_owned(), *value);
    }

    assert_eq!(
        &trie
            .leaves()
            .map(|leaflet| *leaflet.value())
            .sorted()
            .collect::<Vec<_>>(),
        &[1, 2, 3]
    );
}
