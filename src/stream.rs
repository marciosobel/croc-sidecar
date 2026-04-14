use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Stream;
use tokio::{io::BufReader, process::ChildStderr};

use crate::{CrocChild, CrocEvent, CrocParser, Error, Result};

/// A wrapper that parses `croc` lines in a stream.
pub struct CrocEventStream {
    parser: CrocParser<BufReader<ChildStderr>>,
}

impl CrocEventStream {
    pub(crate) fn new(child: &mut CrocChild) -> Result<Self> {
        let stderr = child.take_stderr().ok_or(Error::NoStderr)?;
        let reader = BufReader::new(stderr);
        let parser = CrocParser::new(reader);

        Ok(Self { parser })
    }
}

impl Stream for CrocEventStream {
    type Item = CrocEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut().parser.poll_next_event(cx) {
            Poll::Ready(Ok(event)) => {
                if matches!(event, CrocEvent::EOF) {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(event))
                }
            }
            Poll::Ready(Err(error)) => match error {
                Error::IoError(error) => Poll::Ready(Some(error.into())),
                Error::NoStderr => unreachable!(),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
