extern crate yaserde;
use serde::{Deserialize, Serialize};
use chrono::DateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feed {
    pub channel: Channel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub title: String,
    /* #[yaserde(prefix = "A")] */
    pub link: String,
    #[serde(rename = "item")]
    pub videos: Vec<Video>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Video {
    pub title: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub time: String,
}
impl Channel {
    pub fn get_videos(&self) -> Vec<Video> {
        let mut vids = Vec::new();
        for entry in self.videos.iter() {
            let d = DateTime::parse_from_rfc2822(&entry.time.clone()).unwrap();
            vids.push(
                Video {
                    title: entry.title.clone(),
                    link: entry.link.clone(),
                    time: d.to_rfc3339(),
                }
            );
        }
        vids
    }
}
