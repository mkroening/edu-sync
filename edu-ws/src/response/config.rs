//! Response from `tool_mobile_get_public_config`.

use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::{serde_as, NoneAsEmptyString};
use url::Url;

use crate::serde::NumBool;

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(rename = "wwwroot")]
    pub url: Url,
    #[serde(rename = "httpswwwroot")]
    pub https_url: Url,
    #[serde(rename = "sitename")]
    pub site_name: String,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "guestlogin")]
    pub guest_login: bool,
    #[serde(rename = "rememberusername")]
    pub remember_username: RememberUsername,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "authloginviaemail")]
    pub log_in_via_email: bool,
    #[serde(rename = "registerauth")]
    pub register_auth: String,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "forgottenpasswordurl")]
    pub forgotten_password_url: Option<Url>,
    #[serde(rename = "authinstructions")]
    pub auth_instructions: String,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "authnoneenabled")]
    pub auth_none: bool,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "enablewebservices")]
    pub web_services: bool,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "enablemobilewebservice")]
    pub mobile_service: bool,
    #[serde_as(as = "NumBool")]
    #[serde(rename = "maintenanceenabled")]
    pub maintenance: bool,
    #[serde(rename = "maintenancemessage")]
    pub maintenance_message: String,
    #[serde(rename = "logourl")]
    pub logo_url: Option<Url>,
    #[serde(rename = "compactlogourl")]
    pub compact_logo_url: Option<Url>,
    #[serde(rename = "typeoflogin")]
    pub login_type: LoginType,
    #[serde(rename = "launchurl")]
    pub launch_url: Option<Url>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "mobilecssurl")]
    pub mobile_css_url: Option<Url>,
    #[serde(rename = "tool_mobile_disabledfeatures")]
    pub disabled_mobile_features: Option<String>,
    #[serde(rename = "identityproviders")]
    pub identity_providers: Option<Vec<IdentityProvider>>,
    pub country: Option<String>,
    #[serde(rename = "agedigitalconsentverification")]
    pub age_digital_consent_verification: Option<bool>,
    #[serde(rename = "supportname")]
    pub support_name: Option<String>,
    #[serde(rename = "supportemail")]
    pub support_email: Option<String>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "autolang")]
    pub auto_lang: Option<bool>,
    pub lang: Option<String>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "langmenu")]
    pub lang_menu: Option<bool>,
    #[serde(rename = "langlist")]
    pub lang_list: Option<String>,
    pub locale: Option<String>,
    pub warnings: Option<Vec<Warning>>,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RememberUsername {
    No = 0,
    Yes = 1,
    Optional = 2,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum LoginType {
    App = 1,
    Browser = 2,
    Embedded = 3,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct IdentityProvider {
    name: String,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "iconurl")]
    icon_url: Option<Url>,
    url: Url,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Warning {
    item: Option<String>,
    #[serde(rename = "itemid")]
    item_id: Option<u64>,
    #[serde(rename = "warningcode")]
    warning_code: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_config_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Config {
                url: "http://example.com".parse().unwrap(),
                https_url: "https://example.com".parse().unwrap(),
                site_name: "site_name".to_string(),
                guest_login: true,
                remember_username: RememberUsername::Optional,
                log_in_via_email: false,
                register_auth: "register_auth".to_string(),
                forgotten_password_url: Some(
                    "https://example.com/forgotten_password".parse().unwrap()
                ),
                auth_instructions: "auth_instructions".to_string(),
                auth_none: false,
                web_services: true,
                mobile_service: true,
                maintenance: false,
                maintenance_message: "maintenance_message".to_string(),
                logo_url: Some("https://example.com/logo".parse().unwrap()),
                compact_logo_url: Some("https://example.com/compact_logo".parse().unwrap()),
                login_type: LoginType::Browser,
                launch_url: Some("https://example.com/launch".parse().unwrap()),
                mobile_css_url: Some("https://example.com/mobile_css".parse().unwrap()),
                disabled_mobile_features: Some("disabled_mobile_features".to_string()),
                identity_providers: Some(Vec::new()),
                country: Some("country".to_string()),
                age_digital_consent_verification: Some(false),
                support_name: Some("support_name".to_string()),
                support_email: Some("support@example.com".to_string()),
                auto_lang: Some(true),
                lang: Some("en".to_string()),
                lang_menu: Some(true),
                lang_list: Some("en, de".to_string()),
                locale: Some("en_US.UTF-8".to_string()),
                warnings: Some(Vec::new()),
            },
            serde_json::from_value(json!({
                "wwwroot": "http://example.com",
                "httpswwwroot": "https://example.com",
                "sitename": "site_name",
                "guestlogin": 1,
                "rememberusername": 2,
                "authloginviaemail": 0,
                "registerauth": "register_auth",
                "forgottenpasswordurl": "https://example.com/forgotten_password",
                "authinstructions": "auth_instructions",
                "authnoneenabled": 0,
                "enablewebservices": 1,
                "enablemobilewebservice": 1,
                "maintenanceenabled": 0,
                "maintenancemessage": "maintenance_message",
                "logourl": "https://example.com/logo",
                "compactlogourl": "https://example.com/compact_logo",
                "typeoflogin": 2,
                "launchurl": "https://example.com/launch",
                "mobilecssurl": "https://example.com/mobile_css",
                "tool_mobile_disabledfeatures": "disabled_mobile_features",
                "identityproviders": [],
                "country": "country",
                "agedigitalconsentverification": false,
                "supportname": "support_name",
                "supportemail": "support@example.com",
                "autolang": 1,
                "lang": "en",
                "langmenu": 1,
                "langlist": "en, de",
                "locale": "en_US.UTF-8",
                "warnings": []
            }))?
        );
        assert_eq!(
            Config {
                url: "http://example.com".parse().unwrap(),
                https_url: "https://example.com".parse().unwrap(),
                site_name: "site_name".to_string(),
                guest_login: true,
                remember_username: RememberUsername::Optional,
                log_in_via_email: false,
                register_auth: "register_auth".to_string(),
                forgotten_password_url: None,
                auth_instructions: "auth_instructions".to_string(),
                auth_none: false,
                web_services: true,
                mobile_service: true,
                maintenance: false,
                maintenance_message: "maintenance_message".to_string(),
                logo_url: None,
                compact_logo_url: None,
                login_type: LoginType::Browser,
                launch_url: None,
                mobile_css_url: None,
                disabled_mobile_features: None,
                identity_providers: None,
                country: None,
                age_digital_consent_verification: None,
                support_name: None,
                support_email: None,
                auto_lang: None,
                lang: None,
                lang_menu: None,
                lang_list: None,
                locale: None,
                warnings: None,
            },
            serde_json::from_value(json!({
                "wwwroot": "http://example.com",
                "httpswwwroot": "https://example.com",
                "sitename": "site_name",
                "guestlogin": 1,
                "rememberusername": 2,
                "authloginviaemail": 0,
                "registerauth": "register_auth",
                "forgottenpasswordurl": "",
                "authinstructions": "auth_instructions",
                "authnoneenabled": 0,
                "enablewebservices": 1,
                "enablemobilewebservice": 1,
                "maintenanceenabled": 0,
                "maintenancemessage": "maintenance_message",
                "typeoflogin": 2
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_identity_provider_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            IdentityProvider {
                name: "name".to_string(),
                icon_url: Some("https://example.com/icon".parse().unwrap()),
                url: "https://example.com".parse().unwrap(),
            },
            serde_json::from_value(json!({
                "name": "name",
                "iconurl": "https://example.com/icon",
                "url": "https://example.com"
            }))?
        );
        assert_eq!(
            IdentityProvider {
                name: "name".to_string(),
                icon_url: None,
                url: "https://example.com".parse().unwrap(),
            },
            serde_json::from_value(json!({
                "name": "name",
                "iconurl": "",
                "url": "https://example.com"
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_warning_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Warning {
                item: Some("item".to_string()),
                item_id: Some(1),
                warning_code: "warning_code".to_string(),
                message: "message".to_string(),
            },
            serde_json::from_value(json!({
                "item": "item",
                "itemid": 1,
                "warningcode": "warning_code",
                "message": "message"
            }))?
        );
        assert_eq!(
            Warning {
                item: None,
                item_id: None,
                warning_code: "warning_code".to_string(),
                message: "message".to_string(),
            },
            serde_json::from_value(json!({
                "warningcode": "warning_code",
                "message": "message"
            }))?
        );
        Ok(())
    }
}
