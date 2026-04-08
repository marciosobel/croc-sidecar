/// An enum representing different `croc` code generation strategies.
#[derive(Debug, Default)]
pub enum Code {
    /// The code will be generated automatically by `croc`.
    #[default]
    Generated,
    /// Custom code that `croc` will use.
    Custom(String),
}

impl<S: ToString> From<S> for Code {
    fn from(value: S) -> Self {
        Self::Custom(value.to_string())
    }
}
