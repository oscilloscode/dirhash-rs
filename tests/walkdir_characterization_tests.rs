//! * Rationale
//!
//! These characterization tests intend to capture my assumptions about the walkdir crate and they function as a visual reference. Additionally, this prevents me from
//! overlooking a breaking change by resulting in test failures.

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn creating_tempdir() {
    let dir = tempdir().expect("Can't create tempdir");
    let file_path = dir.path().join("my-temporary-note.txt");
    let mut file = File::create(file_path).expect("Error while creating file");
    writeln!(file, "Brian was here. Briefly.").expect("Error while writing file");

    println!("{:?}", dir.path());
    // By closing the `TempDir` explicitly, we can check that it has
    // been deleted successfully. If we don't close it explicitly,
    // the directory will still be deleted when `dir` goes out
    // of scope, but we won't know whether deleting the directory
    // succeeded.
    drop(file);
    dir.close().expect("Can't close tempdir");
}
