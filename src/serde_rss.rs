use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssReadingList {
    pub id: String,
    pub updated: i64,
    pub items: Vec<RssItem>,
    pub continuation: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssItem {
    pub id: String,
    pub crawl_time_msec: String,
    pub timestamp_usec: String,
    pub published: i64,
    pub title: String,
    pub canonical: Vec<Canonical>,
    pub alternate: Vec<Alternate>,
    pub categories: Vec<String>,
    pub origin: Origin,
    pub summary: Summary,
    #[serde(default)]
    pub enclosure: Vec<Enclosure>,
    pub author: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canonical {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alternate {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Origin {
    pub stream_id: String,
    pub html_url: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enclosure {
    pub href: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub length: Option<i64>,
}