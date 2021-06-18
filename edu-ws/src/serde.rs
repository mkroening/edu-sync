//! Utilities for serde.

use std::{borrow::Cow, convert::Infallible};

use serde::{de::DeserializeOwned, Deserialize};
use serde_with::serde_conv;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged, remote = "Result")]
pub enum UntaggedResult<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Deserialize)]
pub struct UntaggedResultHelper<T: DeserializeOwned, E: DeserializeOwned>(
    #[serde(with = "UntaggedResult")] pub Result<T, E>,
);

serde_conv!(
    pub NumBool,
    bool,
    |source: &bool| *source as u8,
    |value| match value {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(format!("bool out of range: {}", value)),
    }
);

#[allow(clippy::ptr_arg)]
serde_with::serde_conv!(
    pub StringAsHtml,
    String,
    |string: &str| html_escape::encode_text(string).into_owned(),
    |html: String| -> Result<_, Infallible> {
        match html_escape::decode_html_entities(&html) {
            Cow::Owned(string) => Ok(string),
            Cow::Borrowed(_) => Ok(html),
        }
    }
);
