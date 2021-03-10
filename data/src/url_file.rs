use serde::{Deserialize, Serialize};
use std::{
    fs::File,
};
use std::{
    io::prelude::*,
};
use chrono::prelude::*;

use dirs_next::home_dir;

pub type ChannelId = String;
pub type ChannelTag = String;
pub type ChannelName = String;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Date {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
    Workday,
    Weekend,
    Always,
    Never,
}

impl Date {
    pub fn eq_to(&self, other: &Weekday) -> bool {
        match (self, other) {
            (Date::Mon, Weekday::Mon) |
            (Date::Tue, Weekday::Tue) |
            (Date::Wed, Weekday::Wed) |
            (Date::Thu, Weekday::Thu) |
            (Date::Fri, Weekday::Fri) |
            (Date::Sat, Weekday::Sat) |
            (Date::Sun, Weekday::Sun) |

            (Date::Workday, Weekday::Mon) |
            (Date::Workday, Weekday::Tue) |
            (Date::Workday, Weekday::Wed) |
            (Date::Workday, Weekday::Thu) |
            (Date::Workday, Weekday::Fri) |

            (Date::Weekend, Weekday::Sat) |
            (Date::Weekend, Weekday::Sun) |

            (Date::Always, _) => true,

            _ => false
        }
    }
}

const URLS_FILE_PATH: &str = ".config/tyt/urls.yaml";

// url file video type
#[derive(Deserialize, Serialize)]
pub struct UrlFile {
    #[serde(default = "empty_url_file_channel")]
    pub channels: Vec<UrlFileChannel>,
    #[serde(default = "empty_url_file_custom_channels")]
    pub custom_channels: Vec<UrlFileCustomChannel>,
}

pub trait UrlFileItem {
    fn id(&self) -> ChannelId;
    fn update_on(&self) -> Vec<Date>;
    fn tag(&self) -> ChannelTag;
    fn name(&self) -> ChannelName;
}

// url file video type
#[derive(Clone, Deserialize, Serialize)]
pub struct UrlFileChannel {
    pub url: String,
    #[serde(default = "empty_string")]
    name: ChannelName,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default = "empty_string")]
    tag: ChannelTag,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UrlFileCustomChannel {
    pub urls: Vec<String>,
    pub name: ChannelName,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default = "empty_string")]
    tag: ChannelTag,
}

impl UrlFileItem for UrlFileChannel {
    fn id(&self) -> ChannelId {
        self.url.clone()
    }
    fn update_on(&self) -> Vec<Date> {
        self.update_on.clone()
    }
    fn tag(&self) -> ChannelTag {
        self.tag.clone()
    }
    fn name(&self) -> ChannelName {
        self.name.clone()
    }
}

impl UrlFileItem for UrlFileCustomChannel {
    fn id(&self) -> ChannelId {
        self.name.clone()
    }
    fn update_on(&self) -> Vec<Date> {
        self.update_on.clone()
    }
    fn tag(&self) -> ChannelTag {
        self.tag.clone()
    }
    fn name(&self) -> ChannelName {
        self.name.clone()
    }
}

fn empty_url_file_channel() -> Vec<UrlFileChannel> { Vec::new() }
fn empty_url_file_custom_channels() -> Vec<UrlFileCustomChannel> { Vec::new() }
fn date_always() -> Vec<Date> { vec![Date::Always] }
fn empty_string() -> String { String::new() }

// impl UrlFile {
impl UrlFile {
    pub fn len(&self) -> usize {
        self.channels.len() + self.custom_channels.len()
    }
}

pub fn read_urls_file() -> UrlFile {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let items: UrlFile = match serde_yaml::from_str(&reader) {
                Ok(file) => file,
                Err(e) => panic!("could not parse yaml url_file: {}", e),
            };

            items
        }
        Err(_) => {
            let mut file = File::create(path).unwrap();
            let channel: UrlFile = UrlFile {
                channels: Vec::new(),
                custom_channels: Vec::new(),
            };
            let string = serde_yaml::to_string(&channel).unwrap();
            match file.write_all(string.as_bytes()) {
                Ok(_) => read_urls_file(),
                Err(e) => panic!("{}", e),
            }
        }
    }
}
