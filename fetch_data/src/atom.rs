use serde::{Deserialize, Serialize};
use crate::rss;
use chrono::DateTime;

// Deserialize structs
#[derive(Debug, Deserialize)]
pub struct Feed {
    #[serde(rename = "entry")]
    pub entries: Vec<Video>,
    pub title: String,
    #[serde(rename = "channelId")]
    pub link: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Video {
    #[serde(rename = "videoId")]
    pub id: String,
    pub title: String,
    #[serde(rename = "published")]
    pub time: String,
}

impl Feed {
    pub fn get_videos(&self) -> Vec<rss::Video> {
        let mut vids = Vec::new();
        for entry in self.entries.iter() {
            vids.push(
                rss::Video {
                    title: entry.title.clone(),
                    link: format!("https://www.youtube.com/watch?v={}", entry.id),
                    time: entry.time.clone(),
                }
            );
        }
        vids
    }
}
