//! Test list:
//! - Define and test what happens with sorting two files have the same hash (i.e., same content).
//!   Probably use the paths to define order.
//! - Add tests to check that sort() behaves as expected (both for the hash and the path)
//!

use std::fmt::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::pathhash::PathHashProvider;

#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PathHashList<T> {
    pathhashvec: Vec<T>,
    hash: Option<[u8; 32]>,
}

impl<T> PathHashList<T>
where
    T: PathHashProvider,
{
    pub fn new(files: Vec<T>) -> Result<Self, std::io::Error> {
        // Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        Ok(PathHashList {
            pathhashvec: files,
            hash: None,
        })
    }

    pub fn hash(&self) -> Option<&[u8; 32]> {
        self.hash.as_ref()
    }

    /// Computes hash of all PathHashs.
    ///
    /// TODO:
    /// Either test both implementations and keep them both, or benchmark them, select one, and only
    /// use that one!
    pub fn compute_hash(&mut self) -> Result<(), std::io::Error> {
        self.compute_hash_with_update()
        // self.compute_hash_with_string()
    }

    /// Computes hash of all PathHashs.
    ///
    /// This version hashes the string representation of a PathHash immediately by calling the
    /// `update()` method repeatedly.
    pub fn compute_hash_with_update(&mut self) -> Result<(), std::io::Error> {
        let mut hashable_data_vec = self.get_hashable_data_vec()?;

        sort_hashable_data_vec(&mut hashable_data_vec);

        let mut hasher = Sha256::new();
        let mut hashable_string = String::new();

        for (hash, path) in hashable_data_vec {
            hashable_string.clear();
            let _ = writeln!(
                &mut hashable_string,
                "{}  {}",
                hex::encode(hash),
                path.to_string_lossy()
            );

            hasher.update(&hashable_string);
        }

        let hash = hasher.finalize();
        self.hash = Some(hash.into());

        Ok(())
    }

    /// Computes hash of all PathHashs.
    ///
    /// This version puts everything into a single string which is then hashed in one go.
    pub fn compute_hash_with_string(&mut self) -> Result<(), std::io::Error> {
        let mut hashable_data_vec = self.get_hashable_data_vec()?;

        sort_hashable_data_vec(&mut hashable_data_vec);

        let mut hashable_string = String::new();

        for (hash, path) in hashable_data_vec {
            let _ = writeln!(
                &mut hashable_string,
                "{}  {}",
                hex::encode(hash),
                path.to_string_lossy()
            );
        }

        let hash = Sha256::digest(hashable_string);
        self.hash = Some(hash.into());

        Ok(())
    }

    fn get_hashable_data_vec(&mut self) -> Result<Vec<([u8; 32], PathBuf)>, std::io::Error> {
        let mut hashable_data_vec: Vec<([u8; 32], PathBuf)> =
            Vec::with_capacity(self.pathhashvec.len());

        for pb in &mut self.pathhashvec {
            if pb.hash().is_none() {
                pb.compute_hash()?;
            }
            hashable_data_vec.push((*pb.hash().unwrap(), pb.path().to_owned()));
        }

        Ok(hashable_data_vec)
    }
}

// Exposed for testing.
fn sort_hashable_data_vec(vec: &mut Vec<([u8; 32], PathBuf)>) {
    vec.sort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pathhash::pathhashspy::PathHashSpy;

    #[test]
    fn pathhashlist_hash_is_none_after_init() {
        let spies: Vec<PathHashSpy> = vec![];
        let pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");
        assert!(pathhashlist.hash.is_none());
    }

    #[test]
    fn pathhashlist_hash_accessor() {
        let spies: Vec<PathHashSpy> = vec![];
        let mut pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");
        assert!(pathhashlist.hash().is_none());
        pathhashlist.hash = Some(*b"01234567890123456789012345678901");
        assert!(pathhashlist.hash().is_some());
        assert_eq!(pathhashlist.hash().unwrap()[7], 0x37);
    }

    #[test]
    fn pathhashlist_compute_hash() {
        let spies = vec![
            PathHashSpy::new(
                Path::new("/some/path").to_owned(),
                Some(*b"\xd8\x3b\xa8\x04\x20\xec\x99\xbc\xb1\x43\xdf\x16\xa0\x0c\x39\xa5\x6c\x14\x03\x41\xe4\x44\x6a\xe9\xb5\xe8\xb5\xa6\xd1\x81\x16\xed"), // hash of "/some/path"
                None,
            ),
            PathHashSpy::new(
                Path::new("/other/path").to_owned(),
                Some(*b"\x59\xea\xd6\x2a\x5f\x16\xe4\xee\x2f\x7d\xe8\x9e\x52\xf9\x78\xd6\xf1\x5e\x97\xf3\x87\x25\x5d\xd7\x7e\xd3\xc7\x2f\x88\x88\x28\x55"), // hash of "/other/path"
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        assert_eq!(pathhashlist.pathhashvec[0].call_count_compute_hash(), 0);
        assert_eq!(pathhashlist.pathhashvec[1].call_count_compute_hash(), 0);

        // Hash of (the newline after the second line is also part of the digest):
        // 59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path
        // d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path
        //
        // -> 4dcf91beae7c9fcc68df4f57ab4344a744e7d0c326003a03e7996f87fe451390
        assert_eq!(pathhashlist.hash().unwrap(), b"\x4d\xcf\x91\xbe\xae\x7c\x9f\xcc\x68\xdf\x4f\x57\xab\x43\x44\xa7\x44\xe7\xd0\xc3\x26\x00\x3a\x03\xe7\x99\x6f\x87\xfe\x45\x13\x90");
    }

    #[test]
    fn pathhashlist_compute_hash_computes_underlying_hash_only_when_necessary() {
        let spies = vec![
            PathHashSpy::new(
                Path::new("/some/path").to_owned(),
                None,
                Some(*b"\xd8\x3b\xa8\x04\x20\xec\x99\xbc\xb1\x43\xdf\x16\xa0\x0c\x39\xa5\x6c\x14\x03\x41\xe4\x44\x6a\xe9\xb5\xe8\xb5\xa6\xd1\x81\x16\xed"), // hash of "/some/path"
            ),
            PathHashSpy::new(
                Path::new("/other/path").to_owned(),
                Some(*b"\x59\xea\xd6\x2a\x5f\x16\xe4\xee\x2f\x7d\xe8\x9e\x52\xf9\x78\xd6\xf1\x5e\x97\xf3\x87\x25\x5d\xd7\x7e\xd3\xc7\x2f\x88\x88\x28\x55"), // hash of "/other/path"
                None,
            ),
        ];
        let mut pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        assert_eq!(pathhashlist.pathhashvec[0].call_count_compute_hash(), 1);
        assert_eq!(pathhashlist.pathhashvec[1].call_count_compute_hash(), 0);

        // Hash of (the newline after the second line is also part of the digest):
        // 59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path
        // d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path
        //
        // -> 4dcf91beae7c9fcc68df4f57ab4344a744e7d0c326003a03e7996f87fe451390
        assert_eq!(pathhashlist.hash().unwrap(), b"\x4d\xcf\x91\xbe\xae\x7c\x9f\xcc\x68\xdf\x4f\x57\xab\x43\x44\xa7\x44\xe7\xd0\xc3\x26\x00\x3a\x03\xe7\x99\x6f\x87\xfe\x45\x13\x90");
    }

    #[test]
    fn pathhashlist_compute_hash_no_files() {
        let spies: Vec<PathHashSpy> = vec![];
        let mut pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");

        assert!(pathhashlist.compute_hash().is_ok());

        // Hash of nothing at all (not even a newline):
        //
        // -> e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(pathhashlist.hash().unwrap(), b"\xe3\xb0\xc4\x42\x98\xfc\x1c\x14\x9a\xfb\xf4\xc8\x99\x6f\xb9\x24\x27\xae\x41\xe4\x64\x9b\x93\x4c\xa4\x95\x99\x1b\x78\x52\xb8\x55");
    }
}
