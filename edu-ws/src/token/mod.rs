//! Tokens and SSO.

pub mod login;
pub mod sso;

use std::{str, string::ToString};

use edu_ws_derive::{DerefWrapper, FromWrapper, HexWrapper};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(
    HexWrapper, DerefWrapper, FromWrapper, Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq,
)]
pub struct Token(#[serde(with = "hex")] pub [u8; 16]);

impl Token {
    pub fn apply(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("token", &self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            "6191f7ea9da0a4aed1cc9ddb23bf4aa7".parse::<Token>().unwrap(),
            serde_json::from_value(serde_json::json!("6191f7ea9da0a4aed1cc9ddb23bf4aa7"))?
        );
        assert_eq!(
            "\"6191f7ea9da0a4aed1cc9ddb23bf4aa7\"".to_string(),
            serde_json::to_string(&"6191f7ea9da0a4aed1cc9ddb23bf4aa7".parse::<Token>().unwrap())?
        );
        Ok(())
    }
}
