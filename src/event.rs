use tokio::io;

/// A collection representing an event that happened in the running `croc` application.
#[derive(Debug)]
pub enum CrocEvent {
    /// An unknown line for the parser has been received.
    Unknown(String),
    /// The process has reached EOF.
    EOF,
    /// An IO error occurred while parsing the line.
    IoError(io::Error),
}

impl From<io::Error> for CrocEvent {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}
