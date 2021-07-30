//! Response from `core_enrol_get_users_courses`.

use std::path::PathBuf;

use serde::Deserialize;
use serde_with::serde_as;
use time::{serde::timestamp, OffsetDateTime};
use url::Url;

use crate::{
    response::SummaryFormat,
    serde::{NumBool, StringAsHtml},
};

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Course {
    pub id: u64,
    #[serde_as(as = "StringAsHtml")]
    #[serde(rename = "shortname")]
    pub short_name: String,
    #[serde_as(as = "StringAsHtml")]
    #[serde(rename = "fullname")]
    pub full_name: String,
    #[serde_as(as = "Option<StringAsHtml>")]
    #[serde(default, rename = "displayname")]
    pub display_name: Option<String>,
    #[serde(rename = "enrolledusercount")]
    pub enrolled_user_count: Option<u64>,
    #[serde(rename = "idnumber")]
    pub id_number: String,
    #[serde_as(as = "NumBool")]
    pub visible: bool,
    pub summary: Option<String>,
    #[serde(rename = "summaryformat")]
    pub summary_format: Option<SummaryFormat>,
    pub format: Option<String>,
    #[serde(rename = "showgrades")]
    pub show_grades: Option<bool>,
    pub lang: Option<String>,
    #[serde(rename = "enablecompletion")]
    pub enable_completion: Option<bool>,
    #[serde(rename = "completionhascriteria")]
    pub completion_has_criteria: Option<bool>,
    #[serde(rename = "completionusertracked")]
    pub completion_user_tracked: Option<bool>,
    pub category: Option<u64>,
    pub progress: Option<f64>,
    pub completed: Option<bool>,
    #[serde(with = "timestamp::option", default, rename = "startdate")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "timestamp::option", default, rename = "enddate")]
    pub end_date: Option<OffsetDateTime>,
    pub marker: Option<u64>,
    #[serde(with = "timestamp::option", default, rename = "lastaccess")]
    pub last_access: Option<OffsetDateTime>,
    #[serde(rename = "isfavourite")]
    pub favourite: Option<bool>,
    pub hidden: Option<bool>,
    #[serde(rename = "overviewfiles")]
    pub overview_files: Option<Vec<OverviewFile>>,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct OverviewFile {
    #[serde(rename = "filename")]
    pub name: Option<String>,
    #[serde(rename = "filepath")]
    pub path: Option<PathBuf>,
    #[serde(rename = "filesize")]
    pub size: Option<u64>,
    /// Downloadable file url.
    #[serde(rename = "fileurl")]
    pub url: Option<Url>,
    #[serde(with = "timestamp::option", default, rename = "timemodified")]
    pub modified: Option<OffsetDateTime>,
    #[serde(default, rename = "mimetype")]
    pub media_type: Option<String>,
    #[serde(rename = "isexternalfile")]
    pub external: Option<bool>,
    /// The repository type for external files.
    #[serde(rename = "repositorytype")]
    pub repository_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::macros::date;

    use super::*;

    #[test]
    fn test_course_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Course {
                id: 1,
                short_name: "a > b && a < c".to_string(),
                full_name: "a > b && a < c".to_string(),
                display_name: Some("a > b && a < c".to_string()),
                enrolled_user_count: Some(100),
                id_number: "02ws-00001".to_string(),
                visible: true,
                summary: Some("summary".to_string()),
                summary_format: Some(SummaryFormat::Plain),
                format: Some("format".to_string()),
                show_grades: Some(true),
                lang: Some("en".to_string()),
                enable_completion: Some(true),
                completion_has_criteria: Some(false),
                completion_user_tracked: Some(true),
                category: Some(1),
                progress: Some(0.0),
                completed: Some(false),
                start_date: Some(date!(2002 - 08 - 20).midnight().assume_utc()),
                end_date: Some(date!(2003 - 02 - 20).midnight().assume_utc()),
                marker: Some(0),
                last_access: Some(date!(2002 - 11 - 20).midnight().assume_utc()),
                favourite: Some(false),
                hidden: Some(false),
                overview_files: Some(Vec::new()),
            },
            serde_json::from_value(json!({
                "id": 1,
                "shortname": "a &gt; b &amp;&amp; a &lt; c",
                "fullname": "a &gt; b &amp;&amp; a &lt; c",
                "displayname": "a &gt; b &amp;&amp; a &lt; c",
                "enrolledusercount": 100,
                "idnumber": "02ws-00001",
                "visible": 1,
                "summary": "summary",
                "summaryformat": 2,
                "format": "format",
                "showgrades": true,
                "lang": "en",
                "enablecompletion": true,
                "completionhascriteria": false,
                "completionusertracked": true,
                "category": 1,
                "progress": 0.0,
                "completed": false,
                "startdate": 1029801600,
                "enddate": 1045699200,
                "marker": 0,
                "lastaccess": 1037750400,
                "isfavourite": false,
                "hidden": false,
                "overviewfiles": []
            }))?
        );
        assert_eq!(
            Course {
                id: 1,
                short_name: "short_name".to_string(),
                full_name: "full_name".to_string(),
                display_name: None,
                enrolled_user_count: None,
                id_number: "02ws-00001".to_string(),
                visible: true,
                summary: None,
                summary_format: None,
                format: None,
                show_grades: None,
                lang: None,
                enable_completion: None,
                completion_has_criteria: None,
                completion_user_tracked: None,
                category: None,
                progress: None,
                completed: None,
                start_date: None,
                end_date: None,
                marker: None,
                last_access: None,
                favourite: None,
                hidden: None,
                overview_files: None,
            },
            serde_json::from_value(json!({
                "id": 1,
                "shortname": "short_name",
                "fullname": "full_name",
                "idnumber": "02ws-00001",
                "visible": 1
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_overview_file_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            OverviewFile {
                name: Some("logo.png".to_string()),
                path: Some(PathBuf::from("/")),
                size: Some(4096),
                url: Some("https://example.com/webservice/pluginfile.php/00001/course/overviewfiles/file.png".parse().unwrap()),
                modified: Some(date!(2002-08-20).midnight().assume_utc()),
                media_type: Some("image/png".to_string()),
                external: Some(true),
                repository_type: Some("repository_type".to_string())
            },
            serde_json::from_value(
                json!({
                    "filename": "logo.png",
                    "filepath": "/",
                    "filesize": 4096,
                    "fileurl": "https://example.com/webservice/pluginfile.php/00001/course/overviewfiles/file.png",
                    "timemodified": 1029801600,
                    "mimetype": "image/png",
                    "isexternalfile": true,
                    "repositorytype": "repository_type"
                })
            )?
        );
        assert_eq!(
            OverviewFile {
                name: None,
                path: None,
                size: None,
                url: None,
                modified: None,
                media_type: None,
                external: None,
                repository_type: None,
            },
            serde_json::from_value(json!({}))?
        );
        Ok(())
    }
}
