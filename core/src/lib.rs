pub mod core;
pub mod data_types {
    pub mod channel {
        pub mod channel;
        pub mod factory;
    }
    pub(crate) mod channel_list;
    pub(crate) mod feed_types;
    pub(crate) mod video {
        pub(crate) mod factory;
        pub(crate) mod video;
    }
}

mod config;
mod draw;
pub mod fetch_data;
mod history;
mod url_file;

use serde::{Deserialize, Serialize};
use tui::widgets::ListItem;
use crate::config::Config;
pub trait ToTuiListItem {
    fn to_list_item(&self) -> ListItem;
}

#[derive(PartialEq, Clone, Copy)]
pub enum Filter {
    NoFilter,
    OnlyNew,
}

#[derive(PartialEq)]
pub enum Action {
    Mark,
    Unmark,
    Up,
    Down,
    Enter,
    Leave,
    NextChannel,
    PrevChannel,
    Open,
    SetVideoFav,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Screen {
    Channels,
    Videos,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortingMethod {
    Date,
    Text,
    UnseenDate,
    UnseenText,
}

impl Default for SortingMethod {
    fn default() -> Self {
        Config::read_config_file().default_sorting_method
    }
}
