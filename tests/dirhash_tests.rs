//! Things to check:
//! - Creating DirHash recursively from path, then compute hash
//! - Creating DirHash from list of paths, then compute hash
//! - Add additional list of paths after creation , then compute hash
//! - Symlinks in specified files
//! - Return error when encountering illegal filetype (char dev, block dev, socket, pipe)

use dirhash_rs::dirhash::DirHash;

mod common;

#[test]
fn with_file_from_dir_no_root_empty_files() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_with_file_from_dir_no_root_empty_files")),
        2,
        &["a", "b"][..],
        1,
        &["x", "y"][..],
        2,
        false,
    );

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), false, false)
        .expect("Can't create DirHash");

    assert!(dh.compute_hash().is_ok());

    // Hash of various empty files in tree structure:
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/y/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/y/1
    //
    // -> 98e8bcf358050f530beeb52aa963152f593007b01f87fe06bfdb15c01834accb
    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/a/y/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root_empty_files/b/y/1\n"
    );
    assert_eq!(dh.hash().unwrap(), b"\x98\xe8\xbc\xf3\x58\x05\x0f\x53\x0b\xee\xb5\x2a\xa9\x63\x15\x2f\x59\x30\x07\xb0\x1f\x87\xfe\x06\xbf\xdb\x15\xc0\x18\x34\xac\xcb");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_with_root_empty_files() {
    let dir = common::creating_tempdir(None, 2, &["a", "b"][..], 1, &["x", "y"][..], 2, false);

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false)
        .expect("Can't create DirHash");

    assert!(dh.compute_hash().is_ok());

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
    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/y/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1\n"
    );
    assert_eq!(dh.hash().unwrap(), b"\x72\x5f\xa4\xe7\xc9\xd4\x80\x01\xe1\xff\x3a\x45\x3d\x7e\xdd\x51\xa8\xbb\xe9\x39\x0c\x06\xb6\x43\x93\xe0\x65\x18\x46\x1a\xdf\xd5");

    dir.close().expect("Can't close tempdir");
}
