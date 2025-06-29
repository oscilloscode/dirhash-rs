use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirHashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, DirHashError>;
