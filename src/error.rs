use std::path::PathBuf;
use thiserror::Error;

// Should common derives like these also be used on such simple enum types?
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum InvalidFileTypeKind {
    Dir,
    BlockDevice,
    CharDevice,
    FIFO,
    Socket,
}

#[derive(Error, Debug)]
pub enum DirHashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PathHash: Invalid filetype: {0:?}")]
    InvalidFileType(InvalidFileTypeKind, PathBuf),
    #[error("HashTableEntry: conversion from a slice to an array failed")]
    HashTableEntry(#[from] std::array::TryFromSliceError),
    #[error("Walkdir: Error while walking directory")]
    WalkDir(#[from] walkdir::Error),
    #[error("DirHash: Mismatched roots")]
    RootMismatch(#[from] std::path::StripPrefixError),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, DirHashError>;
