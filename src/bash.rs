use std::{path::Path, process::Command};

use tracing::{debug, info};

// Convenience function for computing hashtable and hash with bash (fd & sha256sum)
pub fn list_files_with_bash(
    dir: &Path,
    absolute: bool,
    follow_links: bool,
    include_hidden_files: bool,
)  -> String {
    let mut cmd = Command::new("bash");
    cmd.current_dir(&dir).env("LC_ALL", "C").arg("-c");

    let mut fd_args = String::new();

    if absolute {
        fd_args.push_str("--absolute-path ");
    }

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

    cmd.arg(format!("fd {} -t f | sort", fd_args));

    info!("Cmd: {:?}", cmd);

    let list_output = cmd.output().expect("Command failed");

    let list_output_str = String::from_utf8(list_output.stdout)
        .expect("Non UTF-8 path encountered")
        .trim_end()
        .to_string();
    debug!("Output:\n--->{}<---", &list_output_str);

    list_output_str
}

// Convenience function for computing hashtable and hash with bash (fd & sha256sum)
pub fn compute_recursive_hash_with_bash(
    dir: &Path,
    absolute: bool,
    follow_links: bool,
    include_hidden_files: bool,
) -> (String, String) {
    let mut cmd = Command::new("bash");
    cmd.current_dir(&dir).env("LC_ALL", "C").arg("-c");

    let mut fd_args = String::new();

    if absolute {
        fd_args.push_str("--absolute-path ");
    }

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

    info!("Cmd: {:?}", cmd);

    let hash_list_output = cmd.output().expect("Command failed");

    let sh_hashtable_str = String::from_utf8_lossy(&hash_list_output.stdout);
    debug!("{}", &sh_hashtable_str);

    // Inefficient (recalculation), but shouldn't be a problem for tests
    //
    // TODO: "echo" the previous output into sha256sum to remove recalculation
    let mut cmd = Command::new("bash");
    cmd.current_dir(&dir).env("LC_ALL", "C").arg("-c");

    cmd.arg(format!(
        "fd {} -t f --exec sha256sum | sort | sha256sum",
        fd_args
    ));

    info!("Cmd: {:?}", cmd);

    let rec_hash_output = cmd.output().expect("Command failed");
    let rec_hash = String::from_utf8_lossy(&rec_hash_output.stdout);

    let sh_hash_str = rec_hash
        .split_whitespace()
        .next()
        .expect("Couldn't extract the hash string from the sh output");

    debug!("{}", &sh_hash_str);

    (sh_hashtable_str.to_string(), sh_hash_str.to_string())
}

// Testing is done in the "rs_vs_sh.rs" integration test, as it was deemed to repetitive to simply
// check the output of the bash functions alone. The other test don't just compare the output of the
// bash functions to the Rust implementation (which could be equally wrong, resulting in passing
// tests), but also against the expected values. Furthermore, testing the bash implemenations
// requires temporary files/directories to be created, which is more an "integration-testing-thing".
#[cfg(test)]
mod tests { }
