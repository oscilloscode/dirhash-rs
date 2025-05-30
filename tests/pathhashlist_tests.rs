//! Things to check:
//! - Creating PathHashList recursively from path, then compute hash
//! - Creating PathHashList from list of paths, then compute hash
//! - Add additional list of paths after creation , then compute hash
//! - Symlinks in specified files
//! - Return error when encountering illegal filetype (char dev, block dev, socket, pipe)

use dirhash_rs::pathhashlist::PathHashList;

mod common;

#[test]
fn create_from_path_recursively_no_root() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_create_from_path_recursively_no_root")),
        2,
        &["a", "b"][..],
        1,
        &["x", "y"][..],
        2,
    );

    let mut pathhashlist = PathHashList::from_path_recursive(dir.path(), false, false)
        .expect("Can't create PathHashList");

    assert!(pathhashlist.compute_hash().is_ok());

    // Hash of various empty files in tree structure:
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/a/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/a/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/a/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/a/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/a/y/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/b/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/b/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/b/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/b/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_create_from_path_recursively_no_root/b/y/1
    //
    // -> a425a6aa4a9585eeee8cd7f2398c0cefd4f5e25228b71d3161872618993d19b8
    assert_eq!(pathhashlist.hash().unwrap(), b"\xa4\x25\xa6\xaa\x4a\x95\x85\xee\xee\x8c\xd7\xf2\x39\x8c\x0c\xef\xd4\xf5\xe2\x52\x28\xb7\x1d\x31\x61\x87\x26\x18\x99\x3d\x19\xb8");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn create_from_path_recursively_with_root() {
    let dir = common::creating_tempdir(None, 2, &["a", "b"][..], 1, &["x", "y"][..], 2);

    let mut pathhashlist = PathHashList::from_path_recursive(dir.path(), true, false)
        .expect("Can't create PathHashList");

    assert!(pathhashlist.compute_hash().is_ok());

    // Hash of various empty files in tree structure:
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/y/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1
    //
    // -> 725fa4e7c9d48001e1ff3a453d7edd51a8bbe9390c06b64393e06518461adfd5
    assert_eq!(pathhashlist.hash().unwrap(), b"\x72\x5f\xa4\xe7\xc9\xd4\x80\x01\xe1\xff\x3a\x45\x3d\x7e\xdd\x51\xa8\xbb\xe9\x39\x0c\x06\xb6\x43\x93\xe0\x65\x18\x46\x1a\xdf\xd5");

    dir.close().expect("Can't close tempdir");
}
