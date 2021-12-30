use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};
use crate::backend::SortingMethod;

const CONFIG_FILE_PATH: &str = ".config/tyt/config.yml";
const SCHOW_EMPTY_CHANNEL_DEFAULT: bool = true;
const MARK_ON_OPEN_DEFAULT: bool = true;
const DOWN_ON_MARK_DEFAULT: bool = true;
const APP_TITLE_DEFAULT: &str = "TYT";
const UPDATAE_AT_START_DEFAULT: bool = true;
const SORT_BY_TAG_DEFAULT: bool = false;
const MASSAGE_TIMEOUT_DEFAULT: usize = 20;
const NOTIFY_WITH_DEFAULT: &str = "notify_send";
const VIDEO_PLAYER_DEFAULT: &str = "mpv";
const DEFAULT_SORT: SortingMethod = SortingMethod::Date;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
    pub show_empty_channels: bool,
    pub mark_on_open: bool,
    pub down_on_mark: bool,
    pub app_title: String,
    pub update_at_start: bool,
    pub sort_by_tag: bool,
    pub message_timeout: usize,
    pub video_player: String,
    pub default_sorting_method: SortingMethod,
    pub notify_with: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_empty_channels: SCHOW_EMPTY_CHANNEL_DEFAULT,
            mark_on_open: MARK_ON_OPEN_DEFAULT,
            down_on_mark: DOWN_ON_MARK_DEFAULT,
            app_title: APP_TITLE_DEFAULT.into(),
            update_at_start: UPDATAE_AT_START_DEFAULT,
            sort_by_tag: SORT_BY_TAG_DEFAULT,
            message_timeout: MASSAGE_TIMEOUT_DEFAULT,
            notify_with: NOTIFY_WITH_DEFAULT.into(),
            video_player: VIDEO_PLAYER_DEFAULT.into(),
            default_sorting_method: DEFAULT_SORT,
        }
    }
}

impl Config {
    pub(crate) fn init() -> Self {
        let mut path = home_dir().unwrap();
        path.push(CONFIG_FILE_PATH);

        match File::open(path.clone()) {
            Ok(mut file) => {
                let mut reader = String::new();
                file.read_to_string(&mut reader).unwrap();
                let config: Config = match serde_yaml::from_str(&reader) {
                    Ok(file) => file,
                    Err(e) => panic!("could not parse config file: {}", e),
                };

                config
            }
            Err(_) => {
                match File::create(path) {
                    Ok(mut file) => {
                        let def_config = Config::default();
                        let string = serde_yaml::to_string(&def_config).unwrap();

                        match file.write_all(string.as_bytes()) {
                            Ok(_) => Config::init(),
                            Err(e) => panic!("could not write default config: {}", e),
                        }
                    }
                    Err(e) => panic!("could not create config file: {}", e),
                }
            }
        }
    }
}
