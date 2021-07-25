//! Moodle synchronization library.

#![warn(rust_2018_idioms)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![deny(rustdoc::all)]

pub mod account;
pub mod config;
pub mod content;
pub(crate) mod util;
