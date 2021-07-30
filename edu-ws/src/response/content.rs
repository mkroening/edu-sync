//! Response from `core_course_get_contents`.

use std::path::PathBuf;

use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use time::{serde::timestamp, OffsetDateTime};
use url::Url;

use crate::{
    response::SummaryFormat,
    serde::{NumBool, StringAsHtml},
};

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Section {
    pub id: u64,
    #[serde_as(as = "StringAsHtml")]
    pub name: String,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default)]
    pub visible: Option<bool>,
    pub summary: String,
    #[serde(rename = "summaryformat")]
    pub summary_format: SummaryFormat,
    pub section: Option<u64>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "hiddenbynumsections")]
    pub hidden_by_num_sections: Option<bool>,
    #[serde(rename = "uservisible")]
    pub user_visible: Option<bool>,
    #[serde(rename = "availabilityinfo")]
    pub availability_info: Option<String>,
    pub modules: Vec<Module>,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Module {
    pub id: u64,
    pub url: Option<Url>,
    #[serde_as(as = "StringAsHtml")]
    pub name: String,
    pub instance: Option<u64>,
    pub description: Option<String>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(rename = "uservisible")]
    pub user_visible: Option<bool>,
    #[serde(rename = "availabilityinfo")]
    pub availability_info: Option<String>,
    #[serde_as(as = "Option<NumBool>")]
    #[serde(default, rename = "visibleoncoursepage")]
    pub visible_on_course_page: Option<bool>,
    #[serde(rename = "modicon")]
    pub icon: Url,
    #[serde(rename = "modname")]
    pub ty: String,
    #[serde(rename = "modplural")]
    pub type_plural: String,
    pub availability: Option<String>,
    pub indent: u64,
    #[serde(rename = "onclick")]
    pub on_click: Option<String>,
    #[serde(rename = "afterlink")]
    pub after_link_info: Option<String>,
    #[serde(rename = "customdata")]
    pub custom_data: Option<String>,
    #[serde(rename = "completion")]
    pub completion_type: Option<CompletionType>,
    #[serde(rename = "completiondata")]
    pub completion_data: Option<CompletionData>,
    pub contents: Option<Vec<Content>>,
    #[serde(rename = "contentsinfo")]
    pub contents_info: Option<ContentsInfo>,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum CompletionType {
    None = 0,
    Manual = 1,
    Automatic = 2,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct CompletionData {
    pub state: CompletionState,
    #[serde(with = "timestamp", rename = "timecompleted")]
    pub time_completed: OffsetDateTime,
    #[serde(rename = "overrideby")]
    pub override_by: Option<u64>,
    #[serde(rename = "valueused")]
    pub value_used: Option<bool>,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum CompletionState {
    Incomplete = 0,
    Complete = 1,
    CompletePass = 2,
    CompleteFail = 3,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    pub ty: Type,
    #[serde_as(as = "StringAsHtml")]
    #[serde(rename = "filename")]
    pub name: String,
    #[serde(rename = "filepath")]
    pub path: Option<PathBuf>,
    #[serde(rename = "filesize")]
    pub size: u64,
    #[serde(rename = "fileurl")]
    pub url: Option<Url>,
    pub content: Option<String>,
    #[serde(with = "timestamp::option", default, rename = "timecreated")]
    pub created: Option<OffsetDateTime>,
    #[serde(with = "timestamp", rename = "timemodified")]
    pub modified: OffsetDateTime,
    pub sortorder: Option<u64>,
    #[serde(default, rename = "mimetype")]
    pub media_type: Option<String>,
    #[serde(rename = "isexternalfile")]
    pub external_file: Option<bool>,
    #[serde(rename = "repositorytype")]
    pub repository_type: Option<String>,
    #[serde(rename = "userid")]
    pub user_id: Option<u64>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Deserialize, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    File,
    Folder,
    Url,
    Content,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    #[serde(rename = "rawname")]
    pub raw_name: String,
    #[serde(rename = "isstandard")]
    pub standard: bool,
    #[serde(rename = "tagcollid")]
    pub collection_id: u64,
    #[serde(rename = "taginstanceid")]
    pub instance_id: u64,
    #[serde(rename = "taginstancecontextid")]
    pub instance_context_id: u64,
    #[serde(rename = "itemid")]
    pub item_id: u64,
    pub ordering: u64,
    pub flag: bool,
}

#[serde_as]
#[derive(Deserialize, PartialEq, Debug)]
pub struct ContentsInfo {
    #[serde(rename = "filescount")]
    pub count: u64,
    #[serde(rename = "filessize")]
    pub size: u64,
    #[serde(with = "timestamp", rename = "lastmodified")]
    pub modified: OffsetDateTime,
    #[serde(rename = "mimetypes")]
    pub media_types: Vec<String>,
    #[serde(rename = "repositorytype")]
    pub repository_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::macros::datetime;

    use super::*;

    #[test]
    fn test_section_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Section {
                id: 1,
                name: "a > b && a < c".to_string(),
                visible: Some(true),
                summary: "summary".to_string(),
                summary_format: SummaryFormat::Plain,
                section: Some(0),
                hidden_by_num_sections: Some(false),
                user_visible: Some(true),
                availability_info: Some("availability_info".to_string()),
                modules: Vec::new(),
            },
            serde_json::from_value(json!({
                "id": 1,
                "name": "a &gt; b &amp;&amp; a &lt; c",
                "visible": 1,
                "summary": "summary",
                "summaryformat": 2,
                "section": 0,
                "hiddenbynumsections": 0,
                "uservisible": true,
                "availabilityinfo": "availability_info",
                "modules": []
            }))?
        );
        assert_eq!(
            Section {
                id: 1,
                name: "section_name".to_string(),
                visible: None,
                summary: "summary".to_string(),
                summary_format: SummaryFormat::Plain,
                section: None,
                hidden_by_num_sections: None,
                user_visible: None,
                availability_info: None,
                modules: Vec::new(),
            },
            serde_json::from_value(json!({
                "id": 1,
                "name": "section_name",
                "summary": "summary",
                "summaryformat": 2,
                "modules": []
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_module_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Module {
                id: 1,
                url: Some(
                    "https://example.com/mod/folder/view.php?id=1"
                        .parse()
                        .unwrap()
                ),
                name: "a > b && a < c".to_string(),
                instance: Some(1),
                description: Some("module_description".to_string()),
                visible: Some(true),
                user_visible: Some(true),
                availability_info: Some("availability_info".to_string()),
                visible_on_course_page: Some(true),
                icon: "https://example.com/icon".parse().unwrap(),
                ty: "resource".to_string(),
                type_plural: "resources".to_string(),
                availability: Some("availability".to_string()),
                indent: 0,
                on_click: Some("on_click".to_string()),
                after_link_info: Some("after_link_info".to_string()),
                custom_data: Some(serde_json::to_string(&json!({ "an": "object" })).unwrap()),
                completion_type: Some(CompletionType::Manual),
                completion_data: None,
                contents: Some(Vec::new()),
                contents_info: None,
            },
            serde_json::from_value(json!({
                "id": 1,
                "url": "https://example.com/mod/folder/view.php?id=1",
                "name": "a &gt; b &amp;&amp; a &lt; c",
                "instance": 1,
                "description": "module_description",
                "visible": 1,
                "uservisible": true,
                "availabilityinfo": "availability_info",
                "visibleoncoursepage": 1,
                "modicon": "https://example.com/icon",
                "modname": "resource",
                "modplural": "resources",
                "availability": "availability",
                "indent": 0,
                "onclick": "on_click",
                "afterlink": "after_link_info",
                "customdata": "{\"an\":\"object\"}",
                "completion": 1,
                "completiondata": null,
                "contents": [],
                "contentsinfo": null
            }))?
        );
        assert_eq!(
            Module {
                id: 1,
                url: None,
                name: "module_name".to_string(),
                instance: None,
                description: None,
                visible: None,
                user_visible: None,
                availability_info: None,
                visible_on_course_page: None,
                icon: "https://example.com/icon".parse().unwrap(),
                ty: "resource".to_string(),
                type_plural: "resources".to_string(),
                availability: None,
                indent: 0,
                on_click: None,
                after_link_info: None,
                custom_data: None,
                completion_type: None,
                completion_data: None,
                contents: None,
                contents_info: None,
            },
            serde_json::from_value(json!({
                "id": 1,
                "name": "module_name",
                "modicon": "https://example.com/icon",
                "modname": "resource",
                "modplural": "resources",
                "indent": 0
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_completion_data_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            CompletionData {
                state: CompletionState::Incomplete,
                time_completed: datetime!(2002 - 08 - 20 0:00 UTC),
                override_by: Some(1),
                value_used: Some(false),
            },
            serde_json::from_value(json!({
                "state": 0,
                "timecompleted": 1029801600,
                "overrideby": 1,
                "valueused": false
            }))?
        );
        assert_eq!(
            CompletionData {
                state: CompletionState::Incomplete,
                time_completed: datetime!(2002 - 08 - 20 0:00 UTC),
                override_by: None,
                value_used: None,
            },
            serde_json::from_value(json!({
                "state": 0,
                "timecompleted": 1029801600
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_content_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Content {
                ty: Type::File,
                name: "a > b && a < c.pdf".to_string(),
                path: Some(PathBuf::from("/")),
                size: 4096,
                url: Some("https://example.com/".parse().unwrap()),
                content: Some("content".to_string()),
                created: Some(datetime!(2002 - 08 - 20 0:00 UTC)),
                modified: datetime!(2002 - 11 - 20 0:00 UTC),
                sortorder: Some(0),
                media_type: Some("application/pdf".to_string()),
                external_file: Some(false),
                repository_type: Some("repository_type".to_string()),
                user_id: Some(1),
                author: Some("author".to_string()),
                license: Some("license".to_string()),
                tags: Some(Vec::new()),
            },
            serde_json::from_value(json!({
                "type": "file",
                "filename": "a &gt; b &amp;&amp; a &lt; c.pdf",
                "filepath": "/",
                "filesize": 4096,
                "fileurl": "https://example.com/",
                "content": "content",
                "timecreated": 1029801600,
                "timemodified": 1037750400,
                "sortorder": 0,
                "mimetype": "application/pdf",
                "isexternalfile": false,
                "repositorytype": "repository_type",
                "userid": 1,
                "author": "author",
                "license": "license",
                "tags": []
            }))?
        );
        assert_eq!(
            Content {
                ty: Type::File,
                name: "file.pdf".to_string(),
                path: None,
                size: 4096,
                url: None,
                content: None,
                created: None,
                modified: datetime!(2002 - 11 - 20 0:00 UTC),
                sortorder: None,
                media_type: None,
                external_file: None,
                repository_type: None,
                user_id: None,
                author: None,
                license: None,
                tags: None,
            },
            serde_json::from_value(json!({
                "type": "file",
                "filename": "file.pdf",
                "filesize": 4096,
                "timemodified": 1_037_750_400
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_tag_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            Tag {
                id: 1,
                name: "name".to_string(),
                raw_name: "raw_name".to_string(),
                standard: false,
                collection_id: 1,
                instance_id: 1,
                instance_context_id: 1,
                item_id: 1,
                ordering: 1,
                flag: false,
            },
            serde_json::from_value(json!({
                "id": 1,
                "name": "name",
                "rawname": "raw_name",
                "isstandard": false,
                "tagcollid": 1,
                "taginstanceid": 1,
                "taginstancecontextid": 1,
                "itemid": 1,
                "ordering": 1,
                "flag": false
            }))?
        );
        Ok(())
    }

    #[test]
    fn test_contents_info_deserialization() -> serde_json::Result<()> {
        assert_eq!(
            ContentsInfo {
                count: 2,
                size: 8192,
                modified: datetime!(2002 - 11 - 20 0:00 UTC),
                media_types: vec!["application/pdf".to_string(), "text/plain".to_string(),],
                repository_type: Some("repository_type".to_string()),
            },
            serde_json::from_value(json!({
                "filescount": 2,
                "filessize": 8192,
                "lastmodified": 1_037_750_400,
                "mimetypes": ["application/pdf", "text/plain"],
                "repositorytype": "repository_type"
            }))?
        );
        assert_eq!(
            ContentsInfo {
                count: 2,
                size: 8192,
                modified: datetime!(2002 - 11 - 20 0:00 UTC),
                media_types: vec!["application/pdf".to_string(), "text/plain".to_string(),],
                repository_type: None,
            },
            serde_json::from_value(json!({
                "filescount": 2,
                "filessize": 8192,
                "lastmodified": 1_037_750_400,
                "mimetypes": ["application/pdf", "text/plain"]
            }))?
        );
        Ok(())
    }
}
