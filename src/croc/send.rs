use tokio::process::Command;

use super::code::Code;
use crate::{CrocChild, Result};

/// A wrapper for the `croc` `send` command.
pub struct CrocSend<F>
where
    F: AsRef<str>,
{
    /// The inner `croc` command.
    inner: Command,
    /// The files to be sent.
    files: Vec<F>,
    /// Whether or not `croc` should zip the files before sending.
    zip: bool,
    /// The codephrase to be used.
    code: Code,
    /// The hash algorithm to use.
    hash: HashKind,
    /// Enable local relay when sending.
    local: bool,
    /// Enable multiplexing.
    multi: bool,
    /// Whether or not `croc` should respect `.gitignore`
    ignore_git: bool,
    /// Base port for the relay.
    port: u16,
    /// Number of ports to use for transfers.
    transfers: u8,
    /// Files to be excluded.
    excluded_files: Vec<F>,
}

/// A collection of hash algorithms to be used by `croc`.
#[derive(Debug, Default)]
pub enum HashKind {
    /// The `xxhash` algorithm.
    #[default]
    Xxhash,
    /// The `imohash` algorithm.
    Imohash,
    /// The `md5` algorithm.
    Md5,
}

impl<F> CrocSend<F>
where
    F: AsRef<str>,
{
    /// Creates a new `croc` `send` command.
    pub(crate) fn new(mut inner: Command) -> Self {
        inner.arg("send");

        Self {
            inner,
            zip: false,
            code: Code::default(),
            hash: HashKind::default(),
            local: true,
            multi: true,
            ignore_git: false,
            port: 9009,
            transfers: 4,
            files: Vec::new(),
            excluded_files: Vec::new(),
        }
    }

    /// Sets whether `croc` should zip the folder before sending. (Defaults to `false`)
    pub fn zip(mut self, zip: bool) -> Self {
        self.zip = zip;
        self
    }

    /// Sets a custom code to be used when sending files.
    pub fn code<S: ToString>(mut self, code: S) -> Self {
        self.code = code.into();
        self
    }

    /// Sets which hash algorithm to use to hash the files. (Defaults to [`HashKind::Xxhash`])
    pub fn hash(mut self, kind: HashKind) -> Self {
        self.hash = kind;
        self
    }

    /// Sets if the local relay should be used when sending. (Defaults to `true`)
    pub fn local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }

    /// Sets if multiplexing should be enabled or not. (Defaults to `true`)
    pub fn multiplexing(mut self, multiplexing: bool) -> Self {
        self.multi = multiplexing;
        self
    }

    /// Sets if `croc` should respect `.gitignore` / don't send ignored files. (Defaults to `false`)
    pub fn ignore_git(mut self, ignore_git: bool) -> Self {
        self.ignore_git = ignore_git;
        self
    }

    /// Sets the base port for the relay. (Defaults to `9009`)
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the number of ports to be used for transfers. (Defaults to `4`)
    pub fn transfers(mut self, transfers: u8) -> Self {
        self.transfers = transfers;
        self
    }

    /// Adds a new file to be excluded when sending.
    pub fn exclude(mut self, file: F) -> Self {
        self.excluded_files.push(file);
        self
    }

    /// Adds an iterator of files to be excluded when sending.
    pub fn exclude_many<I>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = F>,
    {
        self.excluded_files.extend(files);
        self
    }

    /// Adds a file to send.
    pub fn file(mut self, file: F) -> Self {
        self.files.push(file);
        self
    }

    /// Adds an iterator of files to send.
    pub fn files<I>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = F>,
    {
        self.files.extend(files);
        self
    }

    /// Resolve all send settings and spawns the `Croc` command.
    pub fn spawn(mut self) -> Result<CrocChild> {
        self.parse_options();

        for file in &self.files {
            self.inner.arg(file.as_ref());
        }

        self.inner.spawn().map(CrocChild::new).map_err(Into::into)
    }

    /// Use the settings in `self` to add their respective arguments to the `croc` command.
    fn parse_options(&mut self) {
        self.inner.arg(format!("--hash={}", self.hash));
        self.inner.arg(format!("--transfers={}", self.transfers));
        self.inner.arg(format!("--port={}", self.port));
        if self.zip {
            self.inner.arg("--zip");
        }
        if !self.local {
            self.inner.arg("--no-local");
        }
        if !self.multi {
            self.inner.arg("--no-multi");
        }
        if self.ignore_git {
            self.inner.arg("--git");
        }

        if let Code::Custom(ref code) = self.code {
            #[cfg(target_os = "windows")]
            self.inner.arg(format!("--code={}", code));

            #[cfg(not(target_os = "windows"))]
            self.inner.env("CROC_SECRET", code);
        }

        for file in &self.excluded_files {
            self.inner.arg(format!("--exclude={}", file.as_ref()));
        }
    }
}

impl std::fmt::Display for HashKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashKind::Xxhash => write!(f, "xxhash"),
            HashKind::Imohash => write!(f, "imohash"),
            HashKind::Md5 => write!(f, "md5"),
        }
    }
}

impl<F: AsRef<str>> std::fmt::Debug for CrocSend<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
