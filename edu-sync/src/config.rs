use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::Infallible,
    fmt::{self, Display},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use edu_ws::{
    response::{course::Course, info::Info},
    token::Token,
    ws,
};
use log::warn;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, serde_conv, DisplayFromStr};
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::{account::Id, util};

#[derive(Error, Debug)]
pub enum TomlReadError {
    #[error("I/O error")]
    IoError(#[from] io::Error),
    #[error("TOML deserialization error")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CourseConfig {
    pub name: String,
    pub sync: bool,
}

impl CourseConfig {
    #[must_use]
    pub fn name_as_path_component(&self) -> Cow<'_, str> {
        util::sanitize_path_component(&self.name)
    }
}

impl From<Course> for CourseConfig {
    fn from(course: Course) -> Self {
        Self {
            name: format!("{} {}", course.id, course.full_name),
            sync: false,
        }
    }
}

impl CourseConfig {
    fn apply(&mut self, other: &Self) {
        self.sync = other.sync;
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct SortedCourseConfigs(
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")] Vec<(u64, CourseConfig)>,
);

serde_conv!(
    CourseConfigsSorter,
    BTreeMap<u64, CourseConfig>,
    |source: &BTreeMap<u64, CourseConfig>| {
        let mut course_configs = source
            .iter()
            .map(|(&k, v)| (k, v.clone()))
            .collect::<Vec<_>>();
        course_configs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b).reverse());
        SortedCourseConfigs(course_configs)
    },
    |value: SortedCourseConfigs| -> Result<_, Infallible> {
        Ok(value.0.into_iter().collect())
    }
);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CourseConfigs(#[serde_as(as = "CourseConfigsSorter")] pub BTreeMap<u64, CourseConfig>);

impl CourseConfigs {
    pub fn update(&mut self, courses: Vec<Course>) {
        let mut new_configs = courses
            .into_iter()
            .map(|course| (course.id, CourseConfig::from(course)))
            .collect::<BTreeMap<_, _>>();
        for (id, config) in &self.0 {
            if let Some(new_config) = new_configs.get_mut(id) {
                new_config.apply(config);
            } else {
                warn!("Course \"{}\" ({}) is unavailable", config.name, id);
            }
        }
        self.0 = new_configs;
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AccountConfig {
    pub user: String,
    pub site: String,
    #[serde(flatten)]
    pub id: Id,
    pub token: Token,
    pub path: PathBuf,
    #[serde(default)]
    pub courses: CourseConfigs,
}

impl Display for AccountConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) at {}", self.user, self.id, self.path.display())
    }
}

impl AccountConfig {
    pub async fn new(
        site_url: Url,
        token: Token,
        path: PathBuf,
        lang: Option<String>,
    ) -> Result<Self, ws::Error> {
        let ws_client = ws::Client::new(util::shared_http(), &site_url, token, lang.clone());
        let Info {
            site_url,
            user_id,
            full_name,
            site_name,
            ..
        } = ws_client.get_info().await.map_err(|err| match err {
            ws::RequestError::WsError(err) => err,
            ws::RequestError::HttpError(err) => panic!("{err:?}"),
        })?;
        let id = Id {
            site_url,
            user_id,
            lang,
        };
        Ok(Self {
            user: full_name,
            site: site_name,
            id,
            token,
            path,
            courses: CourseConfigs(BTreeMap::new()),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub parallel_downloads: usize,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountConfig>,
}

impl Config {
    #[must_use]
    pub fn path() -> &'static Path {
        static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

        CONFIG_PATH
            .get_or_init(|| {
                let project_dirs = util::project_dirs();
                let mut config_path = project_dirs.config_dir().join(project_dirs.project_path());
                config_path.set_extension("toml");
                config_path
            })
            .as_path()
    }

    pub fn has_accounts(&self) -> bool {
        !self.accounts.is_empty()
    }

    pub fn has_courses(&self) -> bool {
        self.accounts
            .values()
            .any(|account_config| !account_config.courses.0.is_empty())
    }

    pub fn has_active_courses(&self) -> bool {
        self.accounts
            .values()
            .flat_map(|account_config| account_config.courses.0.values())
            .any(|course_config| course_config.sync)
    }

    pub async fn read() -> Result<Self, TomlReadError> {
        let string_result = fs::read_to_string(Self::path()).await;
        if matches!(&string_result, Err(err) if err.kind() == ErrorKind::NotFound) {
            return Ok(Self::default());
        }
        toml::from_str(&string_result?).map_err(Into::into)
    }

    pub async fn write(&self) -> io::Result<()> {
        fs::create_dir_all(util::project_dirs().config_dir()).await?;
        let mut config_file = File::create(Self::path()).await?;
        let toml = toml::to_string_pretty(self).unwrap();
        config_file.write_all(toml.as_bytes()).await?;
        config_file.flush().await?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parallel_downloads: 5,
            accounts: BTreeMap::default(),
        }
    }
}
