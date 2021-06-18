//! Login-based token requesting.

use std::result;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{serde::UntaggedResultHelper, token::Token};

#[derive(Debug)]
pub struct Client {
    http_client: reqwest::Client,
    login_url: Url,
}

#[derive(Error, Deserialize, Debug, PartialEq)]
#[error("{error}")]
pub struct Error {
    error: String,
    #[serde(rename = "errorcode")]
    error_code: ErrorCode,
    #[serde(rename = "stacktrace")]
    stack_trace: String,
    #[serde(rename = "debuginfo")]
    debug_info: String,
    #[serde(rename = "reproductionlink")]
    reproduction_url: Url,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ErrorCode {
    #[serde(rename = "invalidlogin")]
    InvalidLogin,
    #[serde(rename = "enablewsdescription")]
    WsDisabled,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Response {
    pub token: Token,
    #[serde(default, rename = "privatetoken")]
    pub private_token: Option<String>,
}

impl Client {
    #[must_use]
    pub fn new(http_client: reqwest::Client, site_url: &Url) -> Self {
        let login_url = site_url.join("login/token.php").unwrap();
        Self {
            http_client,
            login_url,
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Response> {
        #[derive(Serialize)]
        struct LoginRequest<'a> {
            username: &'a str,
            password: &'a str,
            service: &'a str,
        }

        self.http_client
            .post(self.login_url.clone())
            .query(&LoginRequest {
                username,
                password,
                service: "moodle_mobile_app",
            })
            .send()
            .await
            .map_err(ReceiveError::HttpError)?
            .json::<UntaggedResultHelper<Response, Error>>()
            .await
            .map_err(ReceiveError::HttpError)?
            .0
            .map_err(ReceiveError::LoginRequestError)
    }
}

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error(transparent)]
    LoginRequestError(#[from] Error),
    #[error("{0}")]
    HttpError(reqwest::Error),
}

pub type Result<T> = result::Result<T, ReceiveError>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_login_response_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Response {
                token: "9859148a89546f0efe716a58e340849b".parse().unwrap(),
                private_token: Some(
                    "8RpHJevJ42W7QN23OMkeYcdOYw3YfWgWGKsak7WB3Z88wcApSCVZ9TgY6M5fEO1m".to_string()
                ),
            },
            serde_json::from_value(json!({
                "token": "9859148a89546f0efe716a58e340849b",
                "privatetoken": "8RpHJevJ42W7QN23OMkeYcdOYw3YfWgWGKsak7WB3Z88wcApSCVZ9TgY6M5fEO1m"
            }))?
        );
        assert_eq!(
            Response {
                token: "9859148a89546f0efe716a58e340849b".parse().unwrap(),
                private_token: None,
            },
            serde_json::from_value(json!({
                "token": "9859148a89546f0efe716a58e340849b",
                "privatetoken": null
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_login_error_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Error {
                error: "Invalid login, please try again".to_string(),
                error_code: ErrorCode::InvalidLogin,
                stack_trace: "* line 113 of /login/token.php: moodle_exception thrown\n"
                    .to_string(),
                debug_info: "\nError code: invalidlogin".to_string(),
                reproduction_url: "https://example.com/".parse().unwrap(),
            },
            serde_json::from_value(json!({
                "error": "Invalid login, please try again",
                "errorcode": "invalidlogin",
                "stacktrace": "* line 113 of /login/token.php: moodle_exception thrown\n",
                "debuginfo": "\nError code: invalidlogin",
                "reproductionlink": "https://example.com/"
            }))?
        );
        Ok(())
    }
}
