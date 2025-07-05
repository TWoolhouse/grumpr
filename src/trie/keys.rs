use super::{Key, Notch};

pub(super) trait NotchNibbles: Notch {
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

impl<S> NotchNibbles for S where S: Notch {}

impl Key for String {
    type Notch = char;

    fn notches(&self) -> impl IntoIterator<Item = Self::Notch> {
        self.chars()
    }
}

impl Notch for char {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        let mut buffer = [0; 4];
        let len = self.encode_utf8(&mut buffer).len();
        buffer.into_iter().take(len)
    }
}

impl Key for str {
    type Notch = char;

    fn notches(&self) -> impl IntoIterator<Item = Self::Notch> {
        self.chars()
    }
}
