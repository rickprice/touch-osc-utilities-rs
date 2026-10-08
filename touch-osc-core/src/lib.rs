//! Lossless reading, writing, and building of TouchOSC (`.tosc`) layouts.
//!
//! [`tree`] is the generic, lossless tree model: it is the source of
//! truth, and is designed so that anything this crate doesn't explicitly
//! understand (an unrecognized property type, a future message shape, ...)
//! is still stored and survives a load/save cycle untouched. Everything
//! else (typed builders, validation, the text/YAML representation used by
//! the `tosc` CLI) is built on top of it and must not discard data the
//! generic layer preserved.
//!
//! See `docs/FORMAT.md` for what's actually been observed in real
//! `.tosc` files and what remains an assumption.

pub mod container;
pub mod error;
pub mod idgen;
mod layout;
pub mod text;
pub mod tree;
pub mod xml;
pub mod yaml;

pub use error::{Error, Result};
pub use tree::*;
