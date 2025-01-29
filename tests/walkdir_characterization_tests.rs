//! * Rationale
//!
//! These characterization tests intend to capture my assumptions about the walkdir crate and they
//! function as a visual reference. Additionally, this prevents me from overlooking a breaking
//! change by resulting in test failures.
//!
//! Things to check:
//! - Listing all files on multiple levels
//! - Listing hidden files
//! - Listing files with special characters in filename
//! - List a lot of files
//! - Listing very deep directories
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
    let dir = tempdir().expect("Can't create tempdir");
    // let dir = tempfile::Builder::new()
    //     .keep(true)
    //     .tempdir()
    //     .expect("Can't create tempdir");

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

fn get_filecount(path: impl AsRef<Path>) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .count()
}

fn get_filecount_at_depth(path: impl AsRef<Path>, depth: usize) -> usize {
    WalkDir::new(path)
        .min_depth(depth)
        .max_depth(depth)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .count()
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir() {
    let dir = creating_tempdir(4, &["a", "b", "c"][..], 6, &["x", "y", "z"][..], 3);

    assert_eq!(get_filecount(&dir), 49);
    assert_eq!(get_filecount_at_depth(&dir, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, 3), 27);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_hidden() {
    let dir = creating_tempdir(2, &["a", "b"][..], 4, &["w", "x", "y", "z"][..], 1);

    File::create(dir.path().join(".gitignore")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/.hidden2")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/x/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/y/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/z/.hidden")).expect("Error while creating hidden file");

    assert_eq!(get_filecount(&dir), 24);
    assert_eq!(get_filecount_at_depth(&dir, 1), 3);
    assert_eq!(get_filecount_at_depth(&dir, 2), 10);
    assert_eq!(get_filecount_at_depth(&dir, 3), 11);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_special() {
    let dir = creating_tempdir(5, &["a", "b", "c", "d"][..], 2, &["y", "z"][..], 4);

    // Printable characters, that are forbidden in Windows but allowed in Linux.
    File::create(dir.path().join("<angle brackets>")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/quote\"quote")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/back\\slash")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/y/pipe|pipe")).expect("Error while creating hidden file");
    File::create(dir.path().join("c/y/question?mark")).expect("Error while creating hidden file");
    File::create(dir.path().join("d/z/aste*risk")).expect("Error while creating hidden file");
    // Apparently, 0-31 (ASCII control characters) aren't allowed in Windows, but Linux only
    // disallows 0 (NULL byte). However, I don't feel like testing control characters. Adding them
    // to filenames is insane anyway...

    assert_eq!(get_filecount(&dir), 51);
    assert_eq!(get_filecount_at_depth(&dir, 1), 6);
    assert_eq!(get_filecount_at_depth(&dir, 2), 10);
    assert_eq!(get_filecount_at_depth(&dir, 3), 35);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_many_files() {
    let dir = creating_tempdir(
        504,
        &["a", "b", "c", "d", "e", "f"][..],
        885,
        &["t", "u", "v", "w", "x", "y", "z"][..],
        1034,
    );

    assert_eq!(get_filecount(&dir), 49242);
    assert_eq!(get_filecount_at_depth(&dir, 1), 504);
    assert_eq!(get_filecount_at_depth(&dir, 2), 5310);
    assert_eq!(get_filecount_at_depth(&dir, 3), 43428);

    dir.close().expect("Can't close tempdir");
}
