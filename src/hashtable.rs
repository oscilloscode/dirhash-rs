use std::array::TryFromSliceError;

#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct HashTableEntry {
    hash: [u8; 32],
    path: String,
}

impl HashTableEntry {
    pub fn new<P, H>(hash: H, path: P) -> Result<Self, TryFromSliceError>
    where
        P: Into<String>,
        H: AsRef<[u8]>,
    {
        Ok(Self {
            hash: hash.as_ref().try_into()?,
            path: path.into(),
        })
    }
}

#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct HashTable {
    entries: Vec<HashTableEntry>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: HashTableEntry) {
        self.entries.push(entry);
    }

    pub fn append(&mut self, mut entries: Vec<HashTableEntry>) {
        self.entries.append(&mut entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let ht = HashTable::new();
        assert!(ht.entries.is_empty());
    }

    #[test]
    fn add() {
        let mut ht = HashTable::new();
        assert!(ht.entries.is_empty());

        let entry = HashTableEntry::new([0; 32], String::from("/some/path"))
            .expect("Can't create HashTableEntry");
        ht.add(entry);
        assert!(!ht.entries.is_empty());
        assert_eq!(ht.entries[0].path, "/some/path");
        assert_eq!(ht.entries[0].hash, [0; 32]);

        let entry = HashTableEntry::new([1; 32], String::from("/other/path"))
            .expect("Can't create HashTableEntry");
        ht.add(entry);
        assert!(!ht.entries.is_empty());
        assert_eq!(ht.entries[1].path, "/other/path");
        assert_eq!(ht.entries[1].hash, [1; 32]);
    }

    #[test]
    fn append() {
        let mut ht = HashTable::new();
        assert!(ht.entries.is_empty());

        let v = vec![
            HashTableEntry::new([0; 32], String::from("/path0")).unwrap(),
            HashTableEntry::new([1; 32], String::from("/path1")).unwrap(),
        ];
        ht.append(v);

        assert_eq!(ht.entries.len(), 2);
        assert_eq!(ht.entries[0].path, "/path0");
        assert_eq!(ht.entries[0].hash, [0; 32]);
        assert_eq!(ht.entries[1].path, "/path1");
        assert_eq!(ht.entries[1].hash, [1; 32]);

        let v = vec![
            HashTableEntry::new([2; 32], String::from("/path2")).unwrap(),
            HashTableEntry::new([3; 32], String::from("/path3")).unwrap(),
        ];
        ht.append(v);

        assert_eq!(ht.entries.len(), 4);
        assert_eq!(ht.entries[0].path, "/path0");
        assert_eq!(ht.entries[0].hash, [0; 32]);
        assert_eq!(ht.entries[1].path, "/path1");
        assert_eq!(ht.entries[1].hash, [1; 32]);
        assert_eq!(ht.entries[2].path, "/path2");
        assert_eq!(ht.entries[2].hash, [2; 32]);
        assert_eq!(ht.entries[3].path, "/path3");
        assert_eq!(ht.entries[3].hash, [3; 32]);
    }
}
