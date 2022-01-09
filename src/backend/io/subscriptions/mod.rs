use crate::{
    backend::{
        SortingMethod,
        io::{read_config, FileType::SubscriptionsFile},
        Error::ParseSubscription,
        Result,
    },
};
use serde::{Deserialize, Serialize};
use chrono::Weekday;
use channel::ChannelSubscription;
use custom_channel::CustomChannelSubscription;

mod channel;
mod custom_channel;

/// Trait for all channel types
pub(crate) trait SubscriptionItem {
    fn id(&self) -> String;
    fn active(&self) -> bool;
    fn tag(&self) -> String;
    fn name(&self) -> String;
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
    pub(crate) fn read() -> Result<Self> {
        let config_file = read_config(SubscriptionsFile);

        match serde_yaml::from_str::<Self>(&config_file) {
            Ok(file) => Ok(file),
            Err(error) => {
                Err(ParseSubscription(error))
            }
        }
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

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Date {
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

impl Default for Date {
    fn default() -> Self {
        Self::Always
    }
}

impl Date {
    pub(crate) fn eq_to(&self, other: &Weekday) -> bool {
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
