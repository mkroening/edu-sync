//! A client for web service requests.

use std::result;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;
use tracing::{debug, error};
use url::Url;

use crate::{
    response::{content::Section, course::Course, info::Info},
    serde::NumBool,
    token::Token,
};

#[derive(Error, Deserialize, Debug, PartialEq)]
#[serde(tag = "errorcode")]
pub enum Error {
    #[error("access exception: {message}")]
    #[serde(rename = "accessexception")]
    AccessException { message: String },
    #[error("course context not valid: {message}")]
    #[serde(rename = "errorcoursecontextnotvalid")]
    CourseContextNotValid { message: String },
    #[error("invalid token: {message}")]
    #[serde(rename = "invalidtoken")]
    InvalidToken { message: String },
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    WsError(#[from] Error),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    Decode(#[from] serde_path_to_error::Error<serde_json::Error>),
}

impl RequestError {
    pub fn is_http(&self) -> bool {
        matches!(self, Self::HttpError(_))
    }
}

pub type Result<T> = result::Result<T, RequestError>;

#[derive(Debug)]
pub struct Client {
    http_client: reqwest::Client,
    ws_url: Url,
    token: Token,
    lang: Option<String>,
}

impl Client {
    #[must_use]
    pub fn new(
        http_client: reqwest::Client,
        site_url: &Url,
        token: Token,
        lang: Option<String>,
    ) -> Self {
        let ws_url = site_url.join("webservice/rest/server.php").unwrap();
        Self {
            http_client,
            ws_url,
            token,
            lang,
        }
    }

    async fn call_web_service<T, P>(&self, function: &str, params: Option<&P>) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize + Sync + ?Sized,
    {
        #[derive(Serialize)]
        struct WsQuery<'a> {
            #[serde(rename = "wstoken")]
            token: &'a Token,
            #[serde(rename = "wsfunction")]
            function: &'a str,
            #[serde(rename = "moodlewsrestformat")]
            rest_format: &'a str,
        }

        #[derive(Serialize)]
        struct Params<'a, P: ?Sized> {
            #[serde(flatten)]
            params: Option<&'a P>,

            /// Filter text
            ///
            /// When deactivated, localization fails and versions in multiple
            /// languages are being concatenated.
            #[serde(rename = "moodlewssettingfilter")]
            filter: bool,

            /// Force a session language
            #[serde(rename = "moodlewssettinglang")]
            lang: Option<&'a str>,
        }

        let response = self
            .http_client
            .post(self.ws_url.clone())
            .query(&WsQuery {
                token: &self.token,
                function,
                rest_format: "json",
            })
            .form(&Params {
                filter: true,
                params,
                lang: self.lang.as_deref(),
            })
            .send()
            .await?
            .text()
            .await
            .unwrap();
        debug!(response);

        let de = &mut serde_json::Deserializer::from_str(&response);
        let ok_err = match serde_path_to_error::deserialize(de) {
            Ok(value) => return Ok(value),
            Err(err) => err,
        };

        let de = &mut serde_json::Deserializer::from_str(&response);
        match serde_path_to_error::deserialize(de) {
            Ok(value) => Err(RequestError::WsError(value)),
            Err(err) => {
                error!(%ok_err, "Could not deserialize response");
                error!(%err, "Could not deserialize error");
                Err(RequestError::Decode(ok_err))
            }
        }
    }

    pub async fn get_info(&self) -> Result<Info> {
        self.call_web_service::<_, ()>("core_webservice_get_site_info", None)
            .await
    }

    pub async fn get_courses(&self, user_id: u64, return_user_count: bool) -> Result<Vec<Course>> {
        #[serde_as]
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "userid")]
            user_id: u64,
            #[serde_as(as = "NumBool")]
            #[serde(rename = "returnusercount")]
            return_user_count: bool,
        }

        self.call_web_service(
            "core_enrol_get_users_courses",
            Some(&Params {
                user_id,
                return_user_count,
            }),
        )
        .await
    }

    pub async fn get_contents(&self, course_id: u64) -> Result<Vec<Section>> {
        #[serde_as]
        #[derive(Serialize)]
        struct Params<'a> {
            #[serde(rename = "courseid")]
            course_id: u64,
            #[serde(rename = "options[0][name]")]
            include_stealth_modules_name: &'a str,
            #[serde_as(as = "NumBool")]
            #[serde(rename = "options[0][value]")]
            include_stealth_modules_value: bool,
        }

        self.call_web_service(
            "core_course_get_contents",
            Some(&Params {
                course_id,
                include_stealth_modules_name: "includestealthmodules",
                include_stealth_modules_value: true,
            }),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_ws_result_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Error::InvalidToken {
                message: "message".to_string()
            },
            serde_json::from_value(json!({
                "errorcode": "invalidtoken",
                "exception": "moodle_exception",
                "message": "message"
            }))?
        );
        Ok(())
    }
}
