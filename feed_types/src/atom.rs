use serde::Deserialize;
/* use crate::internal; */

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

/* #[allow(dead_code)]
 * impl Feed {
 *     pub fn to_internal_channel(self, original_link: &String) -> internal::Channel {
 *         let name = self.name;
 *         let link = format!("https://www.youtube.com/channel/{}", self.channel_id);
 *         let mut videos: Vec<internal::Video> = self.videos.into_iter().map(|v| v.to_internal_video(original_link)).collect();
 *
 *         let mut channel = internal::Channel::new();
 *         channel.name = name;
 *         channel.id = link;
 *         // channel.videos = videos;
 *         channel.append(&mut videos);
 *
 *         channel
 *     }
 * } */

/* #[allow(dead_code)]
 * impl Video {
 *     fn to_internal_video(self, channel_link: &String) -> internal::Video {
 *         let title = self.title;
 *         let link = format!("https://www.youtube.com/watch?v={}", self.id);
 *         let pub_date = self.pub_date;
 *
 *         internal::Video {
 *             title,
 *             link,
 *             pub_date,
 *             channel_link: channel_link.clone(),
 *             ..internal::Video::new()
 *         }
 *     }
 * } */
