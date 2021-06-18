//! Responses to several web service requests.

pub mod config;
pub mod content;
pub mod course;
pub mod info;

use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SummaryFormat {
    Html = 1,
    Moodle = 0,
    Plain = 2,
    Markdown = 4,
}
