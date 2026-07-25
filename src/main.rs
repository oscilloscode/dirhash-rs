// Functionality:
//
// dh list: list all files
// dh analyze: analyze file and create a fingerprint
// dh verify: verify the fingerprint
//

use std::{
    env::current_dir,
    fmt::Write,
    fs::{self, File},
    io::{BufRead, BufReader},
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand};
use dirhash_rs::dirhash::DirHash;
use pathdiff::diff_paths;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use walkdir::WalkDir;

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
struct WalkOptions {
    /// Use absolute paths (instead of relative)
    #[arg(short, long)]
    absolute: bool,

    /// Follow symbolic links
    #[arg(short = 'L', long = "follow")]
    follow_symlinks: bool,

    /// Include hidden files
    #[arg(short = 'H', long = "hidden")]
    include_hidden_files: bool,

    /// Ignore invalid filetypes
    #[arg(short = 'I', long = "ignore_invalid")]
    ignore_invalid_filetypes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct FingerprintMetadata {
    version: u8,
    path: PathBuf,
    #[serde(flatten)]
    walk: WalkOptions,
}

#[derive(Debug, Parser)]
#[command(name = "DirHash")]
#[command(version = "0.1")]
#[command(about = "Compute a fingerprint over all files in a directory recursively", long_about = None)]
struct DirhashCli {
    #[command(subcommand)]
    command: Commands,

    // TODO: this doesn't do anything yet!
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
        #[command(flatten)]
        walk: WalkOptions,
        /// Display the type of the listed files
        #[arg(short = 't', long = "test")]
        display_type: bool,
    },
    /// Analyze the files recursively and create a fingerprint
    Analyze {
        /// Path to analyze (default: cwd)
        path: Option<PathBuf>,
        #[command(flatten)]
        walk: WalkOptions,
        /// Path to fingerprint file
        #[arg(short, long)]
        fingerprint: Option<PathBuf>,
    },
    /// Verify the fingerprint of files recursively
    Verify {
        /// Path to fingerprint file
        fingerprint: PathBuf,
    },
}

fn parse_user_path(cwd: &Path, user_path: Option<PathBuf>) -> PathBuf {
    info!("path param: {:?}", &user_path);
    let path = cwd.join(user_path.unwrap_or(PathBuf::from(".")));
    debug!("path before canonicalize: {:?}", &path);
    let canon_path = path.canonicalize();
    info!("canon path: {:?}", canon_path);

    let canon_path = canon_path.expect("Supplied path doesn't exist");

    if !canon_path.is_dir() {
        panic!("Supplied path is not a directory");
    }

    canon_path
}

fn main() {
    // let _ = tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .with_file(true)
    //     .with_target(false)
    //     .init();

    let cwd = current_dir().expect("Can't get current working directory");

    let args = DirhashCli::parse();

    debug!("parsed args: {:?}", args);

    match args.command {
        Commands::List {
            path,
            walk,
            display_type,
        } => {
            let path = parse_user_path(&cwd, path);
            list_files(path, display_type, walk, args.paranoid);
        }
        Commands::Analyze {
            path,
            walk,
            fingerprint,
        } => {
            let path = parse_user_path(&cwd, path);
            analyze_files(path, fingerprint, walk, args.paranoid);
        }
        Commands::Verify { fingerprint } => {
            verify_files(fingerprint, args.paranoid);
        }
    }
}

fn list_files(path: PathBuf, display_type: bool, walk: WalkOptions, paranoid: bool) {
    info!("Listing files:");
    debug!("Path: {:?}", path);
    debug!("Display file types: {:?}", display_type);
    debug!("Absolute paths: {:?}", walk.absolute);
    debug!("Follow symlinks: {:?}", walk.follow_symlinks);
    debug!("Include hidden files: {:?}", walk.include_hidden_files);
    debug!(
        "Ignore invalid filetypes: {:?}",
        walk.ignore_invalid_filetypes
    );
    debug!("Paranoid mode: {:?}", paranoid);

    // TODO replace with code from dirhash. if there is a bug in the file discovery which leads to
    // more/less files being included, this wouldn't show it.

    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.unwrap();
        info!(
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

fn calculate_fingerprint(meta: FingerprintMetadata, paranoid: bool) -> String {
    let mut fingerprint = String::new();

    let meta_serialized = serde_json::to_string_pretty(&meta).expect("Can't serialize metadata");

    let mut commented_meta = String::new();

    for line in meta_serialized.lines() {
        commented_meta.push_str("# ");
        commented_meta.push_str(line);
        commented_meta.push('\n');
    }

    writeln!(&mut fingerprint, "{commented_meta}")
        .expect("Can't write commented metadata to string buffer");

    let mut dh = DirHash::new()
        .with_files_from_dir(
            &meta.path,
            !meta.walk.absolute,
            meta.walk.follow_symlinks,
            meta.walk.include_hidden_files,
            meta.walk.ignore_invalid_filetypes,
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
            let relative_path = (!meta.walk.absolute).then(|| {
                PathBuf::from(".").join(
                    diff_paths(ignored_path, &meta.path)
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
    info!("Analyzing files:");
    debug!("Path: {:?}", path);
    debug!("Fingerprint path: {:?}", fingerprint_path);
    debug!("Absolute paths: {:?}", walk.absolute);
    debug!("Follow symlinks: {:?}", walk.follow_symlinks);
    debug!("Include hidden files: {:?}", walk.include_hidden_files);
    debug!(
        "Ignore invalid filetypes: {:?}",
        walk.ignore_invalid_filetypes
    );
    debug!("Paranoid mode: {:?}", paranoid);

    let meta = FingerprintMetadata {
        version: 1,
        path: path.clone(),
        walk: walk.clone(),
    };

    let fingerprint = calculate_fingerprint(meta, paranoid);

    print!("{}", fingerprint);

    if let Some(path) = fingerprint_path.as_ref() {
        fs::write(path, fingerprint).expect("Can't write to fingerprint file");
    }
}

fn verify_files(fingerprint_path: PathBuf, paranoid: bool) {
    info!("Verifying files:");
    debug!("Fingerprint path: {:?}", fingerprint_path);
    debug!("Paranoid mode: {:?}", paranoid);

    let filetype = fs::metadata(&fingerprint_path)
        .expect("Can't read metadata of fingerprint file")
        .file_type();

    if !filetype.is_file() {
        panic!("Fingerprint path is not a file!");
    }

    let f = File::open(&fingerprint_path).expect("Can't open fingerprint file");
    let reader = BufReader::new(f);

    let meta_serialized = reader
        .lines()
        .map(|line| line.expect("Can't read line"))
        .take_while(|line| line.starts_with("# "))
        // .map(|line| line.strip_prefix("# ").unwrap())
        .map(|line| line.strip_prefix("# ").unwrap().to_owned())
        .collect::<Vec<_>>()
        .join("\n");

    // let meta = reader
    //     .lines()
    //     .map(|line| line.expect("Can't read line").strip_prefix("# "))
    //     .take_while(Option::is_some)
    //     .flat_map(Option::unwrap)
    //     // .map(|line| line.strip_prefix("# ").unwrap())
    //     // .map(|line| line.strip_prefix("# ").unwrap().to_owned())
    //     .collect::<Vec<_>>()
    //     .join("\n");

    debug!("meta_serialized = {meta_serialized}");

    let meta: FingerprintMetadata = serde_json::from_str(&meta_serialized).unwrap();

    debug!("meta = {meta:?}");

    if meta.version != 1 {
        panic!("Currently, only fingerprints with version \"1\" are supported!")
    }

    let fingerprint = calculate_fingerprint(meta, paranoid);

    print!("Calculated fingerprint:\n{}", fingerprint);

    let file_contents = fs::read_to_string(fingerprint_path).expect("Can't read fingerprint file");

    print!("Fingerprint file:\n{}", file_contents);

    if fingerprint != file_contents {
        panic!("Calculated fingerprint doesn't match fingerprint file!");
    }
}
