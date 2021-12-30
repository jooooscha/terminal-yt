use crate::backend::data::{feed::*, video::Video};
use chrono::DateTime;

#[derive(Default)]
pub struct VideoBuilder {
    video: Video,
}

impl VideoBuilder {
    pub fn create() -> Self {
        let mut video_factory = Self::default();
        video_factory.video.is_new = true;

        video_factory
    }

    pub fn set_title(&mut self, title: String) {
        self.video.title = title;
    }

    pub fn set_link(&mut self, link: String) {
        self.video.link = link;
    }

    pub fn set_origin_url(&mut self, url: &str) {
        self.video.origin_url = url.to_string();
    }

    pub fn set_origin_channel_name(&mut self, name: &str) {
        self.video.origin_channel_name = name.to_string();
    }

    pub fn set_pub_date(&mut self, date: String) {
        self.video.pub_date = date;
    }

    pub fn build(self) -> Video {
        self.video
    }
}

impl PartialEq<VideoBuilder> for VideoBuilder {
    fn eq(&self, other: &VideoBuilder) -> bool {
        self.video == other.video
    }
}

impl From<rss::Video> for VideoBuilder {
    fn from(rss_video: rss::Video) -> Self {
        let mut vf = VideoBuilder::create();

        let title = rss_video.title;
        let link = rss_video.link;
        let pub_date = match DateTime::parse_from_rfc2822(&rss_video.pub_date) {
            Ok(date) => date.to_rfc3339(),
            Err(e) => panic!("error parsing date in video {}: {}", title, e),
        };

        vf.set_title(title);
        vf.set_link(link);
        vf.set_pub_date(pub_date);

        vf
    }
}

impl From<atom::Video> for VideoBuilder {
    fn from(atom_vid: atom::Video) -> Self {
        let mut vf = VideoBuilder::create();

        let title = atom_vid.title;
        let link = format!("https://www.youtube.com/watch?v={}", atom_vid.id);
        let pub_date = atom_vid.pub_date;

        vf.set_title(title);
        vf.set_link(link);
        vf.set_pub_date(pub_date);

        vf
    }
}

/* #[cfg(test)]
 * pub mod tests {
 *     use super::*;
 *     use rand::{prelude::*, Rng};
 *
 *     impl VideoFactory {
 *         pub fn test() -> Self {
 *             let mut vf = VideoFactory::create();
 *             vf.set_title(String::new());
 *             vf.set_link(String::new());
 *             vf.set_origin_url(String::new());
 *             vf.set_origin_channel_name(String::new());
 *             vf.set_pub_date(String::new());
 *             vf.set_marked(false);
 *             vf.set_new(false);
 *
 *             vf
 *         }
 *     }
 *
 *     pub fn get_random_video_factory() -> VideoFactory {
 *         let mut rng = thread_rng();
 *         if rng.gen::<f64>() > 0.5 {
 *             get_unmarked_video_factory()
 *         } else {
 *             get_unmarked_video_factory()
 *         }
 *     }
 *
 *     pub fn get_marked_video_factory() -> VideoFactory {
 *         let mut vf = VideoFactory::test();
 *         vf.set_marked(true);
 *
 *         vf
 *     }
 *
 *     pub fn get_unmarked_video_factory() -> VideoFactory {
 *         let mut vf = VideoFactory::test();
 *
 *         vf.set_marked(false);
 *
 *         vf
 *     }
 * } */
