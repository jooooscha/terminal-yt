use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feed {
    pub channel: Channel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub title: String,
    #[serde(skip)]
    pub link: String,
    #[serde(rename = "item")]
    pub videos: Vec<Video>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Video {
    #[serde(skip)]
    pub title: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub time: String,
}
    


