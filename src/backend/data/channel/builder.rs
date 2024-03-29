use crate::backend::{
    data::{
        channel::Channel,
        feed::Feed,
        video::{builder::VideoBuilder, Video},
    },
    SortingMethodVideos,
    dearrow,
};

#[derive(Default, Clone)]
pub struct ChannelBuilder {
    channel: Channel,
    new_videos: Vec<VideoBuilder>,
    old_videos: Vec<Video>,
}

impl ChannelBuilder {
    pub(crate) fn add_from_feed(mut self, feed: Feed) -> Self {
        self.channel.name = feed.name;
        self.new_videos = feed.videos;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.channel.name = name;
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.channel.id = id;
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.channel.tag = tag;
        self
    }

    pub fn with_old_videos(mut self, videos: Vec<Video>) -> Self {
        self.old_videos = videos;
        self
    }

    pub fn with_sorting(mut self, sorting_method: SortingMethodVideos) -> Self {
        self.channel.sorting_method = sorting_method;
        self
    }

    pub fn use_dearrow(mut self) -> Self {

        for video in self.new_videos.iter_mut() {
            if let Some(id) = video.get_id() {
                let dearrow_title = dearrow::get_best_title(id);
                video.set_dearrow_title(dearrow_title);
            }
        }

        self
    }

    pub fn build(mut self) -> Channel {
        // set already known videos
        let mut videos = self.old_videos;

        // iterate over new videos and add unknown
        for video_factory in self.new_videos.into_iter() {
            let video = video_factory.build();
            let position = videos.iter().position(|v| v == &video);
            if let Some(i) = position {
                let v = videos.get_mut(i).unwrap();
                v.title = video.title;
                v.dearrow_title = video.dearrow_title;
            } else {
                videos.push(video);
            }
        }

        self.channel.videos = videos;
        self.channel.sort();

        // return finished channel
        self.channel.clone()
    }
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use rand::{distributions::Alphanumeric, Rng};
 *
 *     fn random_string() -> String {
 *         rand::thread_rng()
 *             .sample_iter(&Alphanumeric)
 *             .take(9)
 *             .map(char::from)
 *             .collect()
 *     }
 *
 *     impl ChannelFactory {
 *         pub fn test() -> Self {
 *             let mut cf = ChannelFactory::create();
 *             cf.set_name(String::new());
 *             cf.set_id(random_string());
 *             cf.set_tag(String::new());
 *             cf.add_new_videos(Vec::new());
 *             cf.set_old_videos(Vec::new());
 *             cf.set_sorting(SortingMethod::Date);
 *
 *             cf
 *         }
 *     }
 * } */
