//! Things to check:
//! - Compare outputs from rs/sh with random data

use std::{
    fs::{self, File},
    io::Write,
    os::unix::{self, fs::FileTypeExt},
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

use dirhash_rs::dirhash::{DirHash, IgnoreReason};
use tempfile::tempdir;

mod common;

// Convenience function for computing hashtable and hash with sh (fd & sha256sum)
fn compute_hash_with_sh(
    dir: &Path,
    follow_links: bool,
    include_hidden_files: bool,
) -> (String, String) {
    let mut cmd = Command::new("bash");
    cmd.current_dir(&dir).env("LC_ALL", "C").arg("-c");

    let mut fd_args = String::new();

    if follow_links {
        // --follow will not only go into symlinked directories, but also follow symlinked files.
        // Then, the filetype of the target file is used when matching the "-t" flag. Thus, only the
        // type "file" (and not "link") should be taken into account. This behavior is similar to
        // following links and the resulting target types when using walkdir.
        fd_args.push_str("--follow ");
    }

    if include_hidden_files {
        fd_args.push_str("--hidden ");
    }

    cmd.arg(format!("fd {} -t f --exec sha256sum | sort", fd_args));

    eprintln!("Cmd: {:?}", cmd);

    let hash_list_output = cmd.output().expect("Command failed");

    let sh_hashtable_str = String::from_utf8_lossy(&hash_list_output.stdout);
    eprintln!("{}", &sh_hashtable_str);

    // Inefficient (recalculation), but shouldn't be a problem for tests
    let mut cmd = Command::new("bash");
    cmd.current_dir(&dir).env("LC_ALL", "C").arg("-c");

    cmd.arg(format!(
        "fd {} -t f --exec sha256sum | sort | sha256sum",
        fd_args
    ));

    eprintln!("Cmd: {:?}", cmd);

    let rec_hash_output = cmd.output().expect("Command failed");
    let rec_hash = String::from_utf8_lossy(&rec_hash_output.stdout);

    let sh_hash_str = rec_hash
        .split_whitespace()
        .next()
        .expect("Couldn't extract the hash string from the sh output");

    eprintln!("{}", &sh_hash_str);

    (sh_hashtable_str.to_string(), sh_hash_str.to_string())
}

#[test]
fn with_empty_files_and_check_lc_all_ordering() {
    // Setup
    // ------

    let dir = common::creating_tempdir(
        None,
        2,
        // specifically crafted to check if sorting with LC_ALL=C is working
        &["b,foo", "bc,pe", "bcd,ty"][..],
        1,
        &["x", "y"][..],
        2,
        false,
    );

    // rs implementation
    // ------------------

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert!(dh.compute_hash().is_ok());

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), false, false);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

    // Hash of various empty files in tree structure:
    //
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/y/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/y/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/x/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/x/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/y/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/y/1
    //
    // -> 6a4bcbda9920637f38d636ade37b28c81b638dee3ac8729819e39d63433fdc22
    assert_eq!(
        rs_hashtable_str,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b,foo/y/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bc,pe/y/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/x/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/x/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/y/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./bcd,ty/y/1\n"
    );
    assert_eq!(
        rs_hash_str,
        "6a4bcbda9920637f38d636ade37b28c81b638dee3ac8729819e39d63433fdc22"
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
fn ignoring_invalid_files() {
    // Setup
    // ------

    let dir = common::creating_tempdir(None, 3, &["d", "e"][..], 1, &["r", "s"][..], 2, false);

    // Adding links to invalid filetypes, as this is significantly easier than creating them all.
    // When activating "follow_links", walkdir returns the type of the target file instead of the
    // "link" file type for the link.

    // block device
    let sda_path = Path::new("/dev/sda");
    let sda_metadata = fs::metadata(sda_path).expect("Can't get metadata of /dev/sda");
    assert!(sda_metadata.file_type().is_block_device());

    let block_dev_link = dir.path().join("d/s/block_device_link");
    unix::fs::symlink(sda_path, &block_dev_link).expect("Error while creating symlink");

    // character device
    let dev_null_path = Path::new("/dev/null");
    let dev_null_metadata = fs::metadata(dev_null_path).expect("Can't get metadata of /dev/null");
    assert!(dev_null_metadata.file_type().is_char_device());

    let char_dev_link = dir.path().join("char_device_link");
    unix::fs::symlink(dev_null_path, &char_dev_link).expect("Error while creating symlink");

    // fifo
    // Is this a good file? Do all Linux distros have this?
    let initctl_path = Path::new("/run/initctl");
    let initctl_metadata = fs::metadata(initctl_path).expect("Can't get metadata of /run/initctl");
    assert!(initctl_metadata.file_type().is_fifo());

    let fifo_link = dir.path().join("d/fifo_link");
    unix::fs::symlink(initctl_path, &fifo_link).expect("Error while creating symlink");

    // socket
    // Is this a good file? Do all Linux distros have this?
    let systemd_private_path = Path::new("/run/systemd/private");
    let systemd_private_metadata =
        fs::metadata(systemd_private_path).expect("Can't get metadata of /run/systemd/private");
    assert!(systemd_private_metadata.file_type().is_socket());

    let socket_link = dir.path().join("d/r/socket_link");
    unix::fs::symlink(systemd_private_path, &socket_link).expect("Error while creating symlink");

    // rs implementation
    // ------------------

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, true)
        .expect("Can't create DirHash");

    assert_eq!(
        dh.ignored(),
        vec![
            (char_dev_link, IgnoreReason::CharDevice),
            (fifo_link, IgnoreReason::FIFO),
            (socket_link, IgnoreReason::Socket),
            (block_dev_link, IgnoreReason::BlockDevice),
        ]
    );

    assert!(dh.compute_hash().is_ok());

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), true, false);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

    // Hash of various empty files in tree structure:
    //
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0
    // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1
    //
    // -> 4c05901d5193745590fb20d2ccea6ba2360950149a7b10977d0b56adf156d8f9
    assert_eq!(
        rs_hashtable_str,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0\n\
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1\n"
    );
    assert_eq!(
        rs_hash_str,
        "4c05901d5193745590fb20d2ccea6ba2360950149a7b10977d0b56adf156d8f9"
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
fn following_symlinks() {
    // Setup
    // ------

    let dir = common::create_tempdir_with_links();

    // rs implementation
    // ------------------

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, true, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
    assert!(dh.compute_hash().is_ok());

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), true, false);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

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
fn not_following_symlinks() {
    // Setup
    // ------

    let dir = common::create_tempdir_with_links();

    // rs implementation
    // ------------------

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

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), false, false);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

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
fn including_hidden_files() {
    // Setup
    // ------

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

    write!(&mut file, "{}", "hidden test data").expect("Can't write to tempfile");

    // rs implementation
    // ------------------

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, true, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored().len(), 0);
    assert!(dh.compute_hash().is_ok());

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), false, true);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "2a5fe7861edde7d25b095fb793743c343ee075069cf0c66db8a2587dc84a0710  ./.hidden\n\
         916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9  ./datafile\n"
    );

    assert_eq!(dh.hash().unwrap(), b"\xa4\x55\x43\xdc\x9c\x0e\x28\xcf\x4e\xbf\x10\xe9\x52\x7a\x0d\xa0\x6f\x8f\x37\x7e\x38\x2a\x29\x72\x98\x9a\xb6\x66\xb1\x23\x64\x28");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn ignoring_hidden_files() {
    // Setup
    // ------

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

    write!(&mut file, "{}", "hidden test data").expect("Can't write to tempfile");

    // rs implementation
    // ------------------

    let mut dh = DirHash::new()
        .with_files_from_dir(dir.path(), true, false, false, false)
        .expect("Can't create DirHash");

    assert_eq!(dh.ignored(), vec![(hidden_path, IgnoreReason::Hidden)]);
    assert!(dh.compute_hash().is_ok());

    let rs_hash_str = hex::encode(dh.hash().unwrap());
    let rs_hashtable_str = dh.hashtable().unwrap().to_string();

    // sh implementation
    // ------------------
    let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), false, false);

    // Verification
    // ------------

    assert_eq!(sh_hash_str, rs_hash_str);
    assert_eq!(sh_hashtable_str, rs_hashtable_str);

    assert_eq!(
        dh.hashtable().unwrap().to_string(),
        "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9  ./datafile\n"
    );

    assert_eq!(dh.hash().unwrap(), b"\x0e\x5b\x09\x6d\x50\x7d\x3f\xeb\xf1\x3c\xf2\x7b\x36\x1e\x0b\x4c\x64\x7b\x08\x43\x0e\x22\x45\xeb\xbf\xa1\x86\x06\x72\x17\xa8\xf9");

    dir.close().expect("Can't close tempdir");
}

#[test]
fn comparing_rs_sh_with_random_data() {
    const TEST_MAX_DURATION: Duration = Duration::from_secs(1);
    const TEST_MIN_FILES: usize = 1;
    const TEST_MAX_FILES: usize = 5;

    // Setup
    // ------
    let start = Instant::now();

    while start.elapsed() < TEST_MAX_DURATION {
        let dir = common::creating_tempdir(
            None,
            rand::random_range(TEST_MIN_FILES..=TEST_MAX_FILES),
            // specifically crafted to check if sorting with LC_ALL=C is working
            &["b,foo", "bc,pe", "bcd,ty"][..],
            rand::random_range(TEST_MIN_FILES..=TEST_MAX_FILES),
            &["x", "y"][..],
            rand::random_range(TEST_MIN_FILES..=TEST_MAX_FILES),
            true,
        );

        // rs implementation
        // ------------------

        let mut dh = DirHash::new()
            .with_files_from_dir(dir.path(), true, false, true, false)
            .expect("Can't create DirHash");

        assert!(dh.compute_hash().is_ok());

        let rs_hash_str = hex::encode(dh.hash().unwrap());
        let rs_hashtable_str = dh.hashtable().unwrap().to_string();

        // sh implementation
        // ------------------
        let (sh_hashtable_str, sh_hash_str) = compute_hash_with_sh(dir.path(), false, false);

        // Verification
        // ------------
        assert_eq!(sh_hash_str, rs_hash_str);
        assert_eq!(sh_hashtable_str, rs_hashtable_str);

        let duration = start.elapsed();
        eprintln!("Time elapsed: {:?}\n\n", duration);

        dir.close().expect("Can't close tempdir");
    }
}
