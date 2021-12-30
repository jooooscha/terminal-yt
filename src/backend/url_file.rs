use crate::backend::SortingMethod;
use chrono::prelude::*;
use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

pub type ChannelId = String;
pub type ChannelTag = String;
pub type ChannelName = String;

#[cfg(debug_assertions)]
const URLS_FILE_PATH: &str = ".config/tyt/urls_debug.yml";

#[cfg(not(debug_assertions))]
const URLS_FILE_PATH: &str = ".config/tyt/urls.yml";

#[derive(Clone, Deserialize, Serialize, Debug)]
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
        matches!((self, other), (Date::Mon, Weekday::Mon)
            | (Date::Tue, Weekday::Tue)
            | (Date::Wed, Weekday::Wed)
            | (Date::Thu, Weekday::Thu)
            | (Date::Fri, Weekday::Fri)
            | (Date::Sat, Weekday::Sat)
            | (Date::Sun, Weekday::Sun)
            | (Date::Workday, Weekday::Mon)
            | (Date::Workday, Weekday::Tue)
            | (Date::Workday, Weekday::Wed)
            | (Date::Workday, Weekday::Thu)
            | (Date::Workday, Weekday::Fri)
            | (Date::Weekend, Weekday::Sat)
            | (Date::Weekend, Weekday::Sun)
            | (Date::Always, _)
        )
    }
}

// url file video type
#[derive(Deserialize, Serialize, Debug)]
pub struct UrlFile {
    #[serde(default)]
    pub channels: Vec<UrlFileChannel>,
    #[serde(default)]
    pub custom_channels: Vec<UrlFileCustomChannel>,
}

pub trait UrlFileItem {
    fn id(&self) -> ChannelId;
    /* fn update_on(&self) -> Vec<Date>; */
    fn active(&self) -> bool;
    fn tag(&self) -> ChannelTag;
    fn name(&self) -> ChannelName;
    fn sorting_method(&self) -> SortingMethod;
}

// url file video type
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UrlFileChannel {
    pub url: String,
    #[serde(default)]
    name: ChannelName,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default)]
    tag: ChannelTag,
    #[serde(default)]
    sort_by: SortingMethod,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UrlFileCustomChannel {
    pub urls: Vec<String>,
    pub name: ChannelName,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default)]
    tag: ChannelTag,
    #[serde(default)]
    sort_by: SortingMethod,
}

impl UrlFileItem for UrlFileChannel {
    fn id(&self) -> ChannelId {
        self.url.clone()
    }
    fn active(&self) -> bool {
        let today = Local::now().weekday();
        self.update_on.iter().any(|w| w.eq_to(&today))
    }
    fn tag(&self) -> ChannelTag {
        self.tag.clone()
    }
    fn name(&self) -> ChannelName {
        self.name.clone()
    }
    fn sorting_method(&self) -> SortingMethod {
        self.sort_by
    }
}

impl UrlFileItem for UrlFileCustomChannel {
    fn id(&self) -> ChannelId {
        self.name.clone()
    }
    fn active(&self) -> bool {
        let today = Local::now().weekday();
        self.update_on.iter().any(|w| w.eq_to(&today))
    }
    fn tag(&self) -> ChannelTag {
        self.tag.clone()
    }
    fn name(&self) -> ChannelName {
        self.name.clone()
    }
    fn sorting_method(&self) -> SortingMethod {
        self.sort_by
    }
}

// impl UrlFile {
impl UrlFile {
    /// checks wheather the url file contains a channel with the given id
    pub fn contains_channel_by_id(&self, id: &str) -> bool {
        let in_channels = self.channels.iter().any(|channel| channel.id() == id);
        let in_custom_channels = self
            .custom_channels
            .iter()
            .any(|channel| channel.id() == id);

        in_channels || in_custom_channels
    }
}

fn date_always() -> Vec<Date> {
    vec![Date::Always]
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

/* #[cfg(test)]
 * pub mod tests {
 *     use super::{Date, *};
 *
 *     impl UrlFileChannel {
 *         pub fn test(name: String, tag: String, url: String) -> Self {
 *             let update_on = vec![Date::Mon];
 *             let sorting_method = SortingMethod::Date;
 *
 *             UrlFileChannel {
 *                 name,
 *                 update_on,
 *                 tag,
 *                 url,
 *                 sorting_method,
 *             }
 *         }
 *     }
 *
 *     impl UrlFileCustomChannel {
 *         pub fn test(name: String, tag: String, urls: Vec<String>) -> Self {
 *             let update_on = vec![Date::Mon];
 *             let sorting_method = SortingMethod::Date;
 *
 *             UrlFileCustomChannel {
 *                 name,
 *                 update_on,
 *                 tag,
 *                 urls,
 *                 sorting_method,
 *             }
 *         }
 *     }
 *
 *     impl UrlFile {
 *         pub fn test(custom_channels: Vec<UrlFileCustomChannel>) -> Self {
 *             let channels = Vec::new();
 *
 *             UrlFile {
 *                 channels,
 *                 custom_channels,
 *             }
 *         }
 *     }
 * } */
