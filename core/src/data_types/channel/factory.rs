use crate::{
    data_types::{
    channel::channel::Channel,
    video::{factory::VideoFactory, video::Video},
    },
    SortingMethod,
};

pub struct ChannelFactory {
    channel: Channel,
    new_videos: Vec<VideoFactory>,
    old_videos: Vec<Video>,
    name_set: bool,
    id_set: bool,
    tag_set: bool,
    new_videos_set: bool,
    old_videos_set: bool,
    sorting_set: bool,
}

impl ChannelFactory {
    pub fn create() -> ChannelFactory {
        let channel = Channel::new();
        let new_videos = Vec::new();
        let old_videos = Vec::new();

        ChannelFactory {
            channel,
            new_videos,
            old_videos,
            name_set: false,
            id_set: false,
            tag_set: false,
            new_videos_set: false,
            old_videos_set: false,
            sorting_set: false,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.channel.name = name;
        self.name_set = true;
    }

    pub fn name_set(&self) -> bool {
        self.name_set
    }

    pub fn set_id(&mut self, id: String) {
        self.channel.id = id;
        self.id_set = true;
    }

    pub fn set_tag(&mut self, tag: String) {
        self.channel.tag = tag;
        self.tag_set = true;
    }

    pub fn add_new_videos(&mut self, videos: Vec<VideoFactory>) {
        for video in videos.into_iter() {
            if !self.new_videos.iter().any(|v| v == &video) {
                self.new_videos.push(video);
            }
        }

        self.new_videos_set = true;
    }

    pub fn new_videos_added(&self) -> bool {
        self.new_videos_set
    }

    pub fn set_old_videos(&mut self, videos: Vec<Video>) {
        self.old_videos = videos;
        self.old_videos_set = true;
    }

    pub fn set_sorting(&mut self, sorting_method: SortingMethod) {
        self.channel.sorting_method = sorting_method;
        self.sorting_set = true;
    }

    pub fn commit(mut self) -> Result<Channel, String> {
        if !self.name_set {
            return Err(String::from("name not set"));
        }

        if !self.id_set {
            return Err(String::from("id not set"));
        }

        if !self.tag_set {
            return Err(String::from("tag not set"));
        }

        if !self.new_videos_set {
            return Err(String::from("new_videos not set"));
        }

        if !self.old_videos_set {
            return Err(String::from("old_videos not set"));
        }

        if !self.sorting_set {
            return Err(String::from("sorting not set"));
        }

        // -------------------------------------------------------------

        let mut videos = self.old_videos;

        for video_factory in self.new_videos.into_iter() {
            let video = match video_factory.commit() {
                Ok(video) => video,
                Err(error) => return Err(error),
            };
            if !videos.iter().any(|v| v == &video) {
                videos.push(video);
            }
        }

        self.channel.videos = videos;

        self.channel.sort();

        Ok(self.channel) // if all correct return channel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};

    fn random_string() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(9)
            .map(char::from)
            .collect()
    }

    impl ChannelFactory {
        pub fn test() -> Self {
            let mut cf = ChannelFactory::create();
            cf.set_name(String::new());
            cf.set_id(random_string());
            cf.set_tag(String::new());
            cf.add_new_videos(Vec::new());
            cf.set_old_videos(Vec::new());
            cf.set_sorting(SortingMethod::Date);

            cf
        }
    }
}
