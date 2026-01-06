use super::{date_always, Date, SubscriptionItem};
use crate::backend::SortingMethodVideos;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

// url file video type
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ChannelSubscription {
    pub url: String,
    block_regex: Option<String>,
    #[serde(default)]
    name: String,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default)]
    tag: String,
    #[serde(default)]
    sort_by: SortingMethodVideos,
    #[serde(default)]
    download: bool,
}

impl Default for ChannelSubscription {
    fn default() -> Self {
        Self {
            url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A"
                .to_string(),
            name: "Tom Scott".to_string(),
            update_on: vec![Date::default()],
            tag: "Interresting".to_string(),
            sort_by: SortingMethodVideos::default(),
            block_regex: None,
            download: false,
        }
    }
}

impl SubscriptionItem for ChannelSubscription {
    fn id(&self) -> String {
        self.url.clone()
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
