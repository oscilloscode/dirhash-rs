/// Test list:
/// - TestFile creation
/// - FileHash creation
/// - Hashing files
/// - Other filetypes (links, char device, block dev, socket, pipe)
///
use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

pub(crate) struct FileHash {
    path: PathBuf,
    hash: Option<String>,
}

impl FileHash {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Ok(FileHash {
            path: path.as_ref().canonicalize()?,
            hash: Default::default(),
        })
    }
    pub(crate) fn compute_hash(&mut self) -> Result<(), std::io::Error> {
        let data = fs::read(&self.path)?;
        let hash = Sha256::digest(data);
        self.hash = Some(hex::encode(hash));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, Seek, Write};
    use std::sync::OnceLock;

    use super::*;
    use tempfile::NamedTempFile;

    #[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
    struct TestVector {
        content: String,
        hash: String,
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
                    hash: String::from(
                        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    ),
                },
            );
            m.insert(
                TestFileContent::SingleLine,
                TestVector {
                    content: String::from("First line"),
                    hash: String::from(
                        "2361df1018e7458967cc1e554069bdfb1e8ecaad33db0462806129f81ebb6a8a",
                    ),
                },
            );
            m.insert(
                TestFileContent::MultiLine,
                TestVector {
                    content: String::from("First line\nSecond line\nThird line\n"),
                    hash: String::from(
                        "10441734233b9dd30cabeed4511c8e5f56e67cffc1d37a2fcbefca8532cd34f2",
                    ),
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
        assert_eq!(test_vector.content, file_content);
        assert_eq!(test_vector.hash, testfile.test_vector.hash);
    }

    fn check_compute_hash(content: TestFileContent) {
        let testfile = get_testfile(content);
        let mut filehash =
            FileHash::new(&testfile.file.path()).expect("Can't create FileHash from existing file");
        assert!(filehash.hash.is_none());
        filehash.compute_hash();
        assert_eq!(testfile.test_vector.hash, filehash.hash.unwrap());
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
    fn create_filehash_from_non_existent() {
        let filehash = FileHash::new(Path::new("/oiweisliejfliajseflij"));
        assert!(filehash.is_err());
    }

    #[test]
    fn create_filehash_from_existent() {
        let testfile = get_testfile(TestFileContent::SingleLine);
        let filehash =
            FileHash::new(&testfile.file.path()).expect("Can't create FileHash from existing file");
        assert_eq!(testfile.file.path(), filehash.path);
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
