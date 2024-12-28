//! Test list:
//! - PathHashList only calls `compute_hash` if no hash in PathHash
//! - Define and test what happens with sorting two files have the same hash (i.e., same content).
//!   Probably use the paths to define order.
//! - Add tests to check that sort() behaves as expected (both for the hash and the path)
//!

use std::fmt::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

// TODO: Rename this!!
pub trait PathHashProvider {
    fn path(&self) -> &Path;
    fn hash(&self) -> Option<&[u8; 32]>;
    fn compute_hash(&mut self) -> Result<(), std::io::Error>;
}

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

    pub fn compute_hash(&mut self) -> Result<(), std::io::Error> {
        let mut hashable_data_vec: Vec<([u8; 32], PathBuf)> =
            Vec::with_capacity(self.pathhashvec.len());

        dbg!(&hashable_data_vec);

        for pb in &mut self.pathhashvec {
            if pb.hash().is_none() {
                pb.compute_hash()?;
            }
            hashable_data_vec.push((*pb.hash().unwrap(), pb.path().to_owned()));
        }
        dbg!(&hashable_data_vec);

        hashable_data_vec.sort();

        dbg!(&hashable_data_vec);

        // let mut hasher = Sha256::new();
        let mut hashable_string = String::new();

        for (hash, path) in hashable_data_vec {
            let _ = writeln!(
                &mut hashable_string,
                "{}  {}",
                hex::encode(hash),
                path.to_string_lossy()
            );
            // let hashable_string = format!("{}  {}", hex::encode(hash), path.to_string_lossy());
            // print!("{}", hashable_string);
            dbg!(&hashable_string);
        }

        dbg!(&hashable_string);

        // hasher.update(hashable_string);

        // let hash = hasher.finalize();

        let hash = Sha256::digest(hashable_string);

        self.hash = Some(hash.into());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod slice_sort_behavior {
        #[test]
        fn u8_array_first_element_different() {
            let mut v = vec![[4, 2, 3], [1, 2, 3]];
            v.sort();
            assert_eq!([1, 2, 3], v[0]);
            assert_eq!([4, 2, 3], v[1]);
        }

        #[test]
        fn u8_array_middle_element_different() {
            let mut v = vec![[1, 4, 3], [1, 2, 3]];
            v.sort();
            assert_eq!([1, 2, 3], v[0]);
            assert_eq!([1, 4, 3], v[1]);
        }

        #[test]
        fn u8_array_last_element_different() {
            let mut v = vec![[1, 2, 4], [1, 2, 3]];
            v.sort();
            assert_eq!([1, 2, 3], v[0]);
            assert_eq!([1, 2, 4], v[1]);
        }

        #[test]
        fn u8_array_sorted() {
            let mut v = vec![[1, 2, 3], [4, 2, 3]];
            v.sort();
            assert_eq!([1, 2, 3], v[0]);
            assert_eq!([4, 2, 3], v[1]);
        }

        #[test]
        fn u8_vec_equal_but_different_lengths() {
            let mut v = vec![vec![1, 2, 3], vec![1, 2]];
            v.sort();
            assert_eq!(&[1, 2][..], v[0]);
            assert_eq!(&[1, 2, 3][..], v[1]);
        }

        #[test]
        fn u8_vec_one_empty() {
            let mut v = vec![vec![1, 2, 3], vec![]];
            v.sort();
            assert_eq!(0, v[0].len());
            assert_eq!(&[1, 2, 3][..], v[1]);
        }

        #[test]
        fn strings() {
            assert!(false);
        }

        #[test]
        fn tuples() {
            assert!(false);
        }
    }

    mod pathhashspy {
        use super::*;

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

            assert_eq!("/some/path", spies[0].path().to_str().unwrap());
            assert!(spies[0].hash().is_none());
            assert_eq!(0, spies[0].call_count_compute_hash());
            assert_eq!("/other/path", spies[1].path().to_str().unwrap());
            assert_eq!(0x34, spies[1].hash().unwrap()[4]);
            assert_eq!(0, spies[1].call_count_compute_hash());
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

            assert_eq!(1, spy.call_count_compute_hash());
            assert_eq!(b"01234567890123456789012345678901", spy.hash().unwrap());
            assert_eq!(b"01234567890123456789012345678901", &spy.next_hash.unwrap());
        }

        // TODO:
        // - Use something different than std::io::Error to accomodate the error thrown here? But,
        // changing the error type just for the test is probably bad. Check if the type also profits
        // from changing the error type.
        #[test]
        fn compute_hash_no_nexthash() {
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
        fn compute_hash_later_no_nexthash() {
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
    }

    use super::*;
    use pathhashspy::PathHashSpy;

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
        assert_eq!(0x37, pathhashlist.hash().unwrap()[7]);
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

        assert_eq!(0, pathhashlist.pathhashvec[0].call_count_compute_hash());
        assert_eq!(0, pathhashlist.pathhashvec[1].call_count_compute_hash());

        // Hash of (the newline after the second line is also part of the digest):
        // 59ead62a5f16e4ee2f7de89e52f978d6f15e97f387255dd77ed3c72f88882855  /other/path
        // d83ba80420ec99bcb143df16a00c39a56c140341e4446ae9b5e8b5a6d18116ed  /some/path
        //
        // -> 4dcf91beae7c9fcc68df4f57ab4344a744e7d0c326003a03e7996f87fe451390
        assert_eq!(b"\x4d\xcf\x91\xbe\xae\x7c\x9f\xcc\x68\xdf\x4f\x57\xab\x43\x44\xa7\x44\xe7\xd0\xc3\x26\x00\x3a\x03\xe7\x99\x6f\x87\xfe\x45\x13\x90", pathhashlist.hash().unwrap());
    }
}
