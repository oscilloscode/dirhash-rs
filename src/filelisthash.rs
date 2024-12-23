/// Test list:
///
/// Todo:
/// - Rename everything to PathHash<File|Stub>
///
use std::path::Path;

// TODO: Rename this!!
pub trait PathHashProvider {
    fn path(&self) -> &Path;
    fn hash(&self) -> Option<&[u8; 32]>;
    fn compute_hash(&mut self) -> Result<(), std::io::Error>;
}

pub struct FileListHash<T> {
    files: Vec<T>,
    hash: Option<[u8; 32]>,
}

impl<T> FileListHash<T>
where
    T: PathHashProvider,
{
    pub fn new(files: Vec<T>) -> Result<Self, std::io::Error> {
        // Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        Ok(FileListHash { files, hash: None })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    struct PathHashStub {
        path: PathBuf,
        hash: Option<[u8; 32]>,
    }

    impl PathHashProvider for PathHashStub {
        fn compute_hash(&mut self) -> Result<(), std::io::Error> {
            Ok(())
            // Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        }

        fn hash(&self) -> Option<&[u8; 32]> {
            self.hash.as_ref()
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    #[test]
    fn create_stubs_for_filelisthash() {
        let stubs = vec![
            PathHashStub {
                path: Path::new("/some/path").to_owned(),
                hash: None,
            },
            PathHashStub {
                path: Path::new("/other/path").to_owned(),
                hash: Some(*b"01234567890123456789012345678901"),
            },
        ];
        let mut filelisthash = FileListHash::new(stubs).expect("Can't create FileListHash");

        assert!(filelisthash.files[0].compute_hash().is_ok());
        assert_eq!("/some/path", filelisthash.files[0].path().to_str().unwrap());
        assert!(filelisthash.files[0].hash().is_none());
        assert_eq!(
            "/other/path",
            filelisthash.files[1].path().to_str().unwrap()
        );
        assert_eq!(0x34, filelisthash.files[1].hash().unwrap()[4]);
    }

    #[test]
    fn filelisthash_hash_is_none_after_init() {
        let stubs: Vec<PathHashStub> = vec![];
        let filelisthash = FileListHash::new(stubs).expect("Can't create FileListHash");
        assert!(filelisthash.hash.is_none());
    }
}
