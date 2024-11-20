use std::path::{Path, PathBuf};

pub(crate) struct FileHash {
    path: PathBuf,
}

impl FileHash {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Ok(FileHash {
            path: path.as_ref().to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Seek, Write},
        sync::{Mutex, MutexGuard, OnceLock},
    };

    use super::*;
    use tempfile::NamedTempFile;

    static INIT: OnceLock<Mutex<TestFile>> = OnceLock::new();

    struct TestFile {
        file: NamedTempFile,
        path: PathBuf,
        hash: String,
    }

    fn init_file() -> MutexGuard<'static, TestFile> {
        let tempfile = INIT.get_or_init(|| {
            let mut file = NamedTempFile::new().expect("Can't create tempfile");
            write!(&mut file, "tempfile data").expect("Can't write to tempfile");
            file.rewind().expect("Can't rewind file");
            let testfile = TestFile {
                path: file.path().to_owned(),
                file,
                hash: String::from("lkjsdf"),
            };
            testfile.into()
        });
        let mut mg = tempfile.lock().unwrap();
        mg.file.rewind().expect("Can't rewind file");
        mg
    }

    #[test]
    fn init_creates_tempfile() {
        let mut mg = init_file();
        let mut file_content = String::new();
        mg.file
            .read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
        assert_eq!(mg.hash, "lkjsdf");
    }

    #[test]
    fn multiple_inits_rewind_tempfile() {
        let mut testfile = init_file();
        let mut file_content = String::new();
        testfile
            .file
            .read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
        file_content.clear();
        assert_eq!(file_content, "");

        let mut testfile = init_file();
        testfile
            .file
            .read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
    }

    // #[test]
    // fn create_FileHash_from_non_existent() {
    //     let fh = FileHash::new(Path::new("/oiweisliejfliajseflij"));
    //     assert!(fh.is_err());
    // }

    // #[test]
    // fn create_FileHash_from_existent() {
    //     let testfile = init_file();
    //     let fh = FileHash::new(&testfile.path);
    //     assert!(fh.is_ok());
    // }
}
