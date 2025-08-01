/// DirHash
///
/// Test list:
/// - Create instance with path which gets canonicalized
/// - Create instance returns error is path doesn't exist
/// - Get files im empty dir returns nothing
/// - Get files in flat dir (no folders)
/// - Get files in deep dir (multiple levels)
/// - Get all file types (regular, link, char device, block device, socket, pipe)
/// - Getting files doesn't follow symlinks
/// - Compute hash of file (file, link, char device, block device, socket, pipe)
/// - Compute hash of multiple files
/// - Compute hash of list of hashes
///
// use std::path::{Path, PathBuf};
pub mod dirhash;
pub mod error;
pub mod hashtable;
pub mod pathhash;

// pub struct DirHash {
//     path: PathBuf,
// }

// impl DirHash {
//     pub fn new(path: &Path) -> Result<Self, std::io::Error> {
//         Ok(Self {
//             path: path.canonicalize()?,
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn create_inst_exists() {
//         let dh = DirHash::new(Path::new("/tmp")).expect("Existing path resulted in error.");
//         assert_eq!(dh.path.to_str().unwrap(), "/tmp");
//     }

//     #[test]
//     fn create_inst_doesnt_exist() {
//         let dh = DirHash::new(Path::new("/xyz"));
//         assert!(dh.is_err());
//     }

//     #[test]
//     fn create_inst_canonicalize() {
//         let dh = DirHash::new(Path::new("/tmp/..")).expect("Error while canonicalizing path.");
//         assert_eq!(dh.path.to_str().unwrap(), "/");
//     }
// }
