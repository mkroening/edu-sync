//! Response from `core_webservice_get_site_info`.

use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::{serde_as, NoneAsEmptyString};
use url::Url;

use crate::serde::NumBool;

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Info {
    #[serde(rename = "sitename")]
    pub site_name: String,
    pub username: String,
    #[serde(rename = "firstname")]
    pub first_name: String,
    #[serde(rename = "lastname")]
    pub last_name: String,
    #[serde(rename = "fullname")]
    pub full_name: String,
    #[serde(rename = "lang")]
    pub language: String,
    #[serde(rename = "userid")]
    pub user_id: u64,
    #[serde(rename = "siteurl")]
    pub site_url: Url,
    #[serde(rename = "userpictureurl")]
    pub user_picture_url: Url,
    pub functions: Vec<Function>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "downloadfiles")]
    pub can_download_files: Option<bool>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "uploadfiles")]
    pub can_upload_files: Option<bool>,
    pub release: Option<String>,
    pub version: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "mobilecssurl")]
    pub mobile_css_url: Option<Url>,
    #[serde(rename = "advancedfeatures")]
    pub advanced_features: Vec<Feature>,
    #[serde(rename = "usercanmanageownfiles")]
    pub can_manage_own_files: Option<bool>,
    #[serde(rename = "userquota")]
    pub user_quota: Option<u64>,
    #[serde(rename = "usermaxuploadfilesize")]
    pub max_upload_file_size: Option<i64>,
    #[serde(rename = "userhomepage")]
    pub user_homepage: Option<Homepage>,
    #[serde(rename = "siteid")]
    pub site_id: Option<u64>,
    #[serde(rename = "sitecalendartype")]
    pub site_calendar_type: Option<String>,
    #[serde(rename = "usercalendartype")]
    pub user_calendar_type: Option<String>,
    pub theme: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub version: String,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Feature {
    pub name: String,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "value")]
    pub enabled: bool,
}

#[derive(Deserialize_repr, Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Homepage {
    SiteHome = 0,
    Dashboard = 1,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_info_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Info {
                site_name: "site_name".into(),
                username: "username".into(),
                first_name: "first_name".into(),
                last_name: "last_name".into(),
                full_name: "full_name".into(),
                language: "language".into(),
                user_id: 0,
                site_url: "https://example.com".parse().unwrap(),
                user_picture_url: "https://example.com/user_picture".parse().unwrap(),
                functions: Vec::new(),
                can_download_files: Some(true),
                can_upload_files: Some(true),
                release: Some("release".into()),
                version: Some("version".into()),
                mobile_css_url: Some("https://example.com/mobile_css".parse().unwrap()),
                advanced_features: Vec::new(),
                can_manage_own_files: Some(true),
                user_quota: Some(0),
                max_upload_file_size: Some(-1),
                user_homepage: Some(Homepage::Dashboard),
                site_id: Some(0),
                site_calendar_type: Some("gregorian".into()),
                user_calendar_type: Some("gregorian".into()),
                theme: Some("theme".into()),
            },
            serde_json::from_value(json!({
                "sitename": "site_name",
                "username": "username",
                "firstname": "first_name",
                "lastname": "last_name",
                "fullname": "full_name",
                "lang": "language",
                "userid": 0,
                "siteurl": "https://example.com",
                "userpictureurl": "https://example.com/user_picture",
                "functions": [],
                "downloadfiles": 1,
                "uploadfiles": 1,
                "release": "release",
                "version": "version",
                "mobilecssurl": "https://example.com/mobile_css",
                "advancedfeatures": [],
                "usercanmanageownfiles": true,
                "userquota": 0,
                "usermaxuploadfilesize": -1,
                "userhomepage": 1,
                "siteid": 0,
                "sitecalendartype": "gregorian",
                "usercalendartype": "gregorian",
                "theme": "theme",
            }))?,
        );
        assert_eq!(
            Info {
                site_name: "site_name".into(),
                username: "username".into(),
                first_name: "first_name".into(),
                last_name: "last_name".into(),
                full_name: "full_name".into(),
                language: "language".into(),
                user_id: 0,
                site_url: "https://example.com".parse().unwrap(),
                user_picture_url: "https://example.com/user_picture".parse().unwrap(),
                functions: Vec::new(),
                can_download_files: None,
                can_upload_files: None,
                release: None,
                version: None,
                mobile_css_url: None,
                advanced_features: Vec::new(),
                can_manage_own_files: None,
                user_quota: None,
                max_upload_file_size: None,
                user_homepage: None,
                site_id: None,
                site_calendar_type: None,
                user_calendar_type: None,
                theme: None,
            },
            serde_json::from_value(json!({
                "sitename": "site_name",
                "username": "username",
                "firstname": "first_name",
                "lastname": "last_name",
                "fullname": "full_name",
                "lang": "language",
                "userid": 0,
                "siteurl": "https://example.com",
                "userpictureurl": "https://example.com/user_picture",
                "functions": [],
                "mobilecssurl": "",
                "advancedfeatures": [],
            }))?,
        );
        Ok(())
    }

    #[test]
    fn test_function_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Function {
                name: "name".into(),
                version: "version".into()
            },
            serde_json::from_value(json!({"name": "name", "version": "version"}))?
        );
        Ok(())
    }

    #[test]
    fn test_feature_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Feature {
                name: "name".into(),
                enabled: true,
            },
            serde_json::from_value(json!({"name": "name", "value": 1}))?
        );
        Ok(())
    }
}
