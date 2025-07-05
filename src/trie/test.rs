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
fn iter_bytes() {
    let mut trie = Trie::new();
    let keys = vec!["abc", "def", "ghi"];
    let values = vec![1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key.to_owned(), *value);
    }

    let found_bytes = trie.bytes().map(|(byte, _)| byte).sorted().collect_vec();
    let expected_bytes: Vec<u8> = keys
        .iter()
        .map(|key| key.as_bytes()[0])
        .sorted()
        .collect_vec();

    assert_eq!(found_bytes, expected_bytes);
}
