//! Compute a hash over all files, list them in a hashtable (sorted), and finally hash the
//! hashtable, resulting in a single hash that fingerprints the file contents, their name, and their
//! location ("tree" structure).
//!
//! Corresponds to:
//!
//! `LC_ALL=C fd -a -t f $argv --exec sha256sum | sort | tee /dev/tty | sha256sum`
pub mod dirhash;

pub mod bash;
pub mod error;
pub mod hashtable;
pub mod pathhash;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_config;
