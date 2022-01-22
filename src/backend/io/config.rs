use crate::backend::{
    io::{read_config, FileType::ConfigFile},
    Error::ParseConfig,
    Result, SortingMethod,
};
use serde::{Deserialize, Serialize};

const SCHOW_EMPTY_CHANNEL_DEFAULT: bool = true;
const MARK_ON_OPEN_DEFAULT: bool = true;
const DOWN_ON_MARK_DEFAULT: bool = true;
const APP_TITLE_DEFAULT: &str = "TYT";
const UPDATAE_AT_START_DEFAULT: bool = true;
const SORT_BY_TAG_DEFAULT: bool = false;
const MASSAGE_TIMEOUT_DEFAULT: usize = 20;
const NOTIFY_WITH_DEFAULT: &str = "notify_send";
const VIDEO_PLAYER_DEFAULT: &str = "mpv";
/* const DEFAULT_SORT: SortingMethod = SortingMethod::default(); */

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) show_empty_channels: bool,
    pub(crate) mark_on_open: bool,
    pub(crate) down_on_mark: bool,
    pub(crate) app_title: String,
    pub(crate) update_at_start: bool,
    pub(crate) sort_by_tag: bool,
    pub(crate) message_timeout: usize,
    pub(crate) video_player: String,
    pub(crate) default_sorting_method: SortingMethod,
    pub(crate) notify_with: String,
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
            default_sorting_method: SortingMethod::default(),
        }
    }
}

impl Config {
    /// Inits the Config struct. Write default config, if config could not be found
    pub(crate) fn read() -> Result<Self> {
        let config_str = read_config(ConfigFile);
        match serde_yaml::from_str(&config_str) {
            Ok(config) => Ok(config),
            Err(error) => Err(ParseConfig(error)),
        }
    }
}
