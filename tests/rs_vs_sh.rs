//! Things to check:
//! - Compare outputs from rs/sh with random data

use std::process::Command;

use dirhash_rs::pathhashlist::PathHashList;

mod common;

#[test]
fn sh_with_command() {
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

    let mut pathhashlist = PathHashList::from_path_recursive(dir.path(), true, false)
        .expect("Can't create PathHashList");

    assert!(pathhashlist.compute_hash().is_ok());

    // sh implementation
    // ------------------

    let hash_list_output = Command::new("bash")
        .current_dir(&dir)
        .env("LC_ALL", "C")
        .arg("-c")
        .arg("fd -t f --exec sha256sum | sort")
        .output()
        .expect("Command failed");
    let sh_hashtable_str = String::from_utf8_lossy(&hash_list_output.stdout);
    eprintln!("{}", &sh_hashtable_str);

    // Inefficient (recalculation), but shouldn't be a problem for tests
    let rec_hash_output = Command::new("bash")
        .current_dir(&dir)
        .env("LC_ALL", "C")
        .arg("-c")
        .arg("fd -t f --exec sha256sum | sort | sha256sum")
        .output()
        .expect("Command failed");
    let rec_hash = String::from_utf8_lossy(&rec_hash_output.stdout);

    let sh_hash_str = rec_hash
        .split_whitespace()
        .next()
        .expect("Couldn't extract the hash string from the sh output");

    eprintln!("{}", &sh_hash_str);

    // Verification
    // ------------

    let rs_hash_str = hex::encode(pathhashlist.hash().unwrap());
    let rs_hashtable_str = pathhashlist.hashtable().unwrap().to_string();

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
