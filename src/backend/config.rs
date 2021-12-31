use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write, ErrorKind},
};
use crate::{
    backend::SortingMethod,
    notification::notify_error,
};

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
    /// Inits the Config struct. Write default config, if config could not be found
    pub(crate) fn init() -> Self {
        let mut path = match home_dir() {
            Some(p) => p,
            None => {
                notify_error("could not read home dir");
                return Self::default();
            }
        };
        path.push(CONFIG_FILE_PATH);

        let file_result = OpenOptions::new()
            .read(true)
            .create(true)
            .open(path);

        if let Err(error) = file_result {
            match error.kind() {
                ErrorKind::NotFound => {
                    Self::write_default();
                }
                ErrorKind::PermissionDenied => {
                    notify_error("Permission to config denied");
                }
                _ => {},
            }
            return Self::default();
        }

        // save because we return on Err(...)
        let mut file = file_result.unwrap();

        let mut buffer = String::new();
        if let Err(e) = file.read_to_string(&mut buffer) {
            notify_error(&format!("Data is no valid utf-8: {}", e));
            return Self::default();
        }

        let config: Config = match serde_yaml::from_str(&buffer) {
            Ok(config) => config,
            Err(e) => {
                notify_error(&format!("could not parse config file: {}", e));
                return Self::default();
            }
        };

        config
    }

    /// Writes the default config
    /// # Panics
    /// Panics if file could not be opended or already exists
    fn write_default() {
        let mut path = match home_dir() {
            Some(p) => p,
            None => {
                notify_error("could not read home dir");
                return;
            }
        };

        path.push(CONFIG_FILE_PATH);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();

        let string = serde_yaml::to_string(&Config::default()).unwrap();
        let _ = file.write_all(string.as_bytes());
    }
}
