//! A wrapper for the [Croc] binary.
//! Set up the binary using builders and listen to events through streams.
//!
//! [Croc]: https://github.com/schollz/croc
//!
//! # Usage example
//!
//! ```no_run,standalone_crate
//! # use croc_sidecar::Croc;
//! fn main() -> croc_sidecar::Result {
//!     let croc = Croc::new()
//!         .send()
//!         .code("my-custom-code")
//!         .file("/path/to/file.rs")
//!         .spawn()?;
//! #   Ok(())
//! }
//! ```
//!
//! You can also listen to `events`:
//!
//! ```compile_fail,standalone_crate
//! # use futures_util::StreamExt;
//! # use croc_sidecar::Croc;
//! # #[tokio::main]
//! # async fn main() -> croc_sidecar::Result {
//!     let mut croc = Croc::new().send().file("file.rs").spawn()?;
//!
//!     let mut stream = croc.events()?;
//!     while let Some(event) = stream.next().await {
//!         // do something with `event`...
//!     }
//! #   Ok(())
//! # }
//! ```
pub mod croc;
pub use croc::Croc;

pub mod child;
pub(crate) use child::CrocChild;

pub mod event;
pub use event::CrocEvent;

pub mod error;
pub use error::{Error, Result};

pub mod parser;
pub(crate) use parser::CrocParser;

pub mod stream;
pub(crate) use stream::CrocEventStream;
