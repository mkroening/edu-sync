//! Moodle web service HTTP API wrapper.

#![warn(rust_2018_idioms)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod ajax;
pub mod response;
mod serde;
pub mod token;
pub mod ws;
