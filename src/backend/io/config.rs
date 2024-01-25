use crate::backend::{
    io::{read_config, FileType::ConfigFile},
    Error::ParseConfig,
    Result, SortingMethodVideos, SortingMethodChannels,
};
use serde::{Deserialize, Serialize};

const SCHOW_EMPTY_CHANNEL_DEFAULT: bool = true;
const MARK_ON_OPEN_DEFAULT: bool = true;
const DOWN_ON_MARK_DEFAULT: bool = true;
const APP_TITLE_DEFAULT: &str = "TYT";
const UPDATAE_AT_START_DEFAULT: bool = true;
const MASSAGE_TIMEOUT_DEFAULT: usize = 20;
const NOTIFY_WITH_DEFAULT: &str = "notify_send";
const VIDEO_PLAYER_DEFAULT: &str = "mpv";
const USE_DEARROW_DEFAULT: bool = false;
/* const DEFAULT_SORT: SortingMethod = SortingMethod::default(); */

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub show_empty_channels: bool,
    pub mark_on_open: bool,
    pub down_on_mark: bool,
    pub app_title: String,
    pub update_at_start: bool,
    pub sort_channels: SortingMethodChannels,
    pub message_timeout: usize,
    pub video_player: String,
    pub sort_videos: SortingMethodVideos,
    pub notify_with: String,
    pub use_dearrow_titles: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_empty_channels: SCHOW_EMPTY_CHANNEL_DEFAULT,
            mark_on_open: MARK_ON_OPEN_DEFAULT,
            down_on_mark: DOWN_ON_MARK_DEFAULT,
            app_title: APP_TITLE_DEFAULT.into(),
            update_at_start: UPDATAE_AT_START_DEFAULT,
            sort_channels: SortingMethodChannels::default(),
            message_timeout: MASSAGE_TIMEOUT_DEFAULT,
            notify_with: NOTIFY_WITH_DEFAULT.into(),
            video_player: VIDEO_PLAYER_DEFAULT.into(),
            sort_videos: SortingMethodVideos::default(),
            use_dearrow_titles: USE_DEARROW_DEFAULT,
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
