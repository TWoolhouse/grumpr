use crate::librarian::Seed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub(super) seeds: Vec<Seed>,
}

impl FromIterator<(String, u64)> for Library {
    fn from_iter<T: IntoIterator<Item = (String, u64)>>(iter: T) -> Self {
        let mut seeds = Vec::new();

        for (index, (root, count)) in iter.into_iter().enumerate() {
            seeds.push(Seed { root, index, count });
        }

        Library { seeds }
    }
}
