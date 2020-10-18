use std::{
    fs::File,
    io::prelude::*,
};
use dirs::home_dir;

use data_types::{
    internal::{
        ChannelList,
    },
};

const HISTORY_FILE_PATH: &str = ".config/tyt/history.json";

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
            let channel_list: ChannelList = match serde_json::from_str(&reader) {
                Ok(channels) => channels,
                Err(e) => panic!("could not read history file: {}", e),
            };

            // return
            Some(channel_list)
        }
        Err(_) => None,
    }
}

