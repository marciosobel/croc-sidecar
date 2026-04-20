use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::{CrocChild, Result};

/// A wrapper for the `croc` `receive` command.
pub struct CrocReceive {
    inner: Command,
    /// Overwrite if the file already exists.
    overwrite: bool,
    /// The path which the file will be saved at.
    out: Option<PathBuf>,
}

impl CrocReceive {
    /// Creates a new `croc` `send` command.
    pub(crate) fn new(inner: Command) -> Self {
        Self {
            inner,
            overwrite: false,
            out: None,
        }
    }

    /// Removes the prompt to overwrite or resume if file is already present. (Default: `false`)
    pub fn overwrite(mut self, value: bool) -> Self {
        self.overwrite = value;
        self
    }

    /// Sets the output folder to receive the file.
    pub fn out<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref();
        if path.is_dir() {
            self.out = Some(path.to_owned());
        } else {
            self.out = path.parent().map(ToOwned::to_owned);
        }
        self
    }

    /// Resolve all send settings and spawns the `Croc` command.
    pub fn spawn<C: AsRef<str>>(mut self, code: C) -> Result<CrocChild> {
        self.parse_options();
        self.set_receive_code(code.as_ref());
        self.inner.spawn().map(CrocChild::new).map_err(Into::into)
    }

    /// Use the settings in `self` to add their respective arguments to the `croc` command.
    fn parse_options(&mut self) {
        if self.overwrite {
            self.inner.arg("--overwrite");
        }

        if let Some(ref path) = self.out {
            self.inner.arg(format!("--out={}", path.display()));
        }
    }

    /// Sets the receive code for `croc`.
    fn set_receive_code(&mut self, code: &str) {
        #[cfg(target_os = "windows")]
        self.inner.arg(code.trim());

        #[cfg(not(target_os = "windows"))]
        self.inner.env("CROC_SECRET", code.trim());
    }
}

impl std::fmt::Debug for CrocReceive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
