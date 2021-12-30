pub mod atom;
pub mod rss;
use quick_xml::de::from_str;

use crate::backend::data::video::builder::VideoBuilder;

#[derive(Default)]
pub(crate) struct Feed {
    pub(crate) name: String,
    pub(crate) videos: Vec<VideoBuilder>,
}

impl Feed {
    pub fn parse_text(feed: String) -> Result<Self, String> {

        // try to parse as atom
        if let Ok(feed) = from_str::<atom::Feed>(&feed) {
            return Ok(feed.into())
        }

        // try to parse as rss
        if let Ok(feed) = from_str::<rss::Feed>(&feed) {
            return Ok(feed.into())
        }

        Err(String::from("Could not parse feed"))
    }

    pub fn add_videos(&mut self, videos: Vec<VideoBuilder>) {
        for video in videos.into_iter() {
            if !self.videos.iter().any(|v| v == &video) {
                self.videos.push(video);
            }
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl From<rss::Feed> for Feed {
    fn from(rss_feed: rss::Feed) -> Self {
        let feed = rss_feed.channel;

        let name = feed.name;
        /* let id = feed.link; */

        let videos = feed
            .videos
            .into_iter()
            .map(VideoBuilder::from)
            .collect();

        // Feed { name, id, videos }
        Feed { name, videos }
    }
}

impl From<atom::Feed> for Feed {
    fn from(feed: atom::Feed) -> Self {
        let name = feed.name;
        /* let id = format!("https://www.youtube.com/channel/{}", feed.channel_id); */

        let videos = feed
            .videos
            .into_iter()
            .map(VideoBuilder::from)
            .collect();

        // Feed { name, id, videos }
        Feed { name, videos }
    }
}
