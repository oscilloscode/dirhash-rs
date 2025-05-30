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

    pub fn sort(&mut self) {
        self.entries.sort();
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

    #[test]
    fn sort_hash_first_byte() {
        let v = vec![
            HashTableEntry::new(
                [
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                String::from("/one"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    0xF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                String::from("/f"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                String::from("/nine"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    0xA, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                String::from("/a"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                String::from("/zero"),
            )
            .unwrap(),
        ];

        let mut ht = HashTable::new();
        ht.append(v);
        ht.sort();

        assert_eq!(ht.entries[0].path, "/zero");
        assert_eq!(ht.entries[1].path, "/one");
        assert_eq!(ht.entries[2].path, "/nine");
        assert_eq!(ht.entries[3].path, "/a");
        assert_eq!(ht.entries[4].path, "/f");
    }

    #[test]
    fn sort_hash_last_byte() {
        let v = vec![
            HashTableEntry::new(
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 7,
                ],
                String::from("/seven"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0xD,
                ],
                String::from("/d"),
            )
            .unwrap(),
            HashTableEntry::new(
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 2,
                ],
                String::from("/two"),
            )
            .unwrap(),
        ];

        let mut ht = HashTable::new();
        ht.append(v);
        ht.sort();

        assert_eq!(ht.entries[0].path, "/two");
        assert_eq!(ht.entries[1].path, "/seven");
        assert_eq!(ht.entries[2].path, "/d");
    }

    #[test]
    fn sort_hash_path() {
        let v: Vec<HashTableEntry> = vec![
            HashTableEntry::new([0; 32], String::from("ä_umlaut")).unwrap(),
            HashTableEntry::new([0; 32], String::from("8")).unwrap(),
            HashTableEntry::new([0; 32], String::from("\\backslash")).unwrap(),
            HashTableEntry::new([0; 32], String::from("\"quote")).unwrap(),
            HashTableEntry::new([0; 32], String::from("?question mark")).unwrap(),
            HashTableEntry::new([0; 32], String::from("T")).unwrap(),
            HashTableEntry::new([0; 32], String::from("_underscore")).unwrap(),
            HashTableEntry::new([0; 32], String::from("7")).unwrap(),
            HashTableEntry::new([0; 32], String::from("a")).unwrap(),
            HashTableEntry::new([0; 32], String::from("(parens)")).unwrap(),
            HashTableEntry::new([0; 32], String::from("|pipe")).unwrap(),
            HashTableEntry::new([0; 32], String::from("*asterisk")).unwrap(),
            HashTableEntry::new([0; 32], String::from("-hyphen")).unwrap(),
            HashTableEntry::new([0; 32], String::from("~tilde")).unwrap(),
            HashTableEntry::new([0; 32], String::from("<angle brackets>")).unwrap(),
            HashTableEntry::new([0; 32], String::from("{braces}")).unwrap(),
            HashTableEntry::new([0; 32], String::from("[brackets]")).unwrap(),
            HashTableEntry::new([0; 32], String::from("d")).unwrap(),
            HashTableEntry::new([0; 32], String::from("B")).unwrap(),
        ];

        let mut ht = HashTable::new();
        ht.append(v);
        ht.sort();

        assert_eq!(ht.entries[0].path, "\"quote");
        assert_eq!(ht.entries[1].path, "(parens)");
        assert_eq!(ht.entries[2].path, "*asterisk");
        assert_eq!(ht.entries[3].path, "-hyphen");
        assert_eq!(ht.entries[4].path, "7");
        assert_eq!(ht.entries[5].path, "8");
        assert_eq!(ht.entries[6].path, "<angle brackets>");
        assert_eq!(ht.entries[7].path, "?question mark");
        assert_eq!(ht.entries[8].path, "B");
        assert_eq!(ht.entries[9].path, "T");
        assert_eq!(ht.entries[10].path, "[brackets]");
        assert_eq!(ht.entries[11].path, "\\backslash");
        assert_eq!(ht.entries[12].path, "_underscore");
        assert_eq!(ht.entries[13].path, "a");
        assert_eq!(ht.entries[14].path, "d");
        assert_eq!(ht.entries[15].path, "{braces}");
        assert_eq!(ht.entries[16].path, "|pipe");
        assert_eq!(ht.entries[17].path, "~tilde");
        assert_eq!(ht.entries[18].path, "ä_umlaut");
    }
}
