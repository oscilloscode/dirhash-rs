use std::fmt::Display;

use crate::error::Result;

#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct HashTableEntry {
    hash: [u8; 32],
    path: String,
}

impl HashTableEntry {
    pub fn new<P, H>(hash: H, path: P) -> Result<Self>
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

impl Display for HashTableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  {}", hex::encode(self.hash), self.path)
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

    pub fn append(&mut self, entries: &mut Vec<HashTableEntry>) {
        self.entries.append(entries);
    }

    pub fn sort(&mut self) {
        self.entries.sort();
    }
}

// TODO: Check which implementation is more performant
impl Display for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.entries
                .iter()
                .map(|e| e.to_string() + "\n")
                .collect::<String>()
        )
    }

    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     for entry in &self.entries {
    //         writeln!(f, "{}", entry)?
    //     }
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hashtableentry() {
        let hte = HashTableEntry::new([0; 32], String::from("/some/path"))
            .expect("Can't create HashTableEntry");
        assert_eq!(hte.hash, [0; 32]);
        assert_eq!(hte.path, "/some/path");
    }

    #[test]
    fn new_hashtableentry_wrong_hash_too_short() {
        let err = HashTableEntry::new([0; 31], String::from("/some/path")).unwrap_err();
        assert!(matches!(err, crate::error::DirHashError::HashTableEntry(_)));
    }

    #[test]
    fn new_hashtableentry_wrong_hash_too_long() {
        let err = HashTableEntry::new([0; 33], String::from("/some/path")).unwrap_err();
        assert!(matches!(err, crate::error::DirHashError::HashTableEntry(_)));
    }

    #[test]
    fn new_hashtable() {
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

        let mut v = vec![
            HashTableEntry::new([0; 32], String::from("/path0")).unwrap(),
            HashTableEntry::new([1; 32], String::from("/path1")).unwrap(),
        ];
        ht.append(&mut v);

        assert_eq!(ht.entries.len(), 2);
        assert_eq!(ht.entries[0].path, "/path0");
        assert_eq!(ht.entries[0].hash, [0; 32]);
        assert_eq!(ht.entries[1].path, "/path1");
        assert_eq!(ht.entries[1].hash, [1; 32]);

        let mut v = vec![
            HashTableEntry::new([2; 32], String::from("/path2")).unwrap(),
            HashTableEntry::new([3; 32], String::from("/path3")).unwrap(),
        ];
        ht.append(&mut v);

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
        let mut v = vec![
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
        ht.append(&mut v);
        ht.sort();

        assert_eq!(ht.entries[0].path, "/zero");
        assert_eq!(ht.entries[1].path, "/one");
        assert_eq!(ht.entries[2].path, "/nine");
        assert_eq!(ht.entries[3].path, "/a");
        assert_eq!(ht.entries[4].path, "/f");
    }

    #[test]
    fn sort_hash_last_byte() {
        let mut v = vec![
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
        ht.append(&mut v);
        ht.sort();

        assert_eq!(ht.entries[0].path, "/two");
        assert_eq!(ht.entries[1].path, "/seven");
        assert_eq!(ht.entries[2].path, "/d");
    }

    #[test]
    fn sort_hash_path() {
        let mut v: Vec<HashTableEntry> = vec![
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
        ht.append(&mut v);
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

    #[test]
    fn display_hashtableentry() {
        let entry = HashTableEntry::new([2; 32], String::from("/some/path"))
            .expect("Can't create HashTableEntry");
        assert_eq!(
            entry.to_string(),
            "0202020202020202020202020202020202020202020202020202020202020202  /some/path"
        );

        let entry = HashTableEntry::new([200; 32], String::from("/some/path"))
            .expect("Can't create HashTableEntry");
        assert_eq!(
            entry.to_string(),
            "c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8  /some/path"
        );
    }

    #[test]
    fn display_hashtable() {
        let mut ht = HashTable::new();

        let mut v = vec![
            HashTableEntry::new([22; 32], String::from("/path0")).unwrap(),
            HashTableEntry::new([255; 32], String::from("/path1")).unwrap(),
            HashTableEntry::new([74; 32], String::from("/path2")).unwrap(),
            HashTableEntry::new([88; 32], String::from("/path3")).unwrap(),
        ];
        ht.append(&mut v);

        print!("{}", ht.to_string());

        assert_eq!(
            ht.to_string(),
            "1616161616161616161616161616161616161616161616161616161616161616  /path0\n\
             ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  /path1\n\
             4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a  /path2\n\
             5858585858585858585858585858585858585858585858585858585858585858  /path3\n"
        );
    }
}
