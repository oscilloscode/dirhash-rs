use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirHashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
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
