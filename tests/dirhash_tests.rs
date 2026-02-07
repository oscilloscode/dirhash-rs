//! Things to check:
//! - Creating DirHash from list of paths, then compute hash
//! - Add additional list of paths after creation , then compute hash
//! - Return error when encountering illegal filetype (char dev, block dev, socket, pipe)

use std::{
    fs::{self, File},
    io::Write,
    os::unix::{self, fs::FileTypeExt},
    path::Path,
};

use dirhash_rs::{
    dirhash::{DirHash, IgnoreReason},
    error::{DirHashError, InvalidFileTypeKind},
};
use tempfile::tempdir;

mod common;

#[test]
fn with_files_from_dir_dont_follow_symlinks() {
    let dir = common::create_tempdir_with_links();

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(
        dh.ignored(),
        vec![
            (
                dir.path().join("a/downwards_dirlink"),
                IgnoreReason::Symlink
            ),
            (
                dir.path().join("b/x/upwards_dirlink"),
                IgnoreReason::Symlink
            ),
            (dir.path().join("b/y/upwards_link"), IgnoreReason::Symlink),
            (dir.path().join("downwards_link"), IgnoreReason::Symlink)
        ]
    );

    assert!(dh.compute_hash().is_ok());

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./b/x/1\n\
         3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/y/0\n\
         601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/y/1\n\
         6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./1\n\
         a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./b/x/0\n\
         d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./a/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1\n"
    );

    assert_eq!(
        dh.hash().unwrap(),
        b"\x86\xd6\xb0\x64\xdc\xf4\x98\x61\x54\x35\xa8\x79\x22\x1a\x1a\x2d\x76\xb9\x69\xdc\x67\xcb\xd3\xc8\xfd\x7f\x35\xf7\x67\xcb\x8e\x10"
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_follow_symlinks() {
    let dir = common::create_tempdir_with_links();

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
    assert!(dh.compute_hash().is_ok());

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./a/downwards_dirlink/1\n\
         2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./b/x/1\n\
         3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/downwards_dirlink/upwards_dirlink/0\n\
         3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/y/0\n\
         3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./b/x/upwards_dirlink/0\n\
         601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/downwards_dirlink/upwards_dirlink/1\n\
         601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/y/1\n\
         601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./b/x/upwards_dirlink/1\n\
         6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./1\n\
         6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./b/y/upwards_link\n\
         a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./a/downwards_dirlink/0\n\
         a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./b/x/0\n\
         d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./a/0\n\
         d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./downwards_link\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1\n"
    );

    assert_eq!(
        dh.hash().unwrap(),
        b"\xa9\xae\x74\x27\xd5\x34\x1a\x8d\xfe\x93\x3b\x11\x8f\xb4\x40\xd6\x9b\x63\x0f\x45\xd2\x90\x93\x0a\xf2\xea\x9d\x2a\x93\x31\x6a\x6b"
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_include_hidden_files() {
    let dir = tempdir().expect("Can't create tempdir");
    // let dir = tempfile::Builder::new()
    //     .keep(true)
    //     .tempdir()
    //     .expect("Can't create tempdir");
    //
    let datafile_path = dir.path().join("datafile");
    let mut file = File::create(&datafile_path).expect("Error while creating file");

    write!(&mut file, "{}", "test data").expect("Can't write to tempfile");

    let hidden_path = dir.path().join(".hidden");
    let mut file = File::create(&hidden_path).expect("Error while creating hidden file");

    write!(&mut file, "{}", "test data").expect("Can't write to tempfile");

    // Hidden files shall be included by default when with_files_from_dir is refactored to builder
    // pattern
    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
    assert!(dh.compute_hash().is_ok());

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9  ./.hidden\n\
         916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9  ./datafile\n"
    );

    assert_eq!(dh.hash().unwrap(), b"\x3b\x0b\x63\x45\x90\x7e\x1f\xc0\xca\x87\x30\x3f\x16\xc6\x47\xe5\x0e\x29\xd4\xa2\x30\xfa\x45\x11\xe7\x01\xc2\xed\x66\xb5\x95\x20");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_ignore_hidden_files() {
    let dir = tempdir().expect("Can't create tempdir");
    // let dir = tempfile::Builder::new()
    //     .keep(true)
    //     .tempdir()
    //     .expect("Can't create tempdir");

    let datafile_path = dir.path().join("datafile");
    let mut file = File::create(&datafile_path).expect("Error while creating file");

    write!(&mut file, "{}", "test data").expect("Can't write to tempfile");

    let hidden_path = dir.path().join(".hidden");
    let mut file = File::create(&hidden_path).expect("Error while creating hidden file");

    write!(&mut file, "{}", "test data").expect("Can't write to tempfile");

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, false, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored(), vec![(hidden_path, IgnoreReason::Hidden)]);

    assert!(dh.compute_hash().is_ok());

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9  ./datafile\n"
    );

    assert_eq!(dh.hash().unwrap(), b"\x0e\x5b\x09\x6d\x50\x7d\x3f\xeb\xf1\x3c\xf2\x7b\x36\x1e\x0b\x4c\x64\x7b\x08\x43\x0e\x22\x45\xeb\xbf\xa1\x86\x06\x72\x17\xa8\xf9");

    dir.close().expect("Can't close tempdir");
}

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
        .with_files_from_dir(dir.path(), false, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
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
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
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

#[test]
fn with_file_from_dir_no_root() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_with_file_from_dir_no_root")),
        3,
        &["c", "d"][..],
        2,
        &["x", "y", "z"][..],
        1,
        false,
    );

    // Add data to files
    fs::write(dir.path().join("2"), b"hallo\n").expect("Error while adding data to test file");
    fs::write(dir.path().join("d/y/0"), b"apple\nbread\ncherry\n")
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("1"), &[0xCC, 0xCC, 0xCC, 0xCC])
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("c/0"), b"Mario\n").expect("Error while adding data to test file");
    fs::write(
        dir.path().join("d/0"),
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ],
    )
    .expect("Error while adding data to test file");
    fs::write(dir.path().join("c/z/0"), b"DirHash\n")
        .expect("Error while adding data to test file");

    fs::write(dir.path().join("c/y/0"), b"hallo\n").expect("Error while adding data to test file");
    fs::write(dir.path().join("0"), b"apple\nbread\ncherry\n")
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("d/x/0"), &[0xCC, 0xCC, 0xCC, 0xCC])
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("d/1"), b"Mario\n").expect("Error while adding data to test file");
    fs::write(
        dir.path().join("c/1"),
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ],
    )
    .expect("Error while adding data to test file");
    fs::write(dir.path().join("d/z/0"), b"DirHash\n")
        .expect("Error while adding data to test file");

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), false, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
    assert!(dh.compute_hash().is_ok());

    // Hash of various files in tree structure:
    // 622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  /tmp/.tmp_with_file_from_dir_no_root/2
    // 622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  /tmp/.tmp_with_file_from_dir_no_root/c/y/0
    // 7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  /tmp/.tmp_with_file_from_dir_no_root/0
    // 7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  /tmp/.tmp_with_file_from_dir_no_root/d/y/0
    // 8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  /tmp/.tmp_with_file_from_dir_no_root/1
    // 8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  /tmp/.tmp_with_file_from_dir_no_root/d/x/0
    // 9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  /tmp/.tmp_with_file_from_dir_no_root/c/0
    // 9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  /tmp/.tmp_with_file_from_dir_no_root/d/1
    // be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  /tmp/.tmp_with_file_from_dir_no_root/c/1
    // be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  /tmp/.tmp_with_file_from_dir_no_root/d/0
    // d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  /tmp/.tmp_with_file_from_dir_no_root/c/z/0
    // d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  /tmp/.tmp_with_file_from_dir_no_root/d/z/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root/c/x/0
    //
    // -> f24512317d0e3287e797f7858801d984c5a005b64d56686b58d6aa53bcf53d69
    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  /tmp/.tmp_with_file_from_dir_no_root/2\n\
         622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  /tmp/.tmp_with_file_from_dir_no_root/c/y/0\n\
         7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  /tmp/.tmp_with_file_from_dir_no_root/0\n\
         7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  /tmp/.tmp_with_file_from_dir_no_root/d/y/0\n\
         8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  /tmp/.tmp_with_file_from_dir_no_root/1\n\
         8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  /tmp/.tmp_with_file_from_dir_no_root/d/x/0\n\
         9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  /tmp/.tmp_with_file_from_dir_no_root/c/0\n\
         9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  /tmp/.tmp_with_file_from_dir_no_root/d/1\n\
         be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  /tmp/.tmp_with_file_from_dir_no_root/c/1\n\
         be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  /tmp/.tmp_with_file_from_dir_no_root/d/0\n\
         d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  /tmp/.tmp_with_file_from_dir_no_root/c/z/0\n\
         d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  /tmp/.tmp_with_file_from_dir_no_root/d/z/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_with_file_from_dir_no_root/c/x/0\n"
    );
    assert_eq!(dh.hash().unwrap(), b"\xf2\x45\x12\x31\x7d\x0e\x32\x87\xe7\x97\xf7\x85\x88\x01\xd9\x84\xc5\xa0\x05\xb6\x4d\x56\x68\x6b\x58\xd6\xaa\x53\xbc\xf5\x3d\x69");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_with_root() {
    let dir = common::creating_tempdir(None, 3, &["c", "d"][..], 2, &["x", "y", "z"][..], 1, false);

    // Add data to files
    fs::write(dir.path().join("0"), b"hallo\n").expect("Error while adding data to test file");
    fs::write(dir.path().join("d/1"), b"apple\nbread\ncherry\n")
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("d/x/0"), &[0xCC, 0xCC, 0xCC, 0xCC])
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("c/z/0"), b"Mario\n").expect("Error while adding data to test file");
    fs::write(
        dir.path().join("d/0"),
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ],
    )
    .expect("Error while adding data to test file");
    fs::write(dir.path().join("c/1"), b"DirHash\n").expect("Error while adding data to test file");

    fs::write(dir.path().join("c/x/0"), b"hallo\n").expect("Error while adding data to test file");
    fs::write(dir.path().join("d/y/0"), b"apple\nbread\ncherry\n")
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("1"), &[0xCC, 0xCC, 0xCC, 0xCC])
        .expect("Error while adding data to test file");
    fs::write(dir.path().join("d/z/0"), b"Mario\n").expect("Error while adding data to test file");
    fs::write(
        dir.path().join("2"),
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ],
    )
    .expect("Error while adding data to test file");
    fs::write(dir.path().join("c/0"), b"DirHash\n").expect("Error while adding data to test file");

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);

    assert!(dh.compute_hash().is_ok());

    // Hash of various empty files in tree structure:
    //
    // 622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  ./0
    // 622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  ./c/x/0
    // 7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  ./d/1
    // 7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  ./d/y/0
    // 8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  ./1
    // 8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  ./d/x/0
    // 9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  ./c/z/0
    // 9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  ./d/z/0
    // be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  ./2
    // be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  ./d/0
    // d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  ./c/0
    // d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  ./c/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/y/0
    //
    // -> 64eabf7ded6f1b974c5a2666ed43d3b1dfc7dbc2c289ede9180b6bbd3b223307
    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  ./0\n\
         622cb3371c1a08096eaac564fb59acccda1fcdbe13a9dd10b486e6463c8c2525  ./c/x/0\n\
         7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  ./d/1\n\
         7fb428bf33bb1103b3a1afa22fe5fb77aa2ec5d008d3552cd2bf946f6184ff20  ./d/y/0\n\
         8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  ./1\n\
         8843b54d2df63ca265cf4a05d27dd2b29a74fb476d296dd44a0e171d74b441ca  ./d/x/0\n\
         9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  ./c/z/0\n\
         9013413f4c27d86ae4e9854eacecba0122aa110ec8b423a2ea1f1d8f50375358  ./d/z/0\n\
         be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  ./2\n\
         be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991  ./d/0\n\
         d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  ./c/0\n\
         d5cc1967a4e009550ae53ef65169bb638734cb43352653645ee8f23ccfefe416  ./c/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/y/0\n"
    );
    assert_eq!(dh.hash().unwrap(), b"\x64\xea\xbf\x7d\xed\x6f\x1b\x97\x4c\x5a\x26\x66\xed\x43\xd3\xb1\xdf\xc7\xdb\xc2\xc2\x89\xed\xe9\x18\x0b\x6b\xbd\x3b\x22\x33\x07");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_invalid_filetypes() {
    let dir = common::creating_tempdir(None, 3, &["d", "e"][..], 2, &["r", "s", "t"][..], 3, false);

    // Adding links to invalid filetypes, as this is significantly easier than creating them all.
    // When activating "follow_links", walkdir returns the type of the target file instead of the
    // "link" file type for the link.

    // block device
    let sda_path = Path::new("/dev/sda");
    let sda_metadata = fs::metadata(sda_path).expect("Can't get metadata of /dev/sda");
    assert!(sda_metadata.file_type().is_block_device());

    let block_dev_link = dir.path().join("block_device_link");
    unix::fs::symlink(sda_path, &block_dev_link).expect("Error while creating symlink");

    let err = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect_err("Block device in files didn't result in error");

    match err {
        DirHashError::InvalidFileType(filetype, _) => match filetype {
            InvalidFileTypeKind::BlockDevice => {}
            _ => panic!("Wrong InvalidFileType enum variant"),
        },
        _ => panic!("Wrong DirHashError enum variant"),
    }

    fs::remove_file(block_dev_link).expect("Error while deleting symlink");

    // character device
    let dev_null_path = Path::new("/dev/null");
    let dev_null_metadata = fs::metadata(dev_null_path).expect("Can't get metadata of /dev/null");
    assert!(dev_null_metadata.file_type().is_char_device());

    let char_dev_link = dir.path().join("d/s/char_device_link");
    unix::fs::symlink(dev_null_path, &char_dev_link).expect("Error while creating symlink");

    let err = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect_err("Char device in files didn't result in error");

    match err {
        DirHashError::InvalidFileType(filetype, _) => match filetype {
            InvalidFileTypeKind::CharDevice => {}
            _ => panic!("Wrong InvalidFileType enum variant"),
        },
        _ => panic!("Wrong DirHashError enum variant"),
    }

    fs::remove_file(char_dev_link).expect("Error while deleting symlink");

    // fifo
    // Is this a good file? Do all Linux distros have this?
    let initctl_path = Path::new("/run/initctl");
    let initctl_metadata = fs::metadata(initctl_path).expect("Can't get metadata of /run/initctl");
    assert!(initctl_metadata.file_type().is_fifo());

    let fifo_link = dir.path().join("e/fifo_link");
    unix::fs::symlink(initctl_path, &fifo_link).expect("Error while creating symlink");

    let err = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect_err("Fifo in files didn't result in error");

    match err {
        DirHashError::InvalidFileType(filetype, _) => match filetype {
            InvalidFileTypeKind::FIFO => {}
            _ => panic!("Wrong InvalidFileType enum variant"),
        },
        _ => panic!("Wrong DirHashError enum variant"),
    }

    fs::remove_file(fifo_link).expect("Error while deleting symlink");

    // socket
    // Is this a good file? Do all Linux distros have this?
    let systemd_private_path = Path::new("/run/systemd/private");
    let systemd_private_metadata =
        fs::metadata(systemd_private_path).expect("Can't get metadata of /run/systemd/private");
    assert!(systemd_private_metadata.file_type().is_socket());

    let socket_link = dir.path().join("e/r/socket_link");
    unix::fs::symlink(systemd_private_path, &socket_link).expect("Error while creating symlink");

    let err = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect_err("Socket in files didn't result in error");

    match err {
        DirHashError::InvalidFileType(filetype, _) => match filetype {
            InvalidFileTypeKind::Socket => {}
            _ => panic!("Wrong InvalidFileType enum variant"),
        },
        _ => panic!("Wrong DirHashError enum variant"),
    }

    fs::remove_file(socket_link).expect("Error while deleting symlink");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn with_files_from_dir_invalid_filetypes_ignore() {
    let dir = common::creating_tempdir(None, 3, &["d", "e"][..], 2, &["r", "s", "t"][..], 3, false);

    // Adding links to invalid filetypes, as this is significantly easier than creating them all.
    // When activating "follow_links", walkdir returns the type of the target file instead of the
    // "link" file type for the link.

    // block device
    let sda_path = Path::new("/dev/sda");
    let sda_metadata = fs::metadata(sda_path).expect("Can't get metadata of /dev/sda");
    assert!(sda_metadata.file_type().is_block_device());

    let block_dev_link = dir.path().join("block_device_link");
    unix::fs::symlink(sda_path, &block_dev_link).expect("Error while creating symlink");

    // character device
    let dev_null_path = Path::new("/dev/null");
    let dev_null_metadata = fs::metadata(dev_null_path).expect("Can't get metadata of /dev/null");
    assert!(dev_null_metadata.file_type().is_char_device());

    let char_dev_link = dir.path().join("e/char_device_link");
    unix::fs::symlink(dev_null_path, &char_dev_link).expect("Error while creating symlink");

    // fifo
    // Is this a good file? Do all Linux distros have this?
    let initctl_path = Path::new("/run/initctl");
    let initctl_metadata = fs::metadata(initctl_path).expect("Can't get metadata of /run/initctl");
    assert!(initctl_metadata.file_type().is_fifo());

    let fifo_link = dir.path().join("d/s/fifo_link");
    unix::fs::symlink(initctl_path, &fifo_link).expect("Error while creating symlink");

    // socket
    // Is this a good file? Do all Linux distros have this?
    let systemd_private_path = Path::new("/run/systemd/private");
    let systemd_private_metadata =
        fs::metadata(systemd_private_path).expect("Can't get metadata of /run/systemd/private");
    assert!(systemd_private_metadata.file_type().is_socket());

    let socket_link = dir.path().join("d/r/socket_link");
    unix::fs::symlink(systemd_private_path, &socket_link).expect("Error while creating symlink");

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, true)
        .expect("Can't create DirHash");

    assert_eq!(
        dh.ignored(),
        vec![
            (block_dev_link, IgnoreReason::BlockDevice),
            (socket_link, IgnoreReason::Socket),
            (fifo_link, IgnoreReason::FIFO),
            (char_dev_link, IgnoreReason::CharDevice),
        ]
    );

    assert!(dh.compute_hash().is_ok());

    // Hash of various empty files in tree structure:
    //
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/2
    //
    // -> 4778b420c8834f6e833db5be5ecab1864f2d3740b576f790fe7376fe43ab096d
    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/2\n"
    );
    assert_eq!(dh.hash().unwrap(), b"\x47\x78\xb4\x20\xc8\x83\x4f\x6e\x83\x3d\xb5\xbe\x5e\xca\xb1\x86\x4f\x2d\x37\x40\xb5\x76\xf7\x90\xfe\x73\x76\xfe\x43\xab\x09\x6d");

    dir.close().expect("Can't close tempdir");
}
