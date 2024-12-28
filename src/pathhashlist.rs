/// Test list:
///
use std::path::Path;

// TODO: Rename this!!
pub trait PathHashProvider {
    fn path(&self) -> &Path;
    fn hash(&self) -> Option<&[u8; 32]>;
    fn compute_hash(&mut self) -> Result<(), std::io::Error>;
}

pub struct PathHashList<T> {
    files: Vec<T>,
    hash: Option<[u8; 32]>,
}

impl<T> PathHashList<T>
where
    T: PathHashProvider,
{
    pub fn new(files: Vec<T>) -> Result<Self, std::io::Error> {
        // Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        Ok(PathHashList { files, hash: None })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // Todo:
    // - Create a new function and force calling it by wrapping spy into "instrumentation" module?
    struct PathHashSpy {
        path: PathBuf,
        hash: Option<[u8; 32]>,
        next_hash: Option<[u8; 32]>,
        call_count_compute_hash: u32,
    }

    impl PathHashSpy {
        fn call_count_compute_hash(&self) -> u32 {
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
                None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "oh no!")),
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
        let mut spies = vec![
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

        assert_eq!("/some/path", spies[0].path().to_str().unwrap());
        assert!(spies[0].hash().is_none());
        assert_eq!(0, spies[0].call_count_compute_hash());
        assert_eq!("/other/path", spies[1].path().to_str().unwrap());
        assert_eq!(0x34, spies[1].hash().unwrap()[4]);
        assert_eq!(0, spies[1].call_count_compute_hash());
    }

    #[test]
    fn spy_compute_hash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: Some(*b"01234567890123456789012345678901"),
            call_count_compute_hash: 0,
        };

        assert!(spy.compute_hash().is_ok());

        assert_eq!(1, spy.call_count_compute_hash());
        assert_eq!(b"01234567890123456789012345678901", spy.hash().unwrap());
        assert_eq!(b"01234567890123456789012345678901", &spy.next_hash.unwrap());
    }

    // TODO:
    // - Use something different than std::io::Error to accomodate the error thrown here? But,
    // changing the error type just for the test is probably bad. Check if the type also profits
    // from changing the error type.
    #[test]
    fn spy_compute_hash_no_nexthash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: None,
            call_count_compute_hash: 0,
        };

        let e = spy.compute_hash();
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        assert_eq!(1, spy.call_count_compute_hash());
    }

    #[test]
    fn spy_compute_hash_later_no_nexthash() {
        let mut spy = PathHashSpy {
            path: Path::new("/some/path").to_owned(),
            hash: None,
            next_hash: Some(*b"01234567890123456789012345678901"),
            call_count_compute_hash: 0,
        };

        assert!(spy.compute_hash().is_ok());

        assert_eq!(b"01234567890123456789012345678901", spy.hash().unwrap());
        assert_eq!(b"01234567890123456789012345678901", &spy.next_hash.unwrap());
        assert_eq!(1, spy.call_count_compute_hash());

        spy.next_hash = None;
        let e = spy.compute_hash();
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        assert_eq!(2, spy.call_count_compute_hash());
    }

    #[test]
    fn pathhashlist_hash_is_none_after_init() {
        let spies: Vec<PathHashSpy> = vec![];
        let pathhashlist = PathHashList::new(spies).expect("Can't create PathHashList");
        assert!(pathhashlist.hash.is_none());
    }
}
