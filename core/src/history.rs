use dirs_next::home_dir;
use std::{fs::File, io::prelude::*};

use crate::data_types::channel_list::ChannelList;
use crate::{data_types::video::video::Video, ToTuiListItem};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};

#[cfg(debug_assertions)]
const HISTORY_FILE_PATH: &str = ".config/tyt/history_debug.json";
#[cfg(debug_assertions)]
const PLAYBACK_HISTORY_PATH: &str = ".config/tyt/playback_history_debug.json";

#[cfg(not(debug_assertions))]
const HISTORY_FILE_PATH: &str = ".config/tyt/history.json";
#[cfg(not(debug_assertions))]
const PLAYBACK_HISTORY_PATH: &str = ".config/tyt/playback_history.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinimalVideo {
    pub title: String,
    pub channel: String,
}

impl ToTuiListItem for MinimalVideo {
    fn to_list_item(&self) -> ListItem {
        let channel = format!("{} {} - ", tui::symbols::DOT, &self.channel);
        let title = format!("{}", &self.title);

        let style = Style::default().fg(Color::DarkGray);

        ListItem::new(Spans::from(vec![
            Span::styled(channel, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC)),
        ]))
    }
}

impl From<Video> for MinimalVideo {
    fn from(video: Video) -> MinimalVideo {
        MinimalVideo {
            title: video.title().clone(),
            channel: video.origin_channel_name().clone(),
        }
    }
}

// ------------------------------------------------------------------------------------------

pub fn write_history(channel_list: &ChannelList) {
    write_history_intern(channel_list, HISTORY_FILE_PATH);
}

fn write_history_intern(channel_list: &ChannelList, history_path: &str) {
    let json = serde_json::to_string(channel_list).unwrap();

    let mut path = home_dir().unwrap();
    path.push(history_path);

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => panic!("history write error: {}", e),
    };
    file.write_all(json.as_bytes()).unwrap();
}

pub fn read_history() -> ChannelList {
    let mut cl = read_history_intern(HISTORY_FILE_PATH);
    cl.apply_url_file_changes(); // update all things that have changed in url file
    cl
}

fn read_history_intern(history_path: &str) -> ChannelList {
    let mut path = home_dir().unwrap();
    path.push(history_path);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let channel_list: ChannelList = match serde_json::from_str(&reader) {
                Ok(channels) => channels,
                Err(e) => panic!("could not read history file: {}", e),
            };

            // return
            channel_list
        }
        Err(_) => ChannelList::new(),
    }
}

// ------------------------------------------------------------------------------------------

pub fn write_playback_history(list: &Vec<MinimalVideo>) {
    let json = serde_json::to_string(list).unwrap();

    let mut path = home_dir().unwrap();
    path.push(PLAYBACK_HISTORY_PATH);

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => panic!("playback write error: {}", e),
    };
    file.write_all(json.as_bytes()).unwrap();
}

pub fn read_playback_history() -> Vec<MinimalVideo> {
    let mut path = home_dir().unwrap();
    path.push(PLAYBACK_HISTORY_PATH);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let playback_history: Vec<MinimalVideo> = match serde_json::from_str(&reader) {
                Ok(channels) => channels,
                Err(_) => Vec::new(),
            };

            playback_history
        }
        Err(_) => Vec::new(),
    }
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::data_types::{
        channel::factory::ChannelFactory, video::factory::tests::get_random_video_factory,
    };
    use std::fs::remove_file;

    #[test]
    fn test_rw_history() {
        let mut channels = Vec::new();
        for _ in 0..10 {
            let mut cf = ChannelFactory::test();

            let mut videos = Vec::new();
            for _ in 0..10 {
                videos.push(get_random_video_factory());
            }
            cf.add_new_videos(videos);

            let channel = cf.commit().unwrap();
            channels.push(channel);
        }

        let input = ChannelList::test(channels);

        let file = "./test_write_history";

        write_history_intern(&input, file);
        let output = read_history_intern(file);

        assert_eq!(input, output);

        let _ = remove_file(file);
    }
}
