//! SSO-based token creation.

use std::{str, string::ToString};

use edu_ws_derive::{DerefWrapper, FromWrapper, HexWrapper};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use super::Token;

#[derive(
    HexWrapper, DerefWrapper, FromWrapper, Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq,
)]
pub struct Signature(#[serde(with = "hex")] pub [u8; 16]);

impl Signature {
    fn from(site_url: &Url, passport: f64) -> Self {
        let mut context = md5::Context::new();
        context.consume(site_url.as_str().trim_end_matches('/'));
        context.consume(&passport.to_string());
        Self(context.compute().into())
    }
}

#[derive(PartialEq, Debug)]
pub struct SSOTokenBuilder {
    expected_signature: Signature,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid signature (expected {expected:?}, found {found:?})")]
    InvalidSignature {
        expected: Signature,
        found: Signature,
    },
    #[error("invalid token url")]
    InvalidTokenUrl,
}

impl SSOTokenBuilder {
    #[must_use]
    pub fn prepare_sso(site_url: &Url, launch_url: Url, url_scheme: &str) -> (Url, Self) {
        let passport = rand::random::<f64>() * 1000.0;
        let mut login_url = launch_url;
        login_url
            .query_pairs_mut()
            .append_pair("service", "moodle_mobile_app")
            .append_pair("passport", &passport.to_string())
            .append_pair("urlscheme", url_scheme);
        let expected_signature = Signature::from(site_url, passport);
        (login_url, Self { expected_signature })
    }

    fn parse_token_url(token_url: &Url) -> Result<(Signature, Token), Error> {
        let validation_token = token_url
            .domain()
            .ok_or(Error::InvalidTokenUrl)?
            .trim_start_matches("token=");
        // Validation String Format: <SIGNATURE_HEX16>:::<TOKEN_HEX16>
        const VALIDATION_LEN: usize = 32 + ":::".len() + 32;
        let mut validation_bytes = [0_u8; VALIDATION_LEN];
        base64::decode_config_slice(validation_token, base64::STANDARD, &mut validation_bytes)
            .or(Err(Error::InvalidTokenUrl))?;
        let validation_str = str::from_utf8(&validation_bytes).or(Err(Error::InvalidTokenUrl))?;
        let signature_hex = &validation_str[..32];
        let token_hex = &validation_str[32 + ":::".len()..];
        Ok((signature_hex.parse().unwrap(), token_hex.parse().unwrap()))
    }

    /// This required token is received in the form `edu-sync://token=TOKEN`
    pub fn validate(self, token_url: &Url) -> Result<Token, Error> {
        let (found_signature, token) = Self::parse_token_url(token_url)?;
        if self.expected_signature == found_signature {
            Ok(token)
        } else {
            Err(Error::InvalidSignature {
                expected: self.expected_signature,
                found: found_signature,
            })
        }
    }
}
