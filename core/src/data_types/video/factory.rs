use crate::data_types::{feed_types::*, video::video::Video};
use chrono::DateTime;

pub struct VideoFactory {
    video: Video,
    title_set: bool,
    link_set: bool,
    origin_url_set: bool,
    origin_channel_name_set: bool,
    pub_date_set: bool,
    marked_set: bool,
    new_set: bool,
    fav_set: bool,
}

impl VideoFactory {
    pub fn create() -> VideoFactory {
        let video = Video {
            title: String::new(),
            link: String::new(),
            origin_url: String::new(),
            origin_channel_name: String::new(),
            pub_date: String::new(),
            marked: false,
            new: true,
            fav: false,
        };

        VideoFactory {
            video,
            title_set: false,
            link_set: false,
            origin_url_set: false,
            origin_channel_name_set: false,
            pub_date_set: false,
            marked_set: false,
            new_set: false,
            fav_set: false,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.video.title = title;
        self.title_set = true;
    }

    pub fn set_link(&mut self, link: String) {
        self.video.link = link;
        self.link_set = true;
    }

    pub fn set_origin_url(&mut self, url: String) {
        self.video.origin_url = url;
        self.origin_url_set = true;
    }

    pub fn set_origin_channel_name(&mut self, name: String) {
        self.video.origin_channel_name = name;
        self.origin_channel_name_set = true;
    }

    pub fn set_pub_date(&mut self, date: String) {
        self.video.pub_date = date;
        self.pub_date_set = true;
    }

    pub fn set_marked(&mut self, marked: bool) {
        self.video.marked = marked;
        self.marked_set = true;
    }

    pub fn set_new(&mut self, is_new: bool) {
        self.video.new = is_new;
        self.new_set = true;
    }

    pub fn set_fav(&mut self, is_fav: bool) {
        self.video.fav = is_fav;
        self.fav_set = true;
    }

    pub fn commit(self) -> Result<Video, String> {
        if !self.title_set {
            return Err(String::from("Title not set"));
        }

        if !self.link_set {
            return Err(String::from("link not set"));
        }

        if !self.origin_url_set {
            return Err(String::from("origin_url not set"));
        }

        if !self.origin_channel_name_set {
            return Err(String::from("origin_channel_name not set"));
        }

        if !self.pub_date_set {
            return Err(String::from("pub_date not set"));
        }

        if !self.marked_set {
            return Err(String::from("marked not set"));
        }

        if !self.new_set {
            return Err(String::from("New not set"));
        }

        if !self.fav_set {
            return Err(String::from("Fav not set"));
        }

        Ok(self.video) // if all correct return video
    }
}

impl PartialEq<VideoFactory> for VideoFactory {
    fn eq(&self, other: &VideoFactory) -> bool {
        self.video == other.video
    }
}

impl From<rss::Video> for VideoFactory {
    fn from(rss_video: rss::Video) -> Self {
        let mut vf = VideoFactory::create();

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

impl From<atom::Video> for VideoFactory {
    fn from(atom_vid: atom::Video) -> Self {
        let mut vf = VideoFactory::create();

        let title = atom_vid.title;
        let link = format!("https://www.youtube.com/watch?v={}", atom_vid.id);
        let pub_date = atom_vid.pub_date;

        vf.set_title(title);
        vf.set_link(link);
        vf.set_pub_date(pub_date);

        vf
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::{prelude::*, Rng};

    impl VideoFactory {
        pub fn test() -> Self {
            let mut vf = VideoFactory::create();
            vf.set_title(String::new());
            vf.set_link(String::new());
            vf.set_origin_url(String::new());
            vf.set_origin_channel_name(String::new());
            vf.set_pub_date(String::new());
            vf.set_marked(false);
            vf.set_new(false);

            vf
        }
    }

    pub fn get_random_video_factory() -> VideoFactory {
        let mut rng = thread_rng();
        if rng.gen::<f64>() > 0.5 {
            get_unmarked_video_factory()
        } else {
            get_unmarked_video_factory()
        }
    }

    pub fn get_marked_video_factory() -> VideoFactory {
        let mut vf = VideoFactory::test();
        vf.set_marked(true);

        vf
    }

    pub fn get_unmarked_video_factory() -> VideoFactory {
        let mut vf = VideoFactory::test();

        vf.set_marked(false);

        vf
    }
}
