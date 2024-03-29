use super::{date_always, Date, SubscriptionItem};
use crate::backend::SortingMethodVideos;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CustomChannelSubscription {
    pub urls: Vec<String>,
    pub name: String,
    block_regex: Option<String>,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default)]
    tag: String,
    #[serde(default)]
    sort_by: SortingMethodVideos,
}

impl SubscriptionItem for CustomChannelSubscription {
    fn id(&self) -> String {
        self.name.clone()
    }
    fn active(&self) -> bool {
        let today = Local::now().weekday();
        self.update_on.iter().any(|w| w.eq_to(&today))
    }
    fn tag(&self) -> String {
        self.tag.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn sorting_method(&self) -> SortingMethodVideos {
        self.sort_by
    }
    fn block_regex(&self) -> &Option<String> {
        &self.block_regex
    }
}
