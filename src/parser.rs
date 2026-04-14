use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, BufReader, ReadBuf};

use crate::Result;
use crate::event::{CrocEvent, FileInfo, Progress};

/// A `croc` parser that can serialize outputs from the provided `reader`.
pub struct CrocParser<R: AsyncBufRead + Unpin> {
    /// The buffered reader used to read output.
    reader: BufReader<R>,
    /// Tracks whether the current transfer is a receiving operation.
    is_receiving: bool,
    /// Internal buffer for reading lines.
    buf: Vec<u8>,
}

impl<R: AsyncBufRead + Unpin> CrocParser<R> {
    /// Creates a new croc output parser for the provided reader.
    pub fn new(inner: R) -> Self {
        let reader = BufReader::new(inner);
        Self {
            reader,
            is_receiving: false,
            buf: Vec::new(),
        }
    }

    /// Polls the next event available in the output
    pub fn poll_next_event(&mut self, cx: &mut Context<'_>) -> Poll<Result<CrocEvent>> {
        loop {
            let mut byte = [0u8; 1];
            let mut read_buf = ReadBuf::new(&mut byte);
            match Pin::new(&mut self.reader).poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) => {
                    if read_buf.filled().is_empty() {
                        if !self.buf.is_empty() {
                            let raw_line = String::from_utf8_lossy(&self.buf).to_string();
                            self.buf.clear();
                            let line = Self::strip_ansi(&raw_line);
                            if !line.trim().is_empty() {
                                if let Some(event) = self.parse_line(&line) {
                                    return Poll::Ready(Ok(event));
                                }
                            }
                        }
                        return Poll::Ready(Ok(CrocEvent::EOF));
                    }
                    let b = byte[0];
                    if b == b'\r' || b == b'\n' {
                        if self.buf.is_empty() {
                            continue;
                        }
                        let raw_line = String::from_utf8_lossy(&self.buf).to_string();
                        self.buf.clear();
                        let line = Self::strip_ansi(&raw_line);
                        if line.trim().is_empty() {
                            continue;
                        }
                        if let Some(event) = self.parse_line(&line) {
                            return Poll::Ready(Ok(event));
                        }
                    } else {
                        self.buf.push(b);
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    /// Parses a single line of output from croc and returns the corresponding `CrocEvent`.
    fn parse_line(&mut self, line: &str) -> Option<CrocEvent> {
        let line = line.trim();

        if Self::should_ignore(line) {
            return None;
        }

        if let Some(event) = self.parse_hashing(line) {
            return Some(event);
        }

        if let Some(event) = self.parse_info(line) {
            return Some(event);
        }

        if let Some(event) = self.parse_code(line) {
            return Some(event);
        }

        if let Some(event) = self.parse_state_change(line) {
            return Some(event);
        }

        if let Some(event) = self.parse_progress(line) {
            return Some(event);
        }

        Some(CrocEvent::Unknown(line.to_string()))
    }

    /// Determines if a specific line of output from `croc` should be completely ignored.
    fn should_ignore(line: &str) -> bool {
        line == "On the other computer run:"
            || line == "(For Windows)"
            || line == "(For Linux/macOS)"
            || line.starts_with("croc ")
            || line.starts_with("CROC_SECRET=")
    }

    /// Attempts to parse a line containing progress for a "Hashing" operation.
    fn parse_hashing(&self, line: &str) -> Option<CrocEvent> {
        if !line.starts_with("Hashing ") {
            return None;
        }

        let percent_idx = line.find("% |")?;
        let left = line[8..percent_idx].trim();
        let last_space = left.rfind(' ')?;

        let file_name = left[..last_space].trim().to_string();
        let percent_str = left[last_space..].trim();
        let percentage = percent_str.parse::<u8>().ok()?;

        let (bytes_sent, bytes_total, speed) = Self::parse_metadata(line);

        Some(CrocEvent::Hashing(Progress {
            file_name,
            percentage,
            bytes_sent,
            bytes_total,
            speed,
        }))
    }

    /// Attempts to parse lines describing file transfer info, updating the receiving state.
    fn parse_info(&mut self, line: &str) -> Option<CrocEvent> {
        if line.starts_with("Sending '") {
            let end_quote = line[9..].find('\'')?;
            let end_quote_idx = 9 + end_quote;
            let name = line[9..end_quote_idx].to_string();

            let start_paren = line[end_quote_idx..].find('(')?;
            let start_paren_idx = end_quote_idx + start_paren + 1;
            let end_paren = line[start_paren_idx..].find(')')?;

            let size_str = line[start_paren_idx..start_paren_idx + end_paren].to_string();
            let size = Self::parse_bytes(&size_str).unwrap_or(0);
            self.is_receiving = false;
            return Some(CrocEvent::SendingInfo(FileInfo { name, size }));
        }

        if line.starts_with("Receiving '") {
            let end_quote = line[11..].find('\'')?;
            let end_quote_idx = 11 + end_quote;
            let name = line[11..end_quote_idx].to_string();

            let start_paren = line[end_quote_idx..].find('(')?;
            let start_paren_idx = end_quote_idx + start_paren + 1;
            let end_paren = line[start_paren_idx..].find(')')?;

            let size_str = line[start_paren_idx..start_paren_idx + end_paren].to_string();
            let size = Self::parse_bytes(&size_str).unwrap_or(0);
            self.is_receiving = true;
            return Some(CrocEvent::ReceivingInfo(FileInfo { name, size }));
        }

        None
    }

    /// Attempts to parse a line indicating the connection code is ready.
    fn parse_code(&self, line: &str) -> Option<CrocEvent> {
        if line.starts_with("Code is: ") {
            let code = line[9..].trim().to_string();
            return Some(CrocEvent::CodeGenerated(code));
        }
        None
    }

    /// Attempts to parse lines that toggle the receiving/sending state without other direct info.
    fn parse_state_change(&mut self, line: &str) -> Option<CrocEvent> {
        if line.starts_with("Receiving (") {
            self.is_receiving = true;
            if let Some(relay) = Self::parse_relay_address(line, "<-") {
                return Some(CrocEvent::ReceivingFrom(relay));
            }
            return Some(CrocEvent::Unknown(line.to_string()));
        } else if line.starts_with("Sending (") {
            self.is_receiving = false;
            if let Some(relay) = Self::parse_relay_address(line, "->") {
                return Some(CrocEvent::SendingTo(relay));
            }
            return Some(CrocEvent::Unknown(line.to_string()));
        }
        None
    }

    /// Attempts to extract a relay address from a state change line using the given delimiter (`<-` or `->`).
    fn parse_relay_address(line: &str, delimiter: &str) -> Option<crate::croc::Relay> {
        let start = line.find(delimiter)?;
        let end = line.rfind(')')?;
        let ip_port = &line[start + 2..end];
        let addr = ip_port.parse::<std::net::SocketAddr>().ok()?;
        Some(crate::croc::Relay::new(addr.ip(), addr.port()))
    }

    /// Attempts to parse file transfer progress updates.
    fn parse_progress(&self, line: &str) -> Option<CrocEvent> {
        let percent_idx = line.find("% |")?;
        let left = line[..percent_idx].trim();
        let last_space = left.rfind(' ')?;

        let file_name = left[..last_space].trim().to_string();
        let percent_str = left[last_space..].trim();
        let percentage = percent_str.parse::<u8>().ok()?;

        let (bytes_sent, bytes_total, speed) = Self::parse_metadata(line);

        let progress = Progress {
            file_name,
            percentage,
            bytes_sent,
            bytes_total,
            speed,
        };

        if self.is_receiving {
            Some(CrocEvent::Receiving(progress))
        } else {
            Some(CrocEvent::Sending(progress))
        }
    }

    /// Extracts extra progress metadata (bytes sent, bytes total, transfer speed) from trailing parentheses.
    fn parse_metadata(line: &str) -> (Option<u64>, Option<u64>, Option<f64>) {
        let mut bytes_sent = None;
        let mut bytes_total = None;
        let mut speed = None;

        if let Some(paren_idx) = line.rfind('(') {
            let meta_str = line[paren_idx + 1..].trim_end_matches(')').trim();
            let parts: Vec<&str> = meta_str.splitn(2, ',').collect();

            let bytes_part = parts[0].trim();
            if let Some(slash_idx) = bytes_part.find('/') {
                let sent_str = bytes_part[..slash_idx].trim();
                let total_str = bytes_part[slash_idx + 1..].trim();

                let total_unit = if let Some(idx) = total_str.find(|c: char| c.is_alphabetic()) {
                    &total_str[idx..]
                } else {
                    ""
                };

                let sent_has_unit = sent_str.contains(|c: char| c.is_alphabetic());
                let sent_to_parse = if !sent_has_unit && !total_unit.is_empty() {
                    format!("{} {}", sent_str, total_unit)
                } else {
                    sent_str.to_string()
                };

                bytes_sent = Self::parse_bytes(&sent_to_parse);
                bytes_total = Self::parse_bytes(total_str);
            } else {
                bytes_sent = Self::parse_bytes(bytes_part);
            }

            if parts.len() > 1 {
                speed = Self::parse_speed(parts[1].trim());
            }
        }

        (bytes_sent, bytes_total, speed)
    }

    /// Converts a string unit (like KB, MB, GB) into its corresponding byte multiplier.
    fn parse_unit_multiplier(unit: &str) -> f64 {
        match unit.trim().to_uppercase().as_str() {
            "B" | "" => 1.0,
            "KB" | "K" => 1024.0,
            "MB" | "M" => 1024.0 * 1024.0,
            "GB" | "G" => 1024.0 * 1024.0 * 1024.0,
            "TB" | "T" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => 1.0,
        }
    }

    /// Strips ANSI escape sequences from a string to ensure clean parsing.
    fn strip_ansi(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut in_ansi = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_ansi = true;
            } else if in_ansi {
                if c.is_ascii_alphabetic() {
                    in_ansi = false;
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Attempts to parse a string containing a byte count and an optional unit into a raw `u64` representing bytes.
    fn parse_bytes(s: &str) -> Option<u64> {
        let s = s.trim();
        let (num_str, unit) = if let Some(idx) = s.find(|c: char| c.is_alphabetic()) {
            (&s[..idx], &s[idx..])
        } else {
            (s, "")
        };

        let val: f64 = num_str.trim().parse().ok()?;
        let multiplier = Self::parse_unit_multiplier(unit);
        Some((val * multiplier) as u64)
    }

    /// Attempts to parse a string containing a transfer speed and an optional unit into a raw `f64`.
    fn parse_speed(s: &str) -> Option<f64> {
        let s = s.trim().trim_end_matches("/s").trim_end_matches("ps");
        let (num_str, unit) = if let Some(idx) = s.find(|c: char| c.is_alphabetic()) {
            (&s[..idx], &s[idx..])
        } else {
            (s, "")
        };

        let val: f64 = num_str.trim().parse().ok()?;
        let multiplier = Self::parse_unit_multiplier(unit);
        Some(val * multiplier)
    }
}
