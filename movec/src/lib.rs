//! Move compiler.

#![deny(missing_docs)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

/// Movec commands handler.
pub mod cmd;
/// Movec configuration.
pub mod manifest;

