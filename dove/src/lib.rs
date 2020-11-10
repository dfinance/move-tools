//! Move compiler.

// #![deny(missing_docs)]

#[macro_use]
extern crate anyhow;
extern crate log;

/// Movec commands handler.
pub mod cmd;
pub mod context;
/// Dove modules index.
pub mod index;
/// Movec configuration.
pub mod manifest;
// /// Move builder.
// pub mod builder;
