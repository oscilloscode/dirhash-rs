use std::{
    fs::{self, File},
    io::Write,
    os::unix,
    path::Path,
};
use tempfile::TempDir;

fn create_numbered_files(dir: impl AsRef<Path>, n: usize, add_random_data: bool) {
    for i in 0..n {
        let mut f =
            File::create(dir.as_ref().join(format!("{}", i))).expect("Error while creating file");

        if add_random_data == true {
            let mut data = [0u8; 32];
            rand::fill(&mut data);
            f.write_all(&data).expect("Can't write random data to file");
        }
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
///
/// Resulting file count: L1F + L1D * (L2F + L2D * L3F)
pub fn creating_tempdir(
    dir_name: Option<String>,
    l1_files: usize,
    l1_dirs: &[&str],
    l2_files: usize,
    l2_dirs: &[&str],
    l3_files: usize,
    add_random_data: bool,
) -> TempDir {
    let dir = match dir_name {
        Some(dir_name) => tempfile::Builder::new()
            // .keep(true)
            .rand_bytes(0)
            .prefix(&dir_name)
            .tempdir()
            .expect("Can't create tempdir"),
        None => tempfile::Builder::new()
            // .keep(true)
            .tempdir()
            .expect("Can't create tempdir"),
    };

    create_numbered_files(&dir, l1_files, add_random_data);

    for d in l1_dirs.iter() {
        let dir_level_1 = dir.path().join(d.to_string());
        std::fs::create_dir(&dir_level_1)
            .expect(&format!("Error while creating directory {:?}", dir_level_1));

        create_numbered_files(&dir_level_1, l2_files, add_random_data);

        for d in l2_dirs.iter() {
            let dir_level_2 = dir_level_1.join(d.to_string());
            std::fs::create_dir(&dir_level_2)
                .expect(&format!("Error while creating directory {:?}", dir_level_2));

            create_numbered_files(&dir_level_2, l3_files, add_random_data);
        }
    }

    dir
}

// Creates a TempDir with file and directory link, going up- and downwards, respectively. The two
// files used as file link targets also have their relative path set as their contents.
//
// .
// ├── 0
// ├── 1
// ├── a
// │   ├── 0
// │   ├── 1
// │   ├── downwards_dirlink -> /tmp/.tmp6en1HI/b/x
// │   ├── x
// │   │   ├── 0
// │   │   └── 1
// │   └── y
// │       ├── 0
// │       └── 1
// ├── b
// │   ├── 0
// │   ├── 1
// │   ├── x
// │   │   ├── 0
// │   │   ├── 1
// │   │   └── upwards_dirlink -> /tmp/.tmp6en1HI/a/y
// │   └── y
// │       ├── 0
// │       ├── 1
// │       └── upwards_link -> /tmp/.tmp6en1HI/1
// └── downwards_link -> /tmp/.tmp6en1HI/a/0
pub fn create_tempdir_with_links() -> TempDir {
    let dir = creating_tempdir(None, 2, &["a", "b"][..], 2, &["x", "y"][..], 2, false);

    fs::write(dir.path().join("a/0"), "a/0").expect("Can't write to tempfile");
    fs::write(dir.path().join("1"), "1").expect("Can't write to tempfile");

    fs::write(dir.path().join("b/x/0"), "b/x/0").expect("Can't write to tempfile");
    fs::write(dir.path().join("b/x/1"), "b/x/1").expect("Can't write to tempfile");

    fs::write(dir.path().join("a/y/0"), "a/y/0").expect("Can't write to tempfile");
    fs::write(dir.path().join("a/y/1"), "a/y/1").expect("Can't write to tempfile");

    // file downwards
    unix::fs::symlink(dir.path().join("a/0"), dir.path().join("downwards_link"))
        .expect("Error while creating symlink");

    // file upwards
    unix::fs::symlink(dir.path().join("1"), dir.path().join("b/y/upwards_link"))
        .expect("Error while creating symlink");

    // dir downwards
    unix::fs::symlink(
        dir.path().join("b/x"),
        dir.path().join("a/downwards_dirlink"),
    )
    .expect("Error while creating symlink");

    // dir upwards
    unix::fs::symlink(
        dir.path().join("a/y"),
        dir.path().join("b/x/upwards_dirlink"),
    )
    .expect("Error while creating symlink");

    dir
}
