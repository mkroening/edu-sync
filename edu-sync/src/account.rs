use std::{fmt, path::PathBuf};

pub use edu_ws::token::Token;
use edu_ws::{
    ajax,
    response::{course::Course, info::Info},
    token::{
        login,
        sso::{self, SSOTokenBuilder},
    },
    ws,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    content::Content,
    util::{self, sanitize_path_component},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Id {
    pub site_url: Url,
    pub user_id: u64,
    pub lang: Option<String>,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.user_id, self.site_url.host_str().unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    id: Id,
    token: Token,
}

impl Account {
    pub async fn login(
        site_url: &Url,
        username: &str,
        password: &str,
    ) -> login::Result<login::Response> {
        login::Client::new(util::shared_http(), site_url)
            .login(username, password)
            .await
    }

    #[must_use]
    pub const fn token(&self) -> Token {
        self.token
    }

    #[must_use]
    pub const fn new(id: Id, token: Token) -> Self {
        Self { id, token }
    }

    fn ws_client(&self) -> ws::Client {
        ws::Client::new(
            util::shared_http(),
            &self.id.site_url,
            self.token,
            self.id.lang.clone(),
        )
    }

    pub async fn get_courses(&self) -> ws::Result<Vec<Course>> {
        let ws_client = self.ws_client();
        ws_client.get_courses(self.id.user_id, false).await
    }

    pub async fn get_contents(
        &self,
        course_id: u64,
        course_path: PathBuf,
    ) -> impl Iterator<Item = Content> {
        self.ws_client()
            .get_contents(course_id)
            .await
            .unwrap()
            .into_iter()
            .flat_map(move |section| {
                let section_name = section.name;
                let course_path = course_path.clone();
                section.modules.into_iter().map(move |module| {
                    (
                        module,
                        course_path.join(sanitize_path_component(&section_name).as_ref()),
                    )
                })
            })
            .filter_map(|(module, section_name)| {
                let module_name = module.name;
                module.contents.map(move |contents| {
                    (
                        section_name.join(sanitize_path_component(&module_name).as_ref()),
                        contents,
                    )
                })
            })
            .flat_map(|(dir, contents)| {
                contents
                    .into_iter()
                    .map(move |content| Content::new(content, dir.clone()))
            })
    }
}

pub struct Builder {
    site_url: Url,
    lang: Option<String>,
    token_builder: SSOTokenBuilder,
}

impl Builder {
    pub async fn new(site_url: Url, url_scheme: &str, lang: Option<String>) -> (Url, Self) {
        let ajax_client = ajax::Client::new(util::shared_http(), &site_url);
        let site_config = ajax_client.get_config().await.unwrap();
        let (sso_url, token_builder) =
            SSOTokenBuilder::prepare_sso(&site_url, site_config.launch_url.unwrap(), url_scheme);
        (
            sso_url,
            Self {
                site_url,
                lang,
                token_builder,
            },
        )
    }

    pub async fn validate(self, token_url: &Url) -> Result<Account, sso::Error> {
        let token = self.token_builder.validate(token_url)?;
        let ws_client = ws::Client::new(
            util::shared_http(),
            &self.site_url,
            token,
            self.lang.clone(),
        );
        let Info {
            site_url, user_id, ..
        } = ws_client.get_info().await.unwrap();
        let id = Id {
            site_url,
            user_id,
            lang: self.lang,
        };
        Ok(Account::new(id, token))
    }
}
