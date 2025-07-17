use super::Key;

const NIBBLE_BITS: usize = (u8::BITS / 2) as usize;

pub(super) trait KeyNibbles: Key {
    fn as_nibbles(&self) -> impl IntoIterator<Item = u8> {
        const _: () = assert!(
            (1 << (size_of::<u8>() * u8::BITS as usize - NIBBLE_BITS)) <= super::CHILDREN,
            "Segment nibbles must fit within the number of children in the Trie"
        );
        self.as_bytes()
            .into_iter()
            .flat_map(|byte| [byte >> NIBBLE_BITS, byte & ((1 << NIBBLE_BITS) - 1) as u8])
    }
}

impl<S> KeyNibbles for S where S: Key + ?Sized {}

impl Key for String {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.bytes()
    }
}

impl Key for &str {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.bytes()
    }
}

impl Key for str {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.bytes()
    }
}
