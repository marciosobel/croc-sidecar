use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader, Lines};

use crate::Result;

use super::CrocEvent;

/// A `croc` parser that can serialize outputs from the provided `reader`.
pub struct CrocParser<R: AsyncBufRead + Unpin> {
    lines: Lines<BufReader<R>>,
}

impl<R: AsyncBufRead + Unpin> CrocParser<R> {
    /// Creates a new croc output parser for the provided reader.
    pub fn new(inner: R) -> Self {
        let reader = BufReader::new(inner);
        let lines = reader.lines();
        Self { lines }
    }

    /// Parses the next event available in the output
    pub async fn parse_next_event(&mut self) -> Result<CrocEvent> {
        if let Some(line) = self.lines.next_line().await? {
            Ok(CrocEvent::Unknown(line))
        } else {
            Ok(CrocEvent::EOF)
        }
    }
}
