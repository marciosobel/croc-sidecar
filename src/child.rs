use tokio::{
    io,
    process::{Child, ChildStderr, ChildStdout},
};

use super::{CrocEventStream, Result};

/// An active `croc` instance.
pub struct CrocChild {
    /// The inner [`Child`].
    inner: Child,
}

impl CrocChild {
    /// Creates a new [`CrocChild`] from the provided [`Child`].
    pub fn new(inner: Child) -> Self {
        Self { inner }
    }

    /// Takes ownership of the child's stderr. Mutually exclusive with `iter()`,
    /// which relies on stderr for event parsing.
    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.inner.stderr.take()
    }

    /// Takes ownership of the child's stdout.
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.inner.stdout.take()
    }

    /// Creates an iterator over the child's events.
    pub fn events(&mut self) -> Result<CrocEventStream> {
        CrocEventStream::new(self)
    }

    /// Tries to kill the running Croc instance.
    pub async fn kill(&mut self) -> io::Result<()> {
        self.inner.kill().await
    }

    /// Gets the running child ID.
    pub fn id(&self) -> Option<u32> {
        self.inner.id()
    }
}
