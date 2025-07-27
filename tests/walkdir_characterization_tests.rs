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

use std::{fs::File, os::unix, path::Path};
use walkdir::WalkDir;

mod common;

fn get_filecount(path: impl AsRef<Path>, count_symlinks: bool) -> usize {
    WalkDir::new(path)
        .follow_links(count_symlinks)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .count()
}

fn get_filecount_at_depth(path: impl AsRef<Path>, count_symlinks: bool, depth: usize) -> usize {
    WalkDir::new(path)
        .follow_links(count_symlinks)
        .min_depth(depth)
        .max_depth(depth)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .count()
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir() {
    let dir = common::creating_tempdir(
        None,
        4,
        &["a", "b", "c"][..],
        6,
        &["x", "y", "z"][..],
        3,
        false,
    );

    assert_eq!(get_filecount(&dir, false), 49);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 27);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_hidden() {
    let dir = common::creating_tempdir(
        None,
        2,
        &["a", "b"][..],
        4,
        &["w", "x", "y", "z"][..],
        1,
        false,
    );

    File::create(dir.path().join(".gitignore")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/.hidden2")).expect("Error while creating hidden file");
    File::create(dir.path().join("a/x/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/y/.hidden")).expect("Error while creating hidden file");
    File::create(dir.path().join("b/z/.hidden")).expect("Error while creating hidden file");

    assert_eq!(get_filecount(&dir, false), 24);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 3);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 10);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 11);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_special() {
    let dir = common::creating_tempdir(
        None,
        5,
        &["a", "b", "c", "d"][..],
        2,
        &["y", "z"][..],
        4,
        false,
    );

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

    assert_eq!(get_filecount(&dir, false), 51);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 6);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 10);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 35);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_many_files() {
    let dir = common::creating_tempdir(
        None,
        504,
        &["a", "b", "c", "d", "e", "f"][..],
        885,
        &["t", "u", "v", "w", "x", "y", "z"][..],
        1034,
        false,
    );

    assert_eq!(get_filecount(&dir, false), 49242);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 504);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 5310);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 43428);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_file_symlinks() {
    let dir = common::creating_tempdir(
        None,
        4,
        &["a", "b", "c"][..],
        6,
        &["x", "y", "z"][..],
        3,
        false,
    );

    // file downwards
    unix::fs::symlink(dir.path().join("a/x/0"), dir.path().join("downwards_link"))
        .expect("Error while creating symlink");

    // file upwards
    unix::fs::symlink(dir.path().join("1"), dir.path().join("b/y/upwards_link"))
        .expect("Error while creating symlink");

    // not following links
    assert_eq!(get_filecount(&dir, false), 49);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 27);

    // following links (filetype of symlink is the same as target)
    assert_eq!(get_filecount(&dir, true), 51); // +2 file symlinks
    assert_eq!(get_filecount_at_depth(&dir, true, 1), 5); // downwards_link
    assert_eq!(get_filecount_at_depth(&dir, true, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, true, 3), 28); // upwards_link

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_file_symlink_loop() {
    let dir = common::creating_tempdir(
        None,
        4,
        &["a", "b", "c"][..],
        6,
        &["x", "y", "z"][..],
        3,
        false,
    );

    // file downwards
    unix::fs::symlink(
        dir.path().join("a/upwards_link"),
        dir.path().join("downwards_link"),
    )
    .expect("Error while creating symlink");

    // file upwards
    unix::fs::symlink(
        dir.path().join("downwards_link"),
        dir.path().join("a/upwards_link"),
    )
    .expect("Error while creating symlink");

    // not following links
    assert_eq!(get_filecount(&dir, false), 49);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 27);

    // following links, but links that result in loops result in Err() and are discarded
    assert_eq!(get_filecount(&dir, true), 49);
    assert_eq!(get_filecount_at_depth(&dir, true, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, true, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, true, 3), 27);

    dir.close().expect("Can't close tempdir");
}

// #[ignore = "will ichs chan"]
#[test]
fn three_level_dir_with_dir_symlinks() {
    let dir = common::creating_tempdir(
        None,
        4,
        &["a", "b", "c"][..],
        6,
        &["x", "y", "z"][..],
        3,
        false,
    );

    // dir downwards
    unix::fs::symlink(dir.path().join("a/x"), dir.path().join("downwards_link"))
        .expect("Error while creating symlink");

    // dir upwards
    unix::fs::symlink(dir.path(), dir.path().join("b/y/upwards_link"))
        .expect("Error while creating symlink");

    // not following links
    assert_eq!(get_filecount(&dir, false), 49);
    assert_eq!(get_filecount_at_depth(&dir, false, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, false, 2), 18);
    assert_eq!(get_filecount_at_depth(&dir, false, 3), 27);

    // following links (filetype of symlink is the same as target, i.e., directory. thus, symlink
    // doesn't get counted but all the files that are located in that target folder)
    // upwards_link results in error because loop is detected
    assert_eq!(get_filecount(&dir, true), 52); // 3 more files from following downward_link, e.g. ./downwards_link/0
    assert_eq!(get_filecount_at_depth(&dir, true, 1), 4);
    assert_eq!(get_filecount_at_depth(&dir, true, 2), 21); // ./downwards_link/0, ./downwards_link/1, ./downwards_link/2
    assert_eq!(get_filecount_at_depth(&dir, true, 3), 27);

    dir.close().expect("Can't close tempdir");
}
