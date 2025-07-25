//! Test list:
//! - Define and test what happens with sorting two files have the same hash (i.e., same content).
//!   Probably use the paths to define order.
//! - Add tests to check that sort() behaves as expected (both for the hash and the path)
//!

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::error::Result;
use crate::hashtable::{HashTable, HashTableEntry};
use crate::pathhash::{PathHash, PathHashProvider};

// TODO:
// Maybe it's better to get rid of getters and just make root and hash public... If root is public,
// could a user of the struct then just reassign the member? Or can I somehow make it public but
// immutable/irreplaceable? Apparently, the borrow checker "prefers" direct access than getters and
// setters.
// https://users.rust-lang.org/t/best-practices-on-setters-and-accessor-methods-in-general/66530
#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PathHashList<T> {
    root: Option<PathBuf>,
    pathhashvec: Vec<T>,
    hash: Option<[u8; 32]>,
    hashtable: Option<HashTable>,
}

impl<T> PathHashList<T>
where
    T: PathHashProvider,
{
    pub fn new(files: Vec<T>, root: Option<&Path>) -> Result<Self> {
        Ok(PathHashList {
            root: root.map(|p| p.to_owned()),
            pathhashvec: files,
            hash: None,
            hashtable: None,
        })
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_ref().map(|p| p.as_path())
    }

    pub fn hash(&self) -> Option<&[u8; 32]> {
        self.hash.as_ref()
    }

    /// Computes hash of all PathHashs.
    ///
    pub fn compute_hash(&mut self) -> Result<()> {
        let mut ht = HashTable::new();

        for pb in &mut self.pathhashvec {
            if pb.hash().is_none() {
                pb.compute_hash()?;
            }

            let maybe_stripped_path = match &self.root {
                Some(root) => Cow::from("./") + pb.path().strip_prefix(root)?.to_string_lossy(),
                None => pb.path().to_string_lossy(),
            };

            ht.add(
                HashTableEntry::new(pb.hash().unwrap(), maybe_stripped_path)
                    .expect("Can't create HashTableEntry"),
            );
        }

        ht.sort();

        let hash = Sha256::digest(ht.to_string());
        self.hashtable = Some(ht);
        self.hash = Some(hash.into());

        Ok(())
    }
}

impl PathHashList<PathHash> {
    // TODO:
    // - A builder pattern is probably more suitable. This would also allow options like following
    //   symlinks etc. to be more descriptive and idiomatic.
    // - Add bool parameter for absolute paths?
    pub fn from_path_recursive(path: &Path, set_root: bool, follow_symlinks: bool) -> Result<Self> {
        let mut files: Vec<PathHash> = vec![];

        // WalkDir::new(path)
        //     .follow_links(follow_symlinks)
        //     .into_iter()
        //     .filter_map(Result::ok)
        //     .filter(|e| e.file_type().is_file())
        //     .count();

        for entry in WalkDir::new(path).follow_links(follow_symlinks).into_iter() {
            let entry = entry?;
            println!("{:?}", entry);
            // TODO:
            // Or should I just filter for files? How are symlinks affected by this?
            if entry.file_type().is_dir() {
                continue;
            }

            let pathhash = PathHash::new(entry.path())?;
            files.push(pathhash);
        }

        let root = if set_root {
            Some(path.to_owned())
        } else {
            None
        };

        Ok(PathHashList {
            root,
            pathhashvec: files,
            hash: None,
            hashtable: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::DirHashError, pathhash::pathhashspy::PathHashSpy};

    #[test]
    fn new() {
        let spies = vec![
            PathHashSpy::new("/some/path", None, None),
            PathHashSpy::new("/other/path".to_owned(), None, None),
        ];
        let pathhashlist = PathHashList::new(spies, Some(Path::new("/some/path")))
            .expect("Can't create PathHashList");
        assert_eq!(pathhashlist.root.unwrap().to_str().unwrap(), "/some/path");
        assert_eq!(
            pathhashlist.pathhashvec[0].path().to_str().unwrap(),
            "/some/path"
        );
        assert_eq!(
            pathhashlist.pathhashvec[1].path().to_str().unwrap(),
            "/other/path"
        );
    }

    #[test]
    fn root_getter() {
        let spies: Vec<PathHashSpy> = vec![];
        let mut pathhashlist = PathHashList::new(spies, Some(Path::new("/some/path")))
            .expect("Can't create PathHashList");
        assert_eq!(pathhashlist.root().unwrap().to_str().unwrap(), "/some/path");
        pathhashlist.root = None;
        assert!(pathhashlist.root().is_none());
    }

    #[test]
    fn hash_is_none_after_init() {
        let spies: Vec<PathHashSpy> = vec![];
        let pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");
        assert!(pathhashlist.hash.is_none());
    }

    #[test]
    fn hash_getter() {
        let spies: Vec<PathHashSpy> = vec![];
        let mut pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");
        assert!(pathhashlist.hash().is_none());
        pathhashlist.hash = Some(*b"01234567890123456789012345678901");
        assert!(pathhashlist.hash().is_some());
        assert_eq!(pathhashlist.hash().unwrap()[7], 0x37);
    }

    #[test]
    fn hashtable_is_none_after_init() {
        let spies: Vec<PathHashSpy> = vec![];
        let pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");
        assert!(pathhashlist.hashtable.is_none());
    }

    #[test]
    fn compute_hash_no_root() {
        let spies = vec![
            PathHashSpy::new(
                "/some/path",
                Some(*b"\xd8\x3b\xa8\x04\x20\xec\x99\xbc\xb1\x43\xdf\x16\xa0\x0c\x39\xa5\x6c\x14\x03\x41\xe4\x44\x6a\xe9\xb5\xe8\xb5\xa6\xd1\x81\x16\xed"), // hash of "/some/path"
                None,
            ),
            PathHashSpy::new(
                "/other/path",
                Some(*b"\x59\xea\xd6\x2a\x5f\x16\xe4\xee\x2f\x7d\xe8\x9e\x52\xf9\x78\xd6\xf1\x5e\x97\xf3\x87\x25\x5d\xd7\x7e\xd3\xc7\x2f\x88\x88\x28\x55"), // hash of "/other/path"
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        assert_eq!(pathhashlist.pathhashvec[0].call_count_compute_hash(), 0);
        assert_eq!(pathhashlist.pathhashvec[1].call_count_compute_hash(), 0);

        // Hash of (the newline after the second line is also part of the digest):
        // 59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path
        // d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path
        //
        // -> 4dcf91beae7c9fcc68df4f57ab4344a744e7d0c326003a03e7996f87fe451390
        assert_eq!(
            pathhashlist.hashtable.as_ref().unwrap().to_string(),
            "59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path\n\
             d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path\n"
        );
        assert_eq!(pathhashlist.hash().unwrap(), b"\x4d\xcf\x91\xbe\xae\x7c\x9f\xcc\x68\xdf\x4f\x57\xab\x43\x44\xa7\x44\xe7\xd0\xc3\x26\x00\x3a\x03\xe7\x99\x6f\x87\xfe\x45\x13\x90");
    }

    #[test]
    fn compute_hash_with_root() {
        let spies = vec![
            PathHashSpy::new(
                "/pre/fix/some/path",
                Some(*b"\xba\xcb\xe3\xc3\x46\xcb\x5c\xb0\xcf\x30\xdb\x33\xad\xc7\xd4\x10\x49\x36\x44\xaa\xfe\x98\xe0\x8e\x0e\x27\x9b\xb3\x5b\x57\x92\x8a"), // hash of "./some/path"
                None,
            ),
            PathHashSpy::new(
                "/pre/fix/other/path",
                Some(*b"\x62\x09\xe5\xaa\x71\x50\xa1\xc6\xee\x59\x2f\x0a\x7f\x6a\x32\xe1\xcb\x74\x93\x33\xcb\x90\x6a\xbf\xfb\x5e\x65\x5e\x04\x91\xc6\x88"), // hash of "./other/path"
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies, Some(Path::new("/pre/fix")))
            .expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        assert_eq!(pathhashlist.pathhashvec[0].call_count_compute_hash(), 0);
        assert_eq!(pathhashlist.pathhashvec[1].call_count_compute_hash(), 0);

        // Hash of (the newline after the second line is also part of the digest):
        //
        // 6209e5aa7150a1c6ee592f0a7f6a32e1cb749333cb906abffb5e655e0491c688  ./other/path
        // bacbe3c346cb5cb0cf30db33adc7d410493644aafe98e08e0e279bb35b57928a  ./some/path
        //
        // -> 13f9a9ba4a18685d46498d4ac27f02ac0c70c8afe14220266032765633c39933
        assert_eq!(
            pathhashlist.hashtable.as_ref().unwrap().to_string(),
            "6209e5aa7150a1c6ee592f0a7f6a32e1cb749333cb906abffb5e655e0491c688  ./other/path\n\
             bacbe3c346cb5cb0cf30db33adc7d410493644aafe98e08e0e279bb35b57928a  ./some/path\n"
        );
        assert_eq!(
            pathhashlist.hash().unwrap(),
            b"\x13\xf9\xa9\xba\x4a\x18\x68\x5d\x46\x49\x8d\x4a\xc2\x7f\x02\xac\x0c\x70\xc8\xaf\xe1\x42\x20\x26\x60\x32\x76\x56\x33\xc3\x99\x33"
        );
    }

    #[test]
    fn compute_hash_with_mismatched_root() {
        let spies = vec![
            PathHashSpy::new(
                "/pre/fix/some/path",
                Some(*b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                None,
            ),
            PathHashSpy::new(
                "/pre/fix/other/path",
                Some(*b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies, Some(Path::new("/not/prefix")))
            .expect("Can't create PathHashList");

        let err = pathhashlist.compute_hash().unwrap_err();
        assert!(matches!(err, DirHashError::RootMismatch(_)));

        assert!(pathhashlist.hashtable.is_none());
        assert!(pathhashlist.hash.is_none());
    }

    #[test]
    fn compute_hash_computes_underlying_hash_only_when_necessary() {
        let spies = vec![
            PathHashSpy::new(
                "/some/path",
                None,
                Some(*b"\xd8\x3b\xa8\x04\x20\xec\x99\xbc\xb1\x43\xdf\x16\xa0\x0c\x39\xa5\x6c\x14\x03\x41\xe4\x44\x6a\xe9\xb5\xe8\xb5\xa6\xd1\x81\x16\xed"), // hash of "/some/path"
            ),
            PathHashSpy::new(
                "/other/path",
                Some(*b"\x59\xea\xd6\x2a\x5f\x16\xe4\xee\x2f\x7d\xe8\x9e\x52\xf9\x78\xd6\xf1\x5e\x97\xf3\x87\x25\x5d\xd7\x7e\xd3\xc7\x2f\x88\x88\x28\x55"), // hash of "/other/path"
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        assert_eq!(pathhashlist.pathhashvec[0].call_count_compute_hash(), 1);
        assert_eq!(pathhashlist.pathhashvec[1].call_count_compute_hash(), 0);

        // Hash of (the newline after the second line is also part of the digest):
        // 59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path
        // d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path
        //
        // -> 4dcf91beae7c9fcc68df4f57ab4344a744e7d0c326003a03e7996f87fe451390
        assert_eq!(
            pathhashlist.hashtable.as_ref().unwrap().to_string(),
            "59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path\n\
             d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path\n"
        );
        assert_eq!(pathhashlist.hash().unwrap(), b"\x4d\xcf\x91\xbe\xae\x7c\x9f\xcc\x68\xdf\x4f\x57\xab\x43\x44\xa7\x44\xe7\xd0\xc3\x26\x00\x3a\x03\xe7\x99\x6f\x87\xfe\x45\x13\x90");
    }

    #[test]
    fn compute_hash_no_files() {
        let spies: Vec<PathHashSpy> = vec![];
        let mut pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        // Hash of nothing at all (not even a newline):
        //
        // -> e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(pathhashlist.hashtable.as_ref().unwrap().to_string(), "");
        assert_eq!(pathhashlist.hash().unwrap(), b"\xe3\xb0\xc4\x42\x98\xfc\x1c\x14\x9a\xfb\xf4\xc8\x99\x6f\xb9\x24\x27\xae\x41\xe4\x64\x9b\x93\x4c\xa4\x95\x99\x1b\x78\x52\xb8\x55");
    }
}
