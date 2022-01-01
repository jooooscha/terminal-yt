use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use super::{
    ChannelId,
    ChannelTag,
    ChannelName,
    date::Date,
    SubscriptionItem,
    date_always,
};
use crate::backend::SortingMethod;

// url file video type
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ChannelSubscription {
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

impl Default for ChannelSubscription {
    fn default() -> Self {
        Self {
            url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A".to_string(),
            name: "Tom Scott".to_string(),
            update_on: vec![Date::default()],
            tag: "Interresting".to_string(),
            sort_by: SortingMethod::default(),
        }
    }
}

impl SubscriptionItem for ChannelSubscription {
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