//! * Rationale
//!
//! These characterization tests intend to capture my assumptions about the walkdir crate and they
//! function as a visual reference. Additionally, this prevents me from overlooking a breaking
//! change by resulting in test failures.
//!
//! Things to check:
//! - Listing all files on multiple levels
//! - List a lot of files
//! - Listing hidden files
//! - Listing files with special characters in filename
//! - How to handle symlinks?

use std::{fs::File, path::Path};
use tempfile::{tempdir, TempDir};
use walkdir::WalkDir;

fn create_numbered_files(dir: impl AsRef<Path>, n: usize) {
    for i in 0..n {
        let _ =
            File::create(dir.as_ref().join(format!("{}", i))).expect("Error while creating file");
    }
}

/// Creates the following directory structure for creating_tempdir(4, &["a", "b", "c"][..], 6,
/// &["x", "y", "z"][..], 3):
/// ```
/// tmpbSlLgw/
/// ├── 0
/// ├── 1
/// ├── 2
/// ├── 3
/// ├── a
/// │   ├── 0
/// │   ├── 1
/// │   ├── 2
/// │   ├── 3
/// │   ├── 4
/// │   ├── 5
/// │   ├── x
/// │   │   ├── 0
/// │   │   ├── 1
/// │   │   └── 2
/// │   ├── y
/// │   │   ├── 0
/// │   │   ├── 1
/// │   │   └── 2
/// │   └── z
/// │       ├── 0
/// │       ├── 1
/// │       └── 2
/// ├── b
/// │   ├── 0
/// │   ├── 1
/// │   ├── 2
/// │   ├── 3
/// │   ├── 4
/// │   ├── 5
/// │   ├── x
/// │   │   ├── 0
/// │   │   ├── 1
/// │   │   └── 2
/// │   ├── y
/// │   │   ├── 0
/// │   │   ├── 1
/// │   │   └── 2
/// │   └── z
/// │       ├── 0
/// │       ├── 1
/// │       └── 2
/// └── c
///     ├── 0
///     ├── 1
///     ├── 2
///     ├── 3
///     ├── 4
///     ├── 5
///     ├── x
///     │   ├── 0
///     │   ├── 1
///     │   └── 2
///     ├── y
///     │   ├── 0
///     │   ├── 1
///     │   └── 2
///     └── z
///         ├── 0
///         ├── 1
///         └── 2
/// ```
fn creating_tempdir(
    l1_files: usize,
    l1_dirs: &[&str],
    l2_files: usize,
    l2_dirs: &[&str],
    l3_files: usize,
) -> TempDir {
    // let dir = tempdir().expect("Can't create tempdir");
    let dir = tempfile::Builder::new()
        .keep(true)
        .tempdir()
        .expect("Can't create tempdir");

    create_numbered_files(&dir, l1_files);

    for d in l1_dirs.iter() {
        let dir_level_1 = dir.path().join(d.to_string());
        std::fs::create_dir(&dir_level_1)
            .expect(&format!("Error while creating directory {:?}", dir_level_1));

        create_numbered_files(&dir_level_1, l2_files);

        for d in l2_dirs.iter() {
            let dir_level_2 = dir_level_1.join(d.to_string());
            std::fs::create_dir(&dir_level_2)
                .expect(&format!("Error while creating directory {:?}", dir_level_2));

            create_numbered_files(&dir_level_2, l3_files);
        }
    }

    dir
}

#[test]
fn this_works() {
    let dir = creating_tempdir(4, &["a", "b", "c"][..], 6, &["x", "y", "z"][..], 3);

    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        println!("{:?}", entry.path().strip_prefix(&dir).unwrap());
    }

    // dir.close().expect("Can't close tempdir");
}
