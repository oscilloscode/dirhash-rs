// Functionality:
//
// dh list: list all files
// dh analyze: analyze file and create a fingerprint
// dh verify: verify the fingerprint
//

use std::{
    env::current_dir,
    fmt::Write,
    fs,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand};
use dirhash_rs::dirhash::DirHash;
use pathdiff::diff_paths;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Args, Clone, Serialize)]
struct WalkOptions {
    /// Use absolute paths (instead of relative)
    #[arg(short, long, global = true)]
    absolute: bool,

    /// Follow symbolic links
    #[arg(short = 'L', long = "follow", global = true)]
    follow_symlinks: bool,

    /// Include hidden files
    #[arg(short = 'H', long = "hidden", global = true)]
    include_hidden_files: bool,

    /// Ignore invalid filetypes
    #[arg(short = 'I', long = "ignore_invalid", global = true)]
    ignore_invalid_filetypes: bool,
}

#[derive(Debug, Parser)]
#[command(name = "DirHash")]
#[command(version = "0.1")]
#[command(about = "Compute a fingerprint over all files in a directory recursively", long_about = None)]
struct DirhashCli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    walk: WalkOptions,

    /// Run a shell-based implementation in parallel to double check the output
    #[arg(short, long, global = true)]
    paranoid: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List files
    List {
        /// Path to list files from (default: cwd)
        path: Option<PathBuf>,
        /// Display the type of the listed files
        #[arg(short = 't', long = "test")]
        display_type: bool,
    },
    /// Analyze the files recursively and create a fingerprint
    Analyze {
        /// Path to analyze (default: cwd)
        path: Option<PathBuf>,
        /// Path to fingerprint file
        #[arg(short, long)]
        fingerprint: Option<PathBuf>,
    },
    /// Verify the fingerprint of files recursively
    Verify {
        // TODO: remove!!!
        /// Path to analyze
        path: PathBuf,
        /// Path to fingerprint file
        fingerprint: PathBuf,
    },
}

fn parse_user_path(cwd: &Path, user_path: Option<PathBuf>) -> PathBuf {
    println!("path param: {:?}", &user_path);
    let path = cwd.join(user_path.unwrap_or(PathBuf::from(".")));
    println!("path before canonicalize: {:?}", &path);
    let canon_path = path.canonicalize();
    println!("canon path: {:?}", canon_path);

    let canon_path = canon_path.expect("Supplied path doesn't exist");

    if !canon_path.is_dir() {
        panic!("Supplied path is not a directory");
    }

    canon_path
}

fn main() {
    let cwd = current_dir().expect("Can't get current working directory");

    let args = DirhashCli::parse();

    println!("parsed args: {:?}", args);

    match args.command {
        Commands::List { path, display_type } => {
            let path = parse_user_path(&cwd, path);
            list_files(path, display_type, args.walk, args.paranoid);
        }
        Commands::Analyze { path, fingerprint } => {
            let path = parse_user_path(&cwd, path);
            analyze_files(path, fingerprint, args.walk, args.paranoid);
        }
        Commands::Verify { path, fingerprint } => {
            let path = parse_user_path(&cwd, Some(path));
            verify_files(path, fingerprint, args.walk, args.paranoid);
        }
    }
}

fn list_files(path: PathBuf, display_type: bool, walk: WalkOptions, paranoid: bool) {
    println!("Listing files:");
    println!("Path: {:?}", path);
    println!("Display file types: {:?}", display_type);
    println!("Absolute paths: {:?}", walk.absolute);
    println!("Follow symlinks: {:?}", walk.follow_symlinks);
    println!("Include hidden files: {:?}", walk.include_hidden_files);
    println!(
        "Ignore invalid filetypes: {:?}",
        walk.ignore_invalid_filetypes
    );
    println!("Paranoid mode: {:?}", paranoid);

    // TODO replace with code from dirhash. if there is a bug in the file discovery which leads to
    // more/less files being included, this wouldn't show it.

    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.unwrap();
        println!(
            "type: {:?} block: {} char: {} fifo: {} socket: {} path: {}",
            entry.file_type(),
            entry.file_type().is_block_device(),
            entry.file_type().is_char_device(),
            entry.file_type().is_fifo(),
            entry.file_type().is_socket(),
            entry.path().display()
        );
    }
}

fn calculate_fingerprint(path: PathBuf, walk: WalkOptions, paranoid: bool) -> String {
    let mut fingerprint = String::new();

    writeln!(&mut fingerprint, "# path={}", path.display())
        .expect("Can't write path to string buffer");

    let walk_value = serde_json::to_value(&walk).expect("Can't serialize to value");

    for (key, value) in walk_value.as_object().expect("Can't get object") {
        writeln!(&mut fingerprint, "# {key}={value}")
            .expect("Can't write walk options to string buffer")
    }

    writeln!(&mut fingerprint, "").expect("Can't write newline to string buffer");

    let mut dh = DirHash::new()
        .with_files_from_dir(
            &path,
            !walk.absolute,
            walk.follow_symlinks,
            walk.include_hidden_files,
            walk.ignore_invalid_filetypes,
        )
        .expect("Can't create DirHash");

    dh.compute_hash().expect("Error while computing hash");

    write!(
        &mut fingerprint,
        "{}\n{}\n",
        dh.hashtable().expect("Can't get hashtable").to_string(),
        hex::encode(dh.hash().expect("Can't get hash string"))
    )
    .expect("Can't write fingerprint to string buffer");

    if !dh.ignored().is_empty() {
        writeln!(&mut fingerprint, "\nIgnored files:")
            .expect("Can't write ignored files header to string buffer");

        for (ignored_path, reason) in dh.ignored() {
            let relative_path = (!walk.absolute).then(|| {
                PathBuf::from(".").join(
                    diff_paths(ignored_path, &path)
                        .expect("Can't create relative path for ignored file"),
                )
            });

            let ignored_path = relative_path.as_deref().unwrap_or(ignored_path.as_path());

            write!(
                &mut fingerprint,
                "{}: {:?}\n",
                ignored_path.display(),
                reason
            )
            .expect("Can't write ignored files to string buffer");
        }
    }

    fingerprint
}

fn analyze_files(
    path: PathBuf,
    fingerprint_path: Option<PathBuf>,
    walk: WalkOptions,
    paranoid: bool,
) {
    println!("Analyzing files:");
    println!("Path: {:?}", path);
    println!("Fingerprint path: {:?}", fingerprint_path);
    println!("Absolute paths: {:?}", walk.absolute);
    println!("Follow symlinks: {:?}", walk.follow_symlinks);
    println!("Include hidden files: {:?}", walk.include_hidden_files);
    println!(
        "Ignore invalid filetypes: {:?}",
        walk.ignore_invalid_filetypes
    );
    println!("Paranoid mode: {:?}", paranoid);

    let fingerprint = calculate_fingerprint(path, walk, paranoid);

    print!("{}", fingerprint);

    if let Some(path) = fingerprint_path.as_ref() {
        fs::write(path, fingerprint).expect("Can't write to fingerprint file");
    }
}

fn verify_files(path: PathBuf, fingerprint_path: PathBuf, walk: WalkOptions, paranoid: bool) {
    println!("Verifying files:");
    println!("Path: {:?}", path);
    println!("Fingerprint path: {:?}", fingerprint_path);
    println!("Absolute paths: {:?}", walk.absolute);
    println!("Follow symlinks: {:?}", walk.follow_symlinks);
    println!("Include hidden files: {:?}", walk.include_hidden_files);
    println!(
        "Ignore invalid filetypes: {:?}",
        walk.ignore_invalid_filetypes
    );
    println!("Paranoid mode: {:?}", paranoid);

    let filetype = fs::metadata(&fingerprint_path)
        .expect("Can't open fingerprint file")
        .file_type();

    if !filetype.is_file() {
        panic!("Fingerprint path is not a file!");
    }

    let fingerprint = calculate_fingerprint(path, walk, paranoid);

    print!("Calculated fingerprint:\n{}", fingerprint);

    let file_contents = fs::read_to_string(fingerprint_path).expect("Can't read fingerprint file");

    print!("Fingerprint file:\n{}", file_contents);

    if fingerprint != file_contents {
        panic!("Calculated fingerprint doesn't match fingerprint file!");
    }
}
