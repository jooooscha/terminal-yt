use crate::{
    backend::{
        SortingMethod,
        io::{read_config, FileType::SubscriptionsFile},
    },
    notification::notify_error,
};
use serde::{Deserialize, Serialize};
use channel::ChannelSubscription;
use custom_channel::CustomChannelSubscription;
use date::Date;

mod date;
mod channel;
mod custom_channel;

pub type ChannelId = String;
pub type ChannelTag = String;
pub type ChannelName = String;

/// Trait for all channel types
pub(crate) trait SubscriptionItem {
    fn id(&self) -> ChannelId;
    fn active(&self) -> bool;
    fn tag(&self) -> ChannelTag;
    fn name(&self) -> ChannelName;
    fn sorting_method(&self) -> SortingMethod;
}

/// Default value for date always
fn date_always() -> Vec<Date> {
    vec![Date::Always]
}

/// Struct for all Subscriptions
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Subscriptions {
    #[serde(default)]
    pub channels: Vec<ChannelSubscription>,
    #[serde(default)]
    pub custom_channels: Vec<CustomChannelSubscription>,
}

impl Default for Subscriptions {
    fn default() -> Self {
        Self {
            channels: vec![ChannelSubscription::default()],
            custom_channels: Vec::new(),
        }
    }
}

impl Subscriptions {
    /// Read Subscriptions file or create default
    pub(crate) fn read() -> Self {
        let config_file = read_config(SubscriptionsFile);

        let subs: Subscriptions = match serde_yaml::from_str(&config_file) {
            Ok(file) => file,
            Err(e) => {
                notify_error(&format!("could not parse subscriptions file: {}", e));
                return Self::default();
            }
        };

        subs
    }

    /// checks wheather the url file contains a channel with the given id
    pub(crate) fn contains_channel_by_id(&self, id: &str) -> bool {
        let in_channels = self.channels.iter().any(|channel| channel.id() == id);
        let in_custom_channels = self
            .custom_channels
            .iter()
            .any(|channel| channel.id() == id);

        in_channels || in_custom_channels
    }
}

/* pub(crate) fn read_urls_file() -> SubscriptionsFile {
 *     let mut path = home_dir().unwrap();
 *     path.push(URLS_FILE_PATH);
 *
 *     match File::open(path.clone()) {
 *         Ok(mut file) => {
 *             let mut reader = String::new();
 *             file.read_to_string(&mut reader).unwrap();
 *             let items: SubscriptionsFile = match serde_yaml::from_str(&reader) {
 *                 Ok(file) => file,
 *                 Err(e) => panic!("could not parse yaml url_file: {}", e),
 *             };
 *
 *             items
 *         }
 *         Err(_) => {
 *             let mut file = File::create(path).unwrap();
 *             let channel: SubscriptionsFile = SubscriptionsFile {
 *                 channels: Vec::new(),
 *                 custom_channels: Vec::new(),
 *             };
 *             let string = serde_yaml::to_string(&channel).unwrap();
 *             match file.write_all(string.as_bytes()) {
 *                 Ok(_) => read_urls_file(),
 *                 Err(e) => panic!("{}", e),
 *             }
 *         }
 *     }
 * } */

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
