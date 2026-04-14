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
