use serde::Deserialize;
use crate::internal;

// Deserialize structs
#[derive(Debug, Deserialize)]
pub struct Feed { // like channel in rss / internal
    #[serde(rename = "title")]
    pub name: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "entry")]
    pub videos: Vec<Video>,
}

#[derive(Debug, Deserialize)]
pub struct Video {
    pub title: String,
    #[serde(rename = "videoId")]
    pub id: String,
    #[serde(rename = "published")]
    pub pub_date: String,
}

#[allow(dead_code)]
impl Feed {
    pub fn to_internal_channel(self) -> internal::Channel {
        let name = self.name;
        let link = format!("https://www.youtube.com/channel/{}", self.channel_id);
        let videos = self.videos.into_iter().map(|v| v.to_internal_video()).collect();

        internal::Channel {
            name,
            link,
            videos,
            ..internal::Channel::new()
        }
    }
}

#[allow(dead_code)]
impl Video {
    fn to_internal_video(self) -> internal::Video {
        let title = self.title;
        let link = format!("https://www.youtube.com/watch?v={}", self.id);
        let pub_date = self.pub_date;

        internal::Video {
            title,
            link,
            pub_date,
            ..internal::Video::new()
        }
    }
}
