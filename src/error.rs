use thiserror::Error;

#[derive(Debug, Error)]
pub enum IronDbError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("persistence error: {0}")]
    Persistence(String),
}

pub type Result<T> = std::result::Result<T, IronDbError>;
