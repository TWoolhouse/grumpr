use itertools::Itertools;

use super::*;

#[test]
fn single() {
    let mut trie: Trie<String, _> = Trie::new();
    let key = "abcde";
    let value = 42;
    trie.insert(key, value);
    assert_eq!(trie._get(key), Some(&value));
}

#[test]
fn multiple_unique() {
    let mut trie: Trie<String, _> = Trie::new();
    let keys = ["abc", "def", "ghi"];
    let values = [1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key, *value);
    }

    for (&key, value) in keys.iter().zip(values.iter()) {
        assert_eq!(trie._get(key), Some(value));
    }
}

#[test]
fn multiple_overlapping() {
    let mut trie: Trie<String, _> = Trie::new();
    let keys = ["abc", "abcd", "abcde"];
    let values = [1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key, *value);
    }

    for (&key, value) in keys.iter().zip(values.iter()) {
        assert_eq!(trie._get(key), Some(value));
    }
}

#[test]
fn iter_bytes() {
    let mut trie: Trie<String, _> = Trie::new();
    let keys = ["abc", "def", "ghi"];
    let values = [1, 2, 3];

    for (&key, value) in keys.iter().zip(values.iter()) {
        trie.insert(key, *value);
    }

    let found_bytes = trie.bytes().map(|(byte, _)| byte).sorted().collect_vec();
    let expected_bytes: Vec<u8> = keys
        .iter()
        .map(|key| key.as_bytes().into_iter().next().unwrap())
        .sorted()
        .collect_vec();

    assert_eq!(found_bytes, expected_bytes);
}
