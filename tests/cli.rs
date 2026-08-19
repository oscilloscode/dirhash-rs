use std::fs;
use std::io::Write;
use std::process::Command;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

use tempfile::NamedTempFile;

mod common;

#[test]
pub fn missing_cmd() {
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.assert().failure();
}

#[test]
pub fn help() {
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.arg("-h");
    cmd.assert().success().stdout(
        predicates::str::contains("Usage")
            .and(predicates::str::contains("Commands"))
            .and(predicates::str::contains("Options")),
    );
}

#[test]
pub fn list() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_list")),
        4,
        &["a", "b", "c"][..],
        3,
        &["s", "t"][..],
        2,
        false,
    );
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"0
1
2
3
a/0
a/1
a/2
a/s/0
a/s/1
a/t/0
a/t/1
b/0
b/1
b/2
b/s/0
b/s/1
b/t/0
b/t/1
c/0
c/1
c/2
c/s/0
c/s/1
c/t/0
c/t/1
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn list_absolute_flag() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_list_absolute_flag")),
        2,
        &["d", "e"][..],
        1,
        &["x", "y"][..],
        1,
        false,
    );

    // Relative paths
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"0
1
d/0
d/x/0
d/y/0
e/0
e/x/0
e/y/0
"#,
    );

    // Abolute paths
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap(), "-a"]);
    cmd.assert().success().stdout(
        r#"/tmp/.tmp_cli_list_absolute_flag/0
/tmp/.tmp_cli_list_absolute_flag/1
/tmp/.tmp_cli_list_absolute_flag/d/0
/tmp/.tmp_cli_list_absolute_flag/d/x/0
/tmp/.tmp_cli_list_absolute_flag/d/y/0
/tmp/.tmp_cli_list_absolute_flag/e/0
/tmp/.tmp_cli_list_absolute_flag/e/x/0
/tmp/.tmp_cli_list_absolute_flag/e/y/0
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn list_hidden_flag() {
    let dir = tempfile::Builder::new()
        // .keep(true)
        .rand_bytes(0)
        .prefix(".tmp_cli_list_hidden_flag")
        .tempdir()
        .expect("Can't create tempdir");

    let datafile_path = dir.path().join("datafile");
    std::fs::write(datafile_path, b"data").expect("Can't write to tempfile");

    let hidden_path = dir.path().join(".hidden");
    std::fs::write(hidden_path, b"hidden").expect("Can't write to tempfile");

    // Ignoring hidden files
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"datafile

Ignored files:
./.hidden: Hidden
"#,
    );

    // Including hidden files
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap(), "-H"]);
    cmd.assert().success().stdout(".hidden\ndatafile\n");

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn list_symlink_flag() {
    let dir = common::create_tempdir_with_links(Some(String::from(".tmp_cli_list_symlink_flag")));

    // Not following symlinks and ignoring them
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"0
1
a/0
a/1
a/x/0
a/x/1
a/y/0
a/y/1
b/0
b/1
b/x/0
b/x/1
b/y/0
b/y/1

Ignored files:
./a/downwards_dirlink: Symlink
./b/x/upwards_dirlink: Symlink
./b/y/upwards_link: Symlink
./downwards_link: Symlink
"#,
    );

    // Following symlinks
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap(), "-L"]);
    cmd.assert().success().stdout(
        r#"0
1
a/0
a/1
a/downwards_dirlink/0
a/downwards_dirlink/1
a/downwards_dirlink/upwards_dirlink/0
a/downwards_dirlink/upwards_dirlink/1
a/x/0
a/x/1
a/y/0
a/y/1
b/0
b/1
b/x/0
b/x/1
b/x/upwards_dirlink/0
b/x/upwards_dirlink/1
b/y/0
b/y/1
b/y/upwards_link
downwards_link
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn list_invalid_flag() {
    let dir = common::create_tempdir_with_links_to_invalid(Some(String::from(
        ".tmp_cli_list_invalid_flag",
    )));

    // Not following symlinks, and thus no invalid files encountered
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"0
1
2
d/0
d/1
d/r/0
d/r/1
d/r/2
d/s/0
d/s/1
d/s/2
d/t/0
d/t/1
d/t/2
e/0
e/1
e/r/0
e/r/1
e/r/2
e/s/0
e/s/1
e/s/2
e/t/0
e/t/1
e/t/2

Ignored files:
./block_device_link: Symlink
./d/r/socket_link: Symlink
./d/s/fifo_link: Symlink
./e/char_device_link: Symlink
"#,
    );

    // Following symlinks -> invalid files found -> panic
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap(), "-L"]);
    cmd.assert().failure().stdout("").stderr(
        predicates::str::contains("panicked").and(predicates::str::contains("InvalidFileType")),
    );

    // Following symlinks -> invalid files found, but ignored
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["list", dir.path().to_str().unwrap(), "-LI"]);
    cmd.assert().success().stdout(
        r#"0
1
2
d/0
d/1
d/r/0
d/r/1
d/r/2
d/s/0
d/s/1
d/s/2
d/t/0
d/t/1
d/t/2
e/0
e/1
e/r/0
e/r/1
e/r/2
e/s/0
e/s/1
e/s/2
e/t/0
e/t/1
e/t/2

Ignored files:
./block_device_link: BlockDevice
./d/r/socket_link: Socket
./d/s/fifo_link: FIFO
./e/char_device_link: CharDevice
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn summary() {
    let dir = common::creating_tempdir(
        None,
        2,
        &["aa", "mm", "xx"][..],
        4,
        &["c", "w"][..],
        1,
        false,
    );
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["summary", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r"Regular files: 20
Hidden files: 0
Symlinks: 0
Block devices: 0
Char devices: 0
FIFOs: 0
Sockets: 0
",
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn summary_with_hidden() {
    let dir = common::creating_tempdir(
        None,
        2,
        &["aa", "mm", "xx"][..],
        4,
        &["c", "w"][..],
        1,
        false,
    );

    std::fs::write(dir.path().join("aa/.hidden"), b"aa/.hidden").expect("Can't write to tempfile");
    std::fs::write(dir.path().join("mm/w/.hidden"), b"mm/w/.hidden")
        .expect("Can't write to tempfile");
    std::fs::write(dir.path().join("xx/c/.hidden"), b"xx/c/.hidden")
        .expect("Can't write to tempfile");

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["summary", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r"Regular files: 20
Hidden files: 3
Symlinks: 0
Block devices: 0
Char devices: 0
FIFOs: 0
Sockets: 0
",
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn summary_with_links() {
    let dir = common::create_tempdir_with_links(None);

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["summary", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r"Regular files: 14
Hidden files: 0
Symlinks: 4
Block devices: 0
Char devices: 0
FIFOs: 0
Sockets: 0
",
    );

    dir.close().expect("Can't close tempdir");
}

// TODO: Currently only FIFOs are used as proxy for all invalid file types. Creating block or char
// devices with mknod would be possible, but is very dangerous. A real device needs to be specified
// and could be accessed by the test or the code under test. The trick with symlinks to real
// block/char devices can't be used as `follow_symlinks` is turned off so that links can get counted
// as well.
#[test]
pub fn summary_with_invalid() {
    let dir = common::creating_tempdir(None, 3, &["G", "R"][..], 4, &["U", "B"][..], 4, false);

    let _ = Command::new("mkfifo")
        .current_dir(&dir)
        .arg("fifo")
        .output()
        .expect("Error while creating FIFO");
    let _ = Command::new("mkfifo")
        .current_dir(dir.path().join("G/B"))
        .arg("another_fifo")
        .output()
        .expect("Error while creating FIFO");

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["summary", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r"Regular files: 27
Hidden files: 0
Symlinks: 0
Block devices: 0
Char devices: 0
FIFOs: 2
Sockets: 0
",
    );

    dir.close().expect("Can't close tempdir");
}

// Same reasoning for the invalid files as `summary_with_invalid`.
#[test]
pub fn summary_with_mixed() {
    let dir = common::create_tempdir_with_links(None);

    std::fs::write(dir.path().join(".hidden"), b".hidden").expect("Can't write to tempfile");
    std::fs::write(dir.path().join("a/.hidden"), b"a/.hidden").expect("Can't write to tempfile");
    std::fs::write(dir.path().join("a/y/.hidden"), b"a/y/.hidden")
        .expect("Can't write to tempfile");
    std::fs::write(dir.path().join("b/x/.hidden"), b"b/x/.hidden")
        .expect("Can't write to tempfile");

    let _ = Command::new("mkfifo")
        .current_dir(&dir)
        .arg("fifo")
        .output()
        .expect("Error while creating FIFO");
    let _ = Command::new("mkfifo")
        .current_dir(dir.path().join("a"))
        .arg("another_fifo")
        .output()
        .expect("Error while creating FIFO");

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["summary", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r"Regular files: 14
Hidden files: 4
Symlinks: 4
Block devices: 0
Char devices: 0
FIFOs: 2
Sockets: 0
",
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_analyze")),
        2,
        &["a", "b"][..],
        1,
        &["g", "h"][..],
        3,
        false,
    );
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/g/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/g/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/g/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/h/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/h/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/h/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/g/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/g/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/g/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/h/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/h/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/h/2

a9cb27164c9537cadde55e6d31fabd0a4befbf1b558d2e236476f7e40a215f6c
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze_absolute_flag() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_analyze_absolute_flag")),
        1,
        &["c"][..],
        4,
        &["d", "e"][..],
        2,
        false,
    );

    // Relative paths
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_absolute_flag",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/3
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/d/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./c/e/1

7c877196d585f1aaeecc234b88e38fcce3a9575468797db690be51a02f4bbfc2
"#,
    );

    // Absolute paths
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap(), "-a"]);
    cmd.assert().success().stdout(r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_absolute_flag",
#   "absolute": true,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/3
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/d/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/.tmp_cli_analyze_absolute_flag/c/e/1

f1d0290ddb06f66dadc8fb37c467cc7e2a121b6681207bccbabae69d2fdca986
"#);

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze_hidden_flag() {
    let dir = tempfile::Builder::new()
        // .keep(true)
        .rand_bytes(0)
        .prefix(".tmp_cli_analyze_hidden_flag")
        .tempdir()
        .expect("Can't create tempdir");

    let datafile_path = dir.path().join("datafile");
    std::fs::write(datafile_path, b"data").expect("Can't write to tempfile");

    let hidden_path = dir.path().join(".hidden");
    std::fs::write(hidden_path, b"hidden").expect("Can't write to tempfile");

    // Ignoring hidden files
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_hidden_flag",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

3a6eb0790f39ac87c94f3856b2dd2c5d110e6811602261a9a923d3bb23adc8b7  ./datafile

86d88597af112e3e996b83a609e2c925c9ab66e961a8d41d0e4bb1eec868a448

Ignored files:
./.hidden: Hidden
"#,
    );

    // Including hidden files
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap(), "-H"]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_hidden_flag",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": true,
#   "ignore_invalid_filetypes": false
# }

3a6eb0790f39ac87c94f3856b2dd2c5d110e6811602261a9a923d3bb23adc8b7  ./datafile
e564b4081d7a9ea4b00dada53bdae70c99b87b6fce869f0c3dd4d2bfa1e53e1c  ./.hidden

342783ce4b379555a17df24f20188be18f439674a5aa790357106eafc77f91c4
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze_symlink_flag() {
    let dir =
        common::create_tempdir_with_links(Some(String::from(".tmp_cli_analyze_symlink_flag")));

    // Not following symlinks and ignoring them
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_symlink_flag",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./b/x/1
3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/y/0
601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/y/1
6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./1
a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./b/x/0
d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./a/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1

86d6b064dcf498615435a879221a1a2d76b969dc67cbd3c8fd7f35f767cb8e10

Ignored files:
./a/downwards_dirlink: Symlink
./b/x/upwards_dirlink: Symlink
./b/y/upwards_link: Symlink
./downwards_link: Symlink
"#,
    );

    // Following symlinks
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap(), "-L"]);
    cmd.assert().success().stdout(r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_symlink_flag",
#   "absolute": false,
#   "follow_symlinks": true,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./a/downwards_dirlink/1
2c1e9c3dc66c67faa7bcbddb69f4d2fb70cfffc2ca0188c3a8b2a0b757310c83  ./b/x/1
3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/downwards_dirlink/upwards_dirlink/0
3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./a/y/0
3b57e943f5f5d6649657683d4625b5512c745d010537379548285946b2d4b791  ./b/x/upwards_dirlink/0
601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/downwards_dirlink/upwards_dirlink/1
601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./a/y/1
601bde2d34fb40a2b4f9ff019e5ce3b662b2ecbd0de84a5470f6dd3791293750  ./b/x/upwards_dirlink/1
6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./1
6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b  ./b/y/upwards_link
a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./a/downwards_dirlink/0
a99f8bcdeef5f422a751b59057c24d001232640796069fe9655157de31068943  ./b/x/0
d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./a/0
d7e98967056f4828cb388a7930d88594b59e4374a7927afdd93890273682c804  ./downwards_link
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./a/x/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./b/y/1

a9ae7427d5341a8dfe933b118fb440d69b630f45d290930af2ea9d2a93316a6b
"#);

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze_invalid_flag() {
    let dir = common::create_tempdir_with_links_to_invalid(Some(String::from(
        ".tmp_cli_analyze_invalid_flag",
    )));

    // Not following symlinks, and thus no invalid files encountered
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_invalid_flag",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/2

4778b420c8834f6e833db5be5ecab1864f2d3740b576f790fe7376fe43ab096d

Ignored files:
./block_device_link: Symlink
./d/r/socket_link: Symlink
./d/s/fifo_link: Symlink
./e/char_device_link: Symlink
"#,
    );

    // Following symlinks -> invalid files found -> panic
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap(), "-L"]);
    cmd.assert().failure().stdout("").stderr(
        predicates::str::contains("panicked").and(predicates::str::contains("InvalidFileType")),
    );

    // Following symlinks -> invalid files found, but ignored
    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["analyze", dir.path().to_str().unwrap(), "-LI"]);
    cmd.assert().success().stdout(
        r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_invalid_flag",
#   "absolute": false,
#   "follow_symlinks": true,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": true
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/r/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/s/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/t/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/r/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/s/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./e/t/2

4778b420c8834f6e833db5be5ecab1864f2d3740b576f790fe7376fe43ab096d

Ignored files:
./block_device_link: BlockDevice
./d/r/socket_link: Socket
./d/s/fifo_link: FIFO
./e/char_device_link: CharDevice
"#,
    );

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn analyze_writes_fingerprint_file() {
    let expected_output_first_run = r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_writes_fingerprint_file",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/2

a690bc7d5293543f8f1303b7084adb572e6a4cb09072ac024feb5a26b01ae0c3
"#;

    let expected_output_second_run = r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_analyze_writes_fingerprint_file",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

ab38d927fd45923fdec2aaae3c2c258f415d108e65e4c376e6938051bc48a19a  ./finger
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./h/j/2
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./i/j/2

ec776c447825f880e04c010393c19179b5c472fe32682a941194d1c93a895c6a
"#;

    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_analyze_writes_fingerprint_file")),
        3,
        &["h", "i"][..],
        1,
        &["j"][..],
        3,
        false,
    );

    // first run (only "normal" temp files)
    let mut cmd = cargo_bin_cmd!("dirhash");
    let fingerprint_path = dir.path().join("finger");
    assert!(!fingerprint_path.exists());
    cmd.args(&[
        "analyze",
        dir.path().to_str().unwrap(),
        "-f",
        fingerprint_path.to_str().unwrap(),
    ]);
    cmd.assert().success().stdout(expected_output_first_run);

    assert!(fingerprint_path.exists());
    let finger_content = fs::read_to_string(fingerprint_path).expect("Can't read fingerprint file");
    assert_eq!(finger_content, expected_output_first_run);

    // second run (now including fingerprint file from first run)
    let mut cmd = cargo_bin_cmd!("dirhash");
    let fingerprint_path = dir.path().join("finger2");
    assert!(!fingerprint_path.exists());
    cmd.args(&[
        "analyze",
        dir.path().to_str().unwrap(),
        "-f",
        fingerprint_path.to_str().unwrap(),
    ]);
    cmd.assert().success().stdout(expected_output_second_run);

    assert!(fingerprint_path.exists());
    let finger_content = fs::read_to_string(fingerprint_path).expect("Can't read fingerprint file");
    assert_eq!(finger_content, expected_output_second_run);

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn verify() {
    let dir = common::creating_tempdir(
        Some(String::from(".tmp_cli_verify")),
        1,
        &["d"][..],
        1,
        &["e", "f"][..],
        2,
        false,
    );

    let mut fingerprint_file =
        NamedTempFile::new().expect("Can't create temporary fingerprint file");

    let finger_content = r#"# {
#   "version": 1,
#   "path": "/tmp/.tmp_cli_verify",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/1

d54869b935d36b0260556f9d283c92c32d2b44ccba9c0c5f6a7bf69183650b4e
"#;

    write!(fingerprint_file, "{}", finger_content).unwrap();

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["verify", fingerprint_file.path().to_str().unwrap()]);
    cmd.assert().success();

    dir.close().expect("Can't close tempdir");
}

#[test]
pub fn verify_bad_version() {
    let mut fingerprint_file =
        NamedTempFile::new().expect("Can't create temporary fingerprint file");

    let finger_content = r#"# {
#   "version": 2,
#   "path": "/does/not/exist",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/1

d54869b935d36b0260556f9d283c92c32d2b44ccba9c0c5f6a7bf69183650b4e
"#;

    write!(fingerprint_file, "{}", finger_content).unwrap();

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["verify", fingerprint_file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout("")
        .stderr(
            predicates::str::contains("panicked").and(predicates::str::contains(
                "Currently, only fingerprints with version \"1\" are supported!",
            )),
        );
}

#[test]
pub fn verify_path_doesnt_exist() {
    let mut fingerprint_file =
        NamedTempFile::new().expect("Can't create temporary fingerprint file");

    let finger_content = r#"# {
#   "version": 1,
#   "path": "/does/not/exist",
#   "absolute": false,
#   "follow_symlinks": false,
#   "include_hidden_files": false,
#   "ignore_invalid_filetypes": false
# }

e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/e/1
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./d/f/1

d54869b935d36b0260556f9d283c92c32d2b44ccba9c0c5f6a7bf69183650b4e
"#;

    write!(fingerprint_file, "{}", finger_content).unwrap();

    let mut cmd = cargo_bin_cmd!("dirhash");
    cmd.args(&["verify", fingerprint_file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout("")
        .stderr(
            predicates::str::contains("panicked").and(predicates::str::contains(
                "code: 2, kind: NotFound, message: \"No such file or directory\"",
            )),
        );
}
