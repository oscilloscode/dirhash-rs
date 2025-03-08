//! Test list:
//! - Other filetypes (links, char device, block dev, socket, pipe)
//!

use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

// TODO: Rename this!!
pub trait PathHashProvider {
    fn path(&self) -> &Path;
    fn hash(&self) -> Option<&[u8; 32]>;
    fn compute_hash(&mut self) -> Result<(), std::io::Error>;
}

/// Struct containing a path and hash from a file on the filesystem.
#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PathHash {
    path: PathBuf,
    hash: Option<[u8; 32]>,
}

impl PathHash {
    /// Creates a [`PathHash`] from a path to a file on the system.
    /// Returns an [`std::io::Error`] if the file doesn't exist because the path gets canonicalized
    /// (which fails if the file doesn't exist).
    pub fn new(path: impl AsRef<Path>) -> Self {
        PathHash {
            path: path.as_ref().to_owned(),
            hash: Default::default(),
        }
    }
}

impl PathHashProvider for PathHash {
    /// Computes the SHA256 hash of the contents of the corresponding file and stores it. Calling
    /// this method again will reread the file and recompute the hash value.
    fn compute_hash(&mut self) -> Result<(), std::io::Error> {
        let data = fs::read(&self.path)?;
        let hash = Sha256::digest(data);
        self.hash = Some(hash.into());
        Ok(())
    }

    /// Returns the stored hash of the file contents. If `None`, use [`Self::compute_hash()`] to compute the
    /// hash value.
    fn hash(&self) -> Option<&[u8; 32]> {
        self.hash.as_ref()
    }

    /// Returns the stored path.
    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, Seek, Write};
    use std::sync::OnceLock;

    use super::*;
    use tempfile::NamedTempFile;

    #[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
    struct TestVector {
        content: String,
        hash: [u8; 32],
    }

    #[derive(Debug)]
    struct TestFile {
        file: NamedTempFile,
        test_vector: TestVector,
    }

    #[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
    enum TestFileContent {
        Empty,
        SingleLine,
        MultiLine,
    }

    fn get_testvector_hashmap() -> &'static HashMap<TestFileContent, TestVector> {
        static HASHMAP: OnceLock<HashMap<TestFileContent, TestVector>> = OnceLock::new();
        HASHMAP.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(
                TestFileContent::Empty,
                TestVector {
                    content: String::from(""),
                    hash: *b"\xe3\xb0\xc4\x42\x98\xfc\x1c\x14\x9a\xfb\xf4\xc8\x99\x6f\xb9\x24\x27\xae\x41\xe4\x64\x9b\x93\x4c\xa4\x95\x99\x1b\x78\x52\xb8\x55",
                },
            );
            m.insert(
                TestFileContent::SingleLine,
                TestVector {
                    content: String::from("First line"),
                    hash: *b"\x23\x61\xdf\x10\x18\xe7\x45\x89\x67\xcc\x1e\x55\x40\x69\xbd\xfb\x1e\x8e\xca\xad\x33\xdb\x04\x62\x80\x61\x29\xf8\x1e\xbb\x6a\x8a",
                },
            );
            m.insert(
                TestFileContent::MultiLine,
                TestVector {
                    content: String::from("First line\nSecond line\nThird line\n"),
                    hash: *b"\x10\x44\x17\x34\x23\x3b\x9d\xd3\x0c\xab\xee\xd4\x51\x1c\x8e\x5f\x56\xe6\x7c\xff\xc1\xd3\x7a\x2f\xcb\xef\xca\x85\x32\xcd\x34\xf2",
                },
            );
            m
        })
    }

    fn get_testfile(content: TestFileContent) -> TestFile {
        let mut file = NamedTempFile::new().expect("Can't create tempfile");

        let test_vector = get_testvector_hashmap().get(&content).unwrap().to_owned();
        write!(&mut file, "{}", &test_vector.content).expect("Can't write to tempfile");
        file.rewind().expect("Can't rewind file");

        TestFile { file, test_vector }
    }

    fn check_testfile(content: TestFileContent) {
        let mut testfile = get_testfile(content);
        let test_vector = get_testvector_hashmap().get(&content).unwrap().to_owned();
        let mut file_content = String::new();
        testfile
            .file
            .read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, test_vector.content);
        assert_eq!(testfile.test_vector.hash, test_vector.hash);
    }

    fn check_compute_hash(content: TestFileContent) {
        let testfile = get_testfile(content);
        let mut pathhash = PathHash::new(testfile.file.path());
        assert!(pathhash.hash().is_none());
        assert!(pathhash.compute_hash().is_ok());
        assert_eq!(*pathhash.hash().unwrap(), testfile.test_vector.hash);
    }

    #[test]
    fn get_testfile_empty() {
        check_testfile(TestFileContent::Empty);
    }

    #[test]
    fn get_testfile_singleline() {
        check_testfile(TestFileContent::SingleLine);
    }

    #[test]
    fn get_testfile_multiline() {
        check_testfile(TestFileContent::MultiLine);
    }

    #[test]
    fn compute_hash_from_non_existent() {
        let mut pathhash = PathHash::new(Path::new("/oiweisliejfliajseflij"));
        assert!(pathhash.hash().is_none());
        let err = pathhash.compute_hash().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn create_pathhash_from_existent() {
        let testfile = get_testfile(TestFileContent::SingleLine);
        let pathhash = PathHash::new(testfile.file.path());
        assert_eq!(pathhash.path(), testfile.file.path());
    }

    #[test]
    fn compute_hash_empty() {
        check_compute_hash(TestFileContent::Empty);
    }

    #[test]
    fn compute_hash_singleline() {
        check_compute_hash(TestFileContent::SingleLine);
    }

    #[test]
    fn compute_hash_multiline() {
        check_compute_hash(TestFileContent::MultiLine);
    }
}

#[cfg(any(test, feature = "test-mocks"))]
pub mod pathhashspy {
    use super::*;

    #[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
    pub struct PathHashSpy {
        path: PathBuf,
        hash: Option<[u8; 32]>,
        next_hash: Option<[u8; 32]>,
        call_count_compute_hash: u32,
    }

    impl PathHashSpy {
        pub fn new(path: PathBuf, hash: Option<[u8; 32]>, next_hash: Option<[u8; 32]>) -> Self {
            Self {
                path,
                hash,
                next_hash,
                call_count_compute_hash: 0,
            }
        }

        pub fn call_count_compute_hash(&self) -> u32 {
            self.call_count_compute_hash
        }
    }

    impl PathHashProvider for PathHashSpy {
        fn compute_hash(&mut self) -> Result<(), std::io::Error> {
            self.call_count_compute_hash += 1;

            match self.next_hash {
                Some(hash) => {
                    self.hash = Some(hash);
                    Ok(())
                }
                None => panic!("Can't compute next hash (next_hash is None)."),
            }
        }

        fn hash(&self) -> Option<&[u8; 32]> {
            self.hash.as_ref()
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    #[test]
    fn create_pathhashprovider_spies() {
        let spies = vec![
            PathHashSpy {
                path: Path::new("/some/path").to_owned(),
                hash: None,
                next_hash: None,
                call_count_compute_hash: 0,
            },
            PathHashSpy {
                path: Path::new("/other/path").to_owned(),
                hash: Some(*b"01234567890123456789012345678901"),
                next_hash: None,
                call_count_compute_hash: 0,
            },
        ];

        assert_eq!(spies[0].path().to_str().unwrap(), "/some/path");
        assert!(spies[0].hash().is_none());
        assert_eq!(spies[0].call_count_compute_hash(), 0);
        assert_eq!(spies[1].path().to_str().unwrap(), "/other/path");
        assert_eq!(spies[1].hash().unwrap()[4], 0x34);
        assert_eq!(spies[1].call_count_compute_hash(), 0);
    }

    #[test]
    fn compute_hash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: Some(*b"01234567890123456789012345678901"),
            call_count_compute_hash: 0,
        };

        assert!(spy.compute_hash().is_ok());

        assert_eq!(spy.call_count_compute_hash(), 1);
        assert_eq!(spy.hash().unwrap(), b"01234567890123456789012345678901");
        assert_eq!(&spy.next_hash.unwrap(), b"01234567890123456789012345678901");
    }

    // TODO:
    // - Use something different than std::io::Error to accomodate the error thrown here? But,
    // changing the error type just for the test is probably bad. Check if the type also profits
    // from changing the error type.
    #[test]
    #[should_panic]
    fn compute_hash_no_nexthash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: None,
            call_count_compute_hash: 0,
        };

        let _ = spy.compute_hash();
    }

    #[test]
    #[should_panic]
    fn compute_hash_later_no_nexthash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: Some(*b"01234567890123456789012345678901"),
            call_count_compute_hash: 0,
        };

        // Can't use asserts to check correct functionality as this would count as a panic,
        // fulfilling the `#[should_panic]` expectation of the test. Returning from the test
        // early will result in no panic which fails the test.
        if !spy.compute_hash().is_ok() {
            return;
        }

        if spy.hash().unwrap() != b"01234567890123456789012345678901" {
            return;
        }
        if &spy.next_hash.unwrap() != b"01234567890123456789012345678901" {
            return;
        }
        if spy.call_count_compute_hash() != 1 {
            return;
        }

        spy.next_hash = None;
        let _ = spy.compute_hash();
    }
}
