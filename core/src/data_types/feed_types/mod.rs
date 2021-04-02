pub mod atom;
pub mod rss;

use crate::data_types::{
    video::factory::VideoFactory,
};

pub(crate) struct Feed {
    pub(crate) name: String,
    pub(crate) videos: Vec<VideoFactory>,
}

impl From<rss::Feed> for Feed {
    fn from(rss_feed: rss::Feed) -> Self {
        let feed = rss_feed.channel;

        let name = feed.name;
        /* let id = feed.link; */

        let videos = feed
            .videos
            .into_iter()
            .map(|rss_vid| VideoFactory::from(rss_vid))
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
            .map(|atom_vid| VideoFactory::from(atom_vid))
            .collect();

        // Feed { name, id, videos }
        Feed { name, videos }
    }
}
