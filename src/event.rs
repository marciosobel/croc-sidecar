use tokio::io;

use crate::croc::Relay;

/// Represents progress of an ongoing operation.
#[derive(Debug, Clone)]
pub struct Progress {
    /// The name of the file being processed.
    pub file_name: String,
    /// The current progress percentage (0-100).
    pub percentage: u8,
    /// The number of bytes transferred so far, if available.
    pub bytes_sent: Option<u64>,
    /// The total number of bytes to transfer, if available.
    pub bytes_total: Option<u64>,
    /// The current transfer speed, if available.
    pub speed: Option<f64>,
}

/// Represents information about a file being transferred.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The name of the file.
    pub name: String,
    /// The size of the file in bytes.
    pub size: u64,
}

/// A collection representing an event that happened in the running `croc` application.
#[derive(Debug)]
pub enum CrocEvent {
    /// A hashing operation is in progress.
    Hashing(Progress),
    /// Information about a file being sent.
    SendingInfo(FileInfo),
    /// Information about a file being received.
    ReceivingInfo(FileInfo),
    /// The connection code has been generated.
    CodeGenerated(String),
    /// The transfer is sending data to the specified relay/IP.
    SendingTo(Relay),
    /// The transfer is receiving data from the specified relay/IP.
    ReceivingFrom(Relay),
    /// A file is currently being sent.
    Sending(Progress),
    /// A file is currently being received.
    Receiving(Progress),
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
