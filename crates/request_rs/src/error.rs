use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WgetError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Parsing Error: {0}")]
    ParsingError(String),
    #[error("Invalid Status Code: {0}")]
    InvalidStatusCode(u16),
}

pub type WgetResult<T> = std::result::Result<T, WgetError>;
