//! Test list:
//!

use std::{
    fs, io,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::error::{DirHashError, InvalidFileTypeKind, Result};

// TODO: Rename this!!
pub trait PathHashProvider {
    fn path(&self) -> &Path;
    fn hash(&self) -> Option<&[u8; 32]>;
    fn compute_hash(&mut self) -> Result<()>;
}

/// Struct containing a path and hash from a file on the filesystem.
#[derive(Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PathHash {
    path: PathBuf,
    hash: Option<[u8; 32]>,
}

impl PathHash {
    /// Creates a [`PathHash`] from a path to a file on the system.
    ///
    /// Returns an [`DirHashError::Io`] if the file doesn't exist or if it isn't absolute. Symlinks
    /// are not resolved ant thus the `canonicalize` method of [`std::path::Path`] can't be used.
    ///
    /// Currently, `..` and `.` are not resolved.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        // Put this first, as this is a simple lexical check without accessing the filesystem.
        if !path.as_ref().is_absolute() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "path not absolute").into());
        }

        // We need the metadata to throw errors on invalid file types. Luckily, this will also
        // return an io::Error (NotFound).
        let filetype = fs::metadata(&path)?.file_type();

        if filetype.is_dir() {
            return Err(DirHashError::InvalidFileType(
                InvalidFileTypeKind::Dir,
                path.as_ref().to_owned(),
            ));
        }

        if filetype.is_block_device() {
            return Err(DirHashError::InvalidFileType(
                InvalidFileTypeKind::BlockDevice,
                path.as_ref().to_owned(),
            ));
        }

        if filetype.is_char_device() {
            return Err(DirHashError::InvalidFileType(
                InvalidFileTypeKind::CharDevice,
                path.as_ref().to_owned(),
            ));
        }

        if filetype.is_fifo() {
            return Err(DirHashError::InvalidFileType(
                InvalidFileTypeKind::FIFO,
                path.as_ref().to_owned(),
            ));
        }

        if filetype.is_socket() {
            return Err(DirHashError::InvalidFileType(
                InvalidFileTypeKind::Socket,
                path.as_ref().to_owned(),
            ));
        }

        Ok(PathHash {
            path: path.as_ref().to_owned(),
            hash: Default::default(),
        })
    }
}

impl PathHashProvider for PathHash {
    /// Computes the SHA256 hash of the contents of the corresponding file and stores it. Calling
    /// this method again will reread the file and recompute the hash value.
    fn compute_hash(&mut self) -> Result<()> {
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
    use std::os::unix;
    use std::os::unix::fs::FileTypeExt;
    use std::sync::OnceLock;

    use crate::error::{DirHashError, InvalidFileTypeKind};

    use super::*;
    use fs::File;
    use tempfile::{tempdir, NamedTempFile};

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
        let mut pathhash =
            PathHash::new(testfile.file.path()).expect("Can't create PathHash from existing file");
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
    fn create_pathhash_not_found() {
        let err = PathHash::new(Path::new("/oiweisliejfliajseflij")).unwrap_err();

        match err {
            DirHashError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Wrong enum variant"),
        }
    }

    #[test]
    fn create_pathhash_from_different_path_types() {
        let err = PathHash::new(Path::new("/oiweisliejfliajseflij")).unwrap_err();
        match err {
            DirHashError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Wrong enum variant"),
        }

        let err = PathHash::new("/oiweisliejfliajseflij").unwrap_err();
        match err {
            DirHashError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Wrong enum variant"),
        }

        let err = PathHash::new(String::from("/oiweisliejfliajseflij")).unwrap_err();
        match err {
            DirHashError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Wrong enum variant"),
        }
    }

    #[test]
    fn create_pathhash_not_absolute() {
        let err = PathHash::new("./0").unwrap_err();
        match err {
            DirHashError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
            }
            _ => panic!("Wrong enum variant"),
        }
    }

    #[test]
    fn create_pathhash_from_existent() {
        let testfile = get_testfile(TestFileContent::SingleLine);
        let pathhash =
            PathHash::new(testfile.file.path()).expect("Can't create PathHash from existing file");
        assert_eq!(pathhash.path(), testfile.file.path());
    }

    #[test]
    fn create_and_hash_symlink() {
        let dir = tempdir().expect("Can't create tempdir");
        // let dir = tempfile::Builder::new()
        //     .keep(true)
        //     .tempdir()
        //     .expect("Can't create tempdir");

        let datafile_path = dir.as_ref().join("datafile");
        let mut file = File::create(&datafile_path).expect("Error while creating file");

        write!(&mut file, "{}", "test data").expect("Can't write to tempfile");

        let symlink_path = dir.path().join("symlink");
        unix::fs::symlink(datafile_path, &symlink_path).expect("Error while creating symlink");

        let symlink_data =
            fs::read_to_string(&symlink_path).expect("Error while reading data from symlink");
        assert_eq!(symlink_data, "test data");

        let mut pathhash = PathHash::new(&symlink_path).unwrap();
        assert_eq!(symlink_path, pathhash.path());

        assert!(pathhash.hash().is_none());
        assert!(pathhash.compute_hash().is_ok());
        assert_eq!(pathhash.hash().unwrap(), b"\x91\x6f\x00\x27\xa5\x75\x07\x4c\xe7\x2a\x33\x17\x77\xc3\x47\x8d\x65\x13\xf7\x86\xa5\x91\xbd\x89\x2d\xa1\xa5\x77\xbf\x23\x35\xf9");

        dir.close().expect("Can't close tempdir");
    }

    #[test]
    fn create_and_hash_hardlink() {
        let dir = tempdir().expect("Can't create tempdir");
        // let dir = tempfile::Builder::new()
        //     .keep(true)
        //     .tempdir()
        //     .expect("Can't create tempdir");

        let datafile_path = dir.as_ref().join("datafile");
        let mut file = File::create(&datafile_path).expect("Error while creating file");

        write!(&mut file, "{}", "Here is some data").expect("Can't write to tempfile");

        let hardlink_path = dir.path().join("hardlink");
        fs::hard_link(datafile_path, &hardlink_path).expect("Error while creating hardlink");

        let hardlink_data =
            fs::read_to_string(&hardlink_path).expect("Error while reading data from hardlink");
        assert_eq!(hardlink_data, "Here is some data");

        let mut pathhash = PathHash::new(&hardlink_path).unwrap();
        assert_eq!(hardlink_path, pathhash.path());

        assert!(pathhash.hash().is_none());
        assert!(pathhash.compute_hash().is_ok());
        assert_eq!(pathhash.hash().unwrap(), b"\x15\xf2\x36\xd5\xf1\x4e\xc9\xbd\x26\x47\xcb\x5d\xd5\x09\xbf\x53\x3c\x31\x4a\xa3\xc7\x11\x9d\x2d\x7b\x70\x46\x6a\xa5\x00\x58\x95");

        dir.close().expect("Can't close tempdir");
    }

    #[test]
    fn dir_returns_error() {
        let dev_path = Path::new("/dev");
        let dev_metadata = fs::metadata(dev_path).expect("Can't get metadata of /dev");
        assert!(dev_metadata.file_type().is_dir());

        let err = PathHash::new(dev_path).expect_err("Directory didn't return an error");

        match err {
            DirHashError::InvalidFileType(filetype, path) => match filetype {
                InvalidFileTypeKind::Dir => {
                    assert_eq!(path, dev_path)
                }
                _ => panic!("Wrong InvalidFileType enum variant"),
            },
            _ => panic!("Wrong DirHashError enum variant"),
        }
    }

    #[test]
    fn block_device_returns_error() {
        let sda_path = Path::new("/dev/sda");
        let sda_metadata = fs::metadata(sda_path).expect("Can't get metadata of /dev/sda");
        assert!(sda_metadata.file_type().is_block_device());

        let err = PathHash::new(sda_path).expect_err("Block device didn't return an error");

        match err {
            DirHashError::InvalidFileType(filetype, path) => match filetype {
                InvalidFileTypeKind::BlockDevice => {
                    assert_eq!(path, sda_path)
                }
                _ => panic!("Wrong InvalidFileType enum variant"),
            },
            _ => panic!("Wrong DirHashError enum variant"),
        }
    }

    #[test]
    fn char_device_returns_error() {
        let dev_null_path = Path::new("/dev/null");
        let dev_null_metadata =
            fs::metadata(dev_null_path).expect("Can't get metadata of /dev/null");
        assert!(dev_null_metadata.file_type().is_char_device());

        let err = PathHash::new(dev_null_path).expect_err("Char device didn't return an error");

        match err {
            DirHashError::InvalidFileType(filetype, path) => match filetype {
                InvalidFileTypeKind::CharDevice => {
                    assert_eq!(path, dev_null_path)
                }
                _ => panic!("Wrong InvalidFileType enum variant"),
            },
            _ => panic!("Wrong DirHashError enum variant"),
        }
    }

    #[test]
    fn fifo_returns_error() {
        // Is this a good file? Do all Linux distros have this?
        let initctl_path = Path::new("/run/initctl");
        let initctl_metadata =
            fs::metadata(initctl_path).expect("Can't get metadata of /run/initctl");
        assert!(initctl_metadata.file_type().is_fifo());

        let err = PathHash::new(initctl_path).expect_err("FIFO file didn't return an error");

        match err {
            DirHashError::InvalidFileType(filetype, path) => match filetype {
                InvalidFileTypeKind::FIFO => {
                    assert_eq!(path, initctl_path)
                }
                _ => panic!("Wrong InvalidFileType enum variant"),
            },
            _ => panic!("Wrong DirHashError enum variant"),
        }
    }

    #[test]
    fn socket_returns_error() {
        // Is this a good file? Do all Linux distros have this?
        let systemd_private_path = Path::new("/run/systemd/private");
        let systemd_private_metadata =
            fs::metadata(systemd_private_path).expect("Can't get metadata of /run/systemd/private");
        assert!(systemd_private_metadata.file_type().is_socket());

        let err =
            PathHash::new(systemd_private_path).expect_err("Socket file didn't return an error");

        match err {
            DirHashError::InvalidFileType(filetype, path) => match filetype {
                InvalidFileTypeKind::Socket => {
                    assert_eq!(path, systemd_private_path)
                }
                _ => panic!("Wrong InvalidFileType enum variant"),
            },
            _ => panic!("Wrong DirHashError enum variant"),
        }
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
        pub fn new(
            path: impl AsRef<Path>,
            hash: Option<[u8; 32]>,
            next_hash: Option<[u8; 32]>,
        ) -> Self {
            Self {
                path: path.as_ref().to_owned(),
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
        fn compute_hash(&mut self) -> Result<()> {
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
            PathHashSpy::new("/some/path", None, None),
            PathHashSpy::new(
                "/other/path",
                Some(*b"01234567890123456789012345678901"),
                None,
            ),
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
        let mut spy = PathHashSpy::new(
            "/some/path",
            None,
            Some(*b"01234567890123456789012345678901"),
        );

        assert!(spy.compute_hash().is_ok());

        assert_eq!(spy.call_count_compute_hash(), 1);
        assert_eq!(spy.hash().unwrap(), b"01234567890123456789012345678901");
        assert_eq!(&spy.next_hash.unwrap(), b"01234567890123456789012345678901");
    }

    #[test]
    #[should_panic]
    fn compute_hash_no_nexthash() {
        let mut spy = PathHashSpy::new("/some/path", None, None);

        let _ = spy.compute_hash();
    }

    #[test]
    #[should_panic]
    fn compute_hash_later_no_nexthash() {
        let mut spy = PathHashSpy::new(
            "/some/path",
            None,
            Some(*b"01234567890123456789012345678901"),
        );

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
