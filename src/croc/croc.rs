use std::{ffi::OsStr, net::IpAddr, process::Stdio};

use tokio::process::Command;

use crate::croc::receive::CrocReceive;

use super::send::CrocSend;

/// A builder around a [`Command`] that allows to set up a `croc` instance.
pub struct Croc {
    /// The inner `croc` command.
    inner: Command,
    /// Use a built-in DNS stub resolver rather than the host operating system
    internal_dns: bool,
    /// Whether to enable debug mode.
    debug: bool,
    /// Automatically agree to all prompts.
    yes: bool,
    /// Whether file compression should be enabled.
    compress: bool,
    /// Force to use only local connections.
    local: bool,
    /// Disables all output.
    quiet: bool,
    /// Whether the code should be copied to the clipboard by `croc`.
    clipboard: ClipboardKind,
    /// The multicast address to use for local discovery.
    multicast: Option<IpAddr>,
    /// The encryption curve to be used when encrypting files.
    curve: EncryptionCurve,
    /// The IP of the sender, if known.
    ip: Option<IpAddr>,
    /// Address of the relay.
    relay: Option<Relay>,
    /// The password of the relay.
    pass: Option<String>,
}

/// Settings about the `croc` instance copying the code to the system clipboard.
#[derive(Debug, Default)]
pub enum ClipboardKind {
    /// The code will not be copied automatically by `croc`.
    Disabled,
    /// The code will be copied automatically by `croc`.
    #[default]
    Enabled,
    /// The full `croc` command be copied automatically.
    Extended,
}

/// Different encryption curves to be used by `croc`.
#[derive(Debug, Default)]
pub enum EncryptionCurve {
    /// The `p521` encryption curve.
    P521,
    /// The `p256` encryption curve.
    #[default]
    P256,
    /// The `p384` encryption curve.
    P384,
    /// The `siec` encryption curve.
    Siec,
    /// The `ed25519` encryption curve.
    Ed25519,
}

/// A representation of a relay for `croc`.
#[derive(Debug)]
struct Relay {
    /// The IP address of the relay.
    address: IpAddr,
    /// The port of the relay.
    port: u16,
}

impl Croc {
    /// Creates a [`Croc`] builder.
    pub fn new() -> Self {
        Self::new_with_path("croc")
    }

    /// Creates a [`Croc`] builder with the provided executable name.
    pub fn new_with_path<P: AsRef<OsStr>>(name: P) -> Self {
        let inner = Command::new(name);

        let croc = Self {
            inner,
            internal_dns: false,
            debug: false,
            yes: true,
            compress: true,
            local: false,
            clipboard: ClipboardKind::default(),
            quiet: false,
            multicast: None,
            curve: EncryptionCurve::default(),
            ip: None,
            relay: None,
            pass: None,
        };

        croc.no_window().no_stdin().no_stdout().pipe_stderr()
    }

    /// Sets if `croc` should use a built-in DNS stub resolver rather than the host operating system. (Default: `false`)
    pub fn internal_dns(mut self, value: bool) -> Self {
        self.internal_dns = value;
        self
    }

    /// Sets the debug mode. (Default: `false`)
    pub fn debug(mut self, value: bool) -> Self {
        self.debug = value;
        self
    }

    /// Sets the `--yes` argument in `croc`, automatically agreeing to all prompts. (Default: `true`)
    pub fn yes(mut self, value: bool) -> Self {
        self.yes = value;
        self
    }

    /// Sets whether compression should be enabled or not. (Default: `true`)
    pub fn compress(mut self, value: bool) -> Self {
        self.compress = value;
        self
    }

    /// Forces `croc` to only use local connections. (Default: `false`)
    pub fn local(mut self, value: bool) -> Self {
        self.local = value;
        self
    }

    /// Sets whether output should be disabled. (Default: `false`)
    pub fn quiet(mut self, value: bool) -> Self {
        self.quiet = value;
        self
    }

    /// Sets the copy to clipboard behaviour for `croc`. (Default: [`ClipboardKind::Enabled`])
    pub fn clipboard<Kind: Into<ClipboardKind>>(mut self, kind: Kind) -> Self {
        self.clipboard = kind.into();
        self
    }

    /// Sets the multicast address for local discovery. (Default: `239.255.255.250`)
    pub fn multicast<Ip: Into<IpAddr>>(mut self, address: Ip) -> Self {
        self.multicast = Some(address.into());
        self
    }

    /// Sets the encryption curve to be used by `croc`. (Default: [`EncryptionCurve::P256`])
    pub fn curve(mut self, curve: EncryptionCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Specifies the IP address of the sender, if known. (Default: `None`)
    pub fn ip<Ip: Into<IpAddr>>(mut self, ip: Ip) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Specifies the IP address of the relay. (Default: `204.168.131.42:9009`)
    pub fn relay<Ip: Into<IpAddr>>(mut self, ip: Ip, port: u16) -> Self {
        self.relay = Some(Relay::new(ip, port));
        self
    }

    /// Specifies the password of the relay. (Default: `pass123`)
    pub fn pass<S: ToString>(mut self, pass: S) -> Self {
        self.pass = Some(pass.to_string());
        self
    }

    /// Disables croc stdout.
    pub fn no_stdout(mut self) -> Self {
        self.inner.stdout(Stdio::null());
        self
    }

    /// Disables croc stdin.
    pub fn no_stdin(mut self) -> Self {
        self.inner.stdin(Stdio::null());
        self
    }

    /// Disables croc stderr.
    pub fn no_stderr(mut self) -> Self {
        self.inner.stderr(Stdio::null());
        self
    }

    /// Pipes stderr for reading. All metadata croc gives us comes from stderr.
    pub fn pipe_stderr(mut self) -> Self {
        self.inner.stderr(Stdio::piped());
        self
    }

    /// Pipes stdout for reading. Croc can output received files to stdout.
    pub fn pipe_stdout(mut self) -> Self {
        self.inner.stdout(Stdio::piped());
        self
    }

    /// Pipes stdin for writing. Used for sending passphrases or other input.
    pub fn pipe_stdin(mut self) -> Self {
        self.inner.stdin(Stdio::piped());
        self
    }

    /// Disables the console window on Windows.
    /// Highly recommended for GUI applications.
    pub fn no_window(mut self) -> Self {
        self.inner.disable_window();
        self
    }

    /// Sets up croc to send files.
    pub fn send<F: AsRef<str>>(mut self) -> CrocSend<F> {
        self.parse_options();
        CrocSend::new(self.inner)
    }

    /// Sets up `croc` to receive files.
    pub fn receive(mut self) -> CrocReceive {
        self.parse_options();
        CrocReceive::new(self.inner)
    }

    /// Use the settings in `self` to add their respective arguments to the `croc` command.
    fn parse_options(&mut self) {
        if self.internal_dns {
            self.inner.arg("--internal-dns");
        }
        if self.debug {
            self.inner.arg("--debug");
        }
        if self.yes {
            self.inner.arg("--yes");
        }
        if !self.compress {
            self.inner.arg("--no-compress");
        }
        if self.local {
            self.inner.arg("--local");
        }
        if self.quiet {
            self.inner.arg("--quiet");
        }
        if let Some(address) = self.multicast {
            self.inner.arg(format!("--multicast={}", address));
        }
        if let Some(address) = self.ip {
            self.inner.arg(format!("--ip={}", address));
        }
        if let Some(ref relay) = self.relay {
            let key = match relay.address() {
                IpAddr::V4(_) => "CROC_RELAY",
                IpAddr::V6(_) => "CROC_RELAY6",
            };
            self.inner.env(key, relay.to_string());
        }
        if let Some(ref pass) = self.pass {
            self.inner.env("CROC_PASS", pass);
        }

        match self.clipboard {
            ClipboardKind::Disabled => {
                self.inner.arg("--disable-clipboard");
            }
            ClipboardKind::Extended => {
                self.inner.arg("--extended-clipboard");
            }
            ClipboardKind::Enabled => {}
        }

        match self.curve {
            EncryptionCurve::P256 => {}
            ref curve => {
                self.inner.arg(format!("--curve={}", curve));
            }
        }
    }
}

impl Relay {
    /// Creates a new `Relay`.
    pub fn new<Ip: Into<IpAddr>>(address: Ip, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
        }
    }

    /// The address of the `Relay`.
    pub fn address(&self) -> &IpAddr {
        &self.address
    }
}

impl Into<ClipboardKind> for bool {
    fn into(self) -> ClipboardKind {
        match self {
            true => ClipboardKind::Enabled,
            false => ClipboardKind::Disabled,
        }
    }
}

trait DisableWindow {
    /// Disables the console window on Windows.
    /// Highly recommended for GUI applications.
    fn disable_window(&mut self);
}

impl DisableWindow for Command {
    fn disable_window(&mut self) {
        #[cfg(target_os = "windows")]
        std::os::windows::process::CommandExt::creation_flags(self, 0x08000000);
    }
}

impl From<Croc> for Command {
    fn from(croc: Croc) -> Self {
        croc.inner
    }
}

impl Default for Croc {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Croc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::fmt::Display for EncryptionCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionCurve::P521 => write!(f, "p521"),
            EncryptionCurve::P256 => write!(f, "p256"),
            EncryptionCurve::P384 => write!(f, "p384"),
            EncryptionCurve::Siec => write!(f, "siec"),
            EncryptionCurve::Ed25519 => write!(f, "ed25519"),
        }
    }
}

impl std::fmt::Display for Relay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.address {
            IpAddr::V4(address) => write!(f, "{}:{}", address, self.port),
            IpAddr::V6(address) => write!(f, "[{}]:{}", address, self.port),
        }
    }
}
