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

impl Into<ClipboardKind> for bool {
    fn into(self) -> ClipboardKind {
        match self {
            true => ClipboardKind::Enabled,
            false => ClipboardKind::Disabled,
        }
    }
}
