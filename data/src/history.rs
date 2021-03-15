use std::{
    fs::File,
    io::prelude::*,
};
use dirs_next::home_dir;

use crate::internal::{
    ChannelList,
    MinimalVideo,
};

const HISTORY_FILE_PATH: &str = ".config/tyt/history.json";
const PLAYBACK_HISTORY_PATH: &str = ".config/tyt/playback_history.json";

pub fn write_history(channel_list: &ChannelList) {
    let json = serde_json::to_string(channel_list).unwrap();

    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => panic!("history write error: {}", e),
    };
    file.write_all(json.as_bytes()).unwrap();
}

pub fn read_history() -> Option<ChannelList> {
    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let mut channel_list: ChannelList = match serde_json::from_str(&reader) {
                Ok(channels) => channels,
                Err(e) => panic!("could not read history file: {}", e),
            };

            channel_list.remove_old();

            // return
            Some(channel_list)
        }
        Err(_) => None,
    }
}

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
