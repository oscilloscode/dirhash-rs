#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Seek, Write},
        sync::OnceLock,
    };

    use super::*;
    use tempfile::tempfile;

    static INIT: OnceLock<File> = OnceLock::new();

    fn init_file() -> &'static File {
        let mut tempfile = INIT.get_or_init(|| {
            let mut file = tempfile().expect("Can't create tempfile");
            write!(&mut file, "tempfile data").expect("Can't write to tempfile");
            file.rewind().expect("Can't rewind file");
            file
        });
        tempfile.rewind().expect("Can't rewind file");
        tempfile
    }

    #[test]
    fn init_creates_tempfile() {
        let mut file = init_file();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
    }

    #[test]
    fn multiple_inits_create_tempfile_once() {
        let file_a = init_file();
        let file_b = init_file();

        assert_eq!(file_a as *const File, file_b as *const File);
    }

    #[test]
    fn multiple_inits_rewind_tempfile() {
        let mut file = init_file();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
        file_content.clear();
        assert_eq!(file_content, "");

        let mut file = init_file();
        file.read_to_string(&mut file_content)
            .expect("Couldn't read file.");
        assert_eq!(file_content, "tempfile data");
    }
}
