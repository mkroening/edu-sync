//! A client for Axaj requests.

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{response::config::Config, serde::UntaggedResultHelper};

#[derive(Debug)]
pub struct Client {
    http_client: reqwest::Client,
    ajax_url: Url,
}

#[derive(Error, Deserialize, Debug, PartialEq)]
#[serde(tag = "errorcode")]
pub enum Exception {
    #[error("{message}")]
    #[serde(rename = "invalidparameter")]
    InvalidParameter {
        message: String,
        #[serde(rename = "link")]
        url: Url,
        #[serde(rename = "moreinfourl")]
        info_url: Url,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum AjaxResult<T> {
    Ok { data: T },
    Err { exception: Exception },
}

impl<T> From<AjaxResult<T>> for Result<T, Exception> {
    fn from(res: AjaxResult<T>) -> Self {
        match res {
            AjaxResult::Ok { data } => Ok(data),
            AjaxResult::Err { exception } => Err(exception),
        }
    }
}

#[derive(Serialize)]
struct Request<'a> {
    #[serde(rename = "methodname")]
    method: &'a str,
    #[serde(rename = "args")]
    arguments: HashMap<String, String>,
}

impl Client {
    #[must_use]
    pub fn new(http_client: reqwest::Client, site_url: &Url) -> Self {
        let ajax_url = site_url.join("lib/ajax/service-nologin.php").unwrap();
        Self {
            http_client,
            ajax_url,
        }
    }

    async fn call_ajax<T>(
        &self,
        requests: &[Request<'_>],
    ) -> Result<Vec<Result<T, Exception>>, ReceiveError>
    where
        T: DeserializeOwned,
    {
        let res = self
            .http_client
            .post(self.ajax_url.clone())
            .json(&requests)
            .send()
            .await?
            .json::<UntaggedResultHelper<Vec<AjaxResult<T>>, RequestError>>()
            .await?
            .0?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(res)
    }

    pub async fn get_config(&self) -> Result<Config, Error> {
        let config = self
            .call_ajax(&[Request {
                method: "tool_mobile_get_public_config",
                arguments: HashMap::new(),
            }])
            .await?
            .into_iter()
            .next()
            .unwrap()?;

        Ok(config)
    }
}

#[derive(Error, Deserialize, Debug, PartialEq)]
#[serde(tag = "errorcode")]
pub enum RequestError {
    #[error("{message}")]
    #[serde(rename = "invalidrecord")]
    InvalidRecord {
        #[serde(rename = "error")]
        message: String,
    },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReceiveError(#[from] ReceiveError),
    #[error(transparent)]
    Exception(#[from] Exception),
}

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_ajax_result_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            AjaxResult::Err::<()> {
                exception: Exception::InvalidParameter {
                    message: "message".to_string(),
                    url: "https://example.com".parse().unwrap(),
                    info_url: "https://example.com/info".parse().unwrap(),
                }
            },
            serde_json::from_value(json!({
                "error": true,
                "exception": {
                    "errorcode": "invalidparameter",
                    "link": "https://example.com",
                    "message": "message",
                    "moreinfourl": "https://example.com/info"
                }
            }))?
        );
        Ok(())
    }
}
