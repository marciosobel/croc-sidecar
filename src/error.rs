use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tried to take stderr but it was not available")]
    NoStderr,
    #[error("IO error: {0}")]
    IoError(#[from] tokio::io::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;
