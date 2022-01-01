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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CustomChannelSubscription {
    pub urls: Vec<String>,
    pub name: ChannelName,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default)]
    tag: ChannelTag,
    #[serde(default)]
    sort_by: SortingMethod,
}

impl SubscriptionItem for CustomChannelSubscription {
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