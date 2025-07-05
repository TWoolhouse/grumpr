use super::Key;

pub(super) trait KeyNibbles: Key {
    fn as_nibbles(&self) -> impl IntoIterator<Item = u8> {
        const _: () = assert!(
            (1 << (size_of::<u8>() * 8 - 4)) <= super::CHILDREN,
            "Segment nibbles must fit within the number of children in the Trie"
        );
        self.as_bytes()
            .into_iter()
            .flat_map(|byte| [byte >> 4, byte & 0x0F])
    }
}

impl<S> KeyNibbles for S where S: Key + ?Sized {}

impl Key for String {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.bytes()
    }
}

impl Key for str {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.bytes()
    }
}
