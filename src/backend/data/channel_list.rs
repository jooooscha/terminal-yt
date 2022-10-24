#![allow(unused)]

use crate::backend::{
    data::channel::Channel,
    io::subscriptions::{SubscriptionItem, Subscriptions},
    io::{read_config, FileType::DbFile},
    Error::ParseDB,
    Filter::{self, *},
    SortingMethodChannels,
    Result, ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use tui::widgets::{ListItem, ListState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChannelList {
    channels: Vec<Channel>,

    #[serde(skip)]
    list_state: ListState,
    #[serde(skip)]
    filter: Filter,
}

impl Default for ChannelList {
    fn default() -> Self {
        Self {
            channels: vec![Channel::default()],
            list_state: ListState::default(),
            filter: Filter::NoFilter,
        }
    }
}

#[allow(clippy::unnecessary_unwrap)]
impl ChannelList {
    pub(crate) fn load() -> Result<Self> {
        let db_file = read_config(DbFile);

        match serde_json::from_str::<Self>(&db_file) {
            Ok(mut channel_list) => {
                channel_list.apply_url_file_changes();
                Ok(channel_list)
            }
            Err(error) => Err(ParseDB(error)),
        }
    }

    pub(crate) fn next(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => {
                if i + 1 < self.len() {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    pub(crate) fn prev(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    //---------------------------------------------------------------

    pub(crate) fn len(&self) -> usize {
        match self.filter {
            NoFilter => self.channels.len(),
            OnlyNew => self.channels.iter().filter(|c| c.videos.iter().any(|v| !v.marked())).count()
        }
    }

    pub fn select(&mut self, i: Option<usize>) {
        if self.len() == 0 || i.is_none() {
            self.list_state.select(None);
        } else {
            let pos = min(i.unwrap(), self.len()-1);
            self.list_state.select(Some(pos));
        }
    }

    pub(crate) fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub(crate) fn state(&self) -> ListState {
        self.list_state.clone()
    }

    pub(crate) fn push(&mut self, channel: Channel) {
        self.channels.push(channel);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Channel> {
        match self.filter {
            NoFilter => self.channels.get(index),
            OnlyNew => self.channels.iter().filter(|c| c.videos.iter().any(|v| !v.marked())).nth(index)
        }
    }

    pub(crate) fn get_unfiltered(&self, index: usize) -> Option<&Channel> {
        self.channels.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Channel> {
        match self.filter {
            NoFilter => self.channels.get_mut(index),
            OnlyNew => self.channels.iter_mut().filter(|c| c.videos.iter().any(|v| !v.marked())).nth(index)
        }
    }

    pub(crate) fn get_unfiltered_mut(&mut self, index: usize) -> Option<&mut Channel> {
        self.channels.get_mut(index)
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Channel> {
        let p = self.get_position_by_id(id)?;
        self.get(p)
    }

    pub fn get_unfiltered_by_id(&self, id: &str) -> Option<&Channel> {
        let p = self.get_unfiltered_position_by_id(id)?;
        self.get_unfiltered(p)
    }

    pub fn get_mut_by_id(&mut self, id: &str) -> Option<&mut Channel> {
        let p = self.get_position_by_id(id)?;
        self.get_mut(p)
    }

    pub fn get_unfiltered_mut_by_id(&mut self, id: &str) -> Option<&mut Channel> {
        let p = self.get_unfiltered_position_by_id(id)?;
        self.get_unfiltered_mut(p)
    }


    pub fn get_position_by_id(&self, id: &str) -> Option<usize> {
        match self.filter {
            NoFilter => self.channels.iter().position(|channel| channel.id() == id),
            OnlyNew => {
                self.channels.iter()
                    .filter(|c| c.videos.iter().any(|v| !v.marked()))
                    .position(|channel| channel.id() == id)
            }
        }
    }

    pub fn get_unfiltered_position_by_id(&self, id: &str) -> Option<usize> {
        self.channels.iter().position(|channel| channel.id() == id)
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    pub fn get_filter(&self) -> Filter {
        self.filter
    }

    pub fn toggle_filter(&mut self) {
        match self.filter {
            NoFilter => self.set_filter(OnlyNew),
            OnlyNew => self.set_filter(NoFilter),
        }
    }

    pub(crate) fn get_spans_list(&self) -> Vec<ListItem> {
        match self.filter {
            NoFilter => {
                self.channels
                    .iter()
                    .map(|channel| channel.to_list_item())
                    .collect()
            }
            OnlyNew => {
                self.channels
                    .iter()
                    .filter(|c| c.videos.iter().any(|v| !v.marked()))
                    .map(|channel| channel.to_list_item())
                    .collect()
            }
        }
    }

    /// Add new videos to already known channel
    pub fn update_channel(&mut self, updated_channel: Channel, sort: SortingMethodChannels) {
        let filter = self.get_filter();
        self.set_filter(Filter::NoFilter);

        let old_channel = self.get_mut_by_id(updated_channel.id()); // get old channel

        if let Some(channel) = old_channel {
            channel.merge_videos(updated_channel.videos); // merge videos of new and old version
        } else {
            self.push(updated_channel);
        }

        match sort {
            SortingMethodChannels::AlphaNumeric => {
                self.channels.sort_by_key(|channel| channel.name().clone().to_lowercase());
            }
            SortingMethodChannels::ByTag => {
                self.channels.sort_by_key(|channel| {
                    if channel.tag().is_empty() {
                        channel.name().clone().to_lowercase() // lowercase is sorted after uppercase
                    } else {
                        format!("{}{}", channel.tag().clone().to_uppercase(), channel.name().clone().to_uppercase())
                    }
                });
            }
        }

        self.set_filter(filter);
    }

    /// Filter all channels that are not in the UrlFile anymore
    fn remove_old(&mut self, url_file: &Subscriptions) {
        self.channels = self
            .channels
            .iter()
            .filter(|channel| url_file.contains_channel_by_id(channel.id()))
            .cloned()
            .collect();

        // remove videos that belong to urls removed from a custom channel
        for custom_channel in url_file.custom_channels.iter() {
            let urls = &custom_channel.urls;

            if let Some(mut channel) = self.get_mut_by_id(&custom_channel.id()) {
                channel.videos = channel
                    .videos
                    .iter()
                    .filter(|video| urls.contains(video.origin_url()))
                    .cloned()
                    .collect();
            }
        }
    }

    fn update_channels_from_url_file(&mut self, subs: &Subscriptions) {
        // update all "normal" channels
        for item in subs.channels.iter() {
            if let Some(ref mut chan) = self.get_mut_by_id(&item.id()) {
                chan.update_from_url_subs(item as &dyn SubscriptionItem);
            }
        }

        // update all custom channels
        for item in subs.custom_channels.iter() {
            if let Some(ref mut chan) = self.get_mut_by_id(&item.id()) {
                chan.update_from_url_subs(item as &dyn SubscriptionItem);
            }
        }
    }

    pub(crate) fn apply_url_file_changes(&mut self) {
        if let Ok(subs) = Subscriptions::read() {
            self.remove_old(&subs);
            self.update_channels_from_url_file(&subs);
        }
    }

    //---------------------------------------------------------------

    #[allow(dead_code)]
    pub(crate) fn get_not_empty(&self) -> Self {
        let mut channels = Vec::new();
        for channel in self.channels.iter().cloned() {
            let num_marked = channel
                .videos
                .clone()
                .into_iter()
                .filter(|video| !video.marked())
                .count();

            if num_marked != 0 {
                channels.push(channel);
            }
        }
        Self {
            channels,
            ..Self::default()
        }
    }
}

impl PartialEq<ChannelList> for ChannelList {
    fn eq(&self, other: &ChannelList) -> bool {
        self.channels == other.channels
    }
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use crate::url_file::*;
 *
 *     impl ChannelList {
 *         pub fn test(channels: Vec<Channel>) -> ChannelList {
 *             let backup = Vec::new();
 *             let list_state = ListState::default();
 *
 *             ChannelList {
 *                 backup,
 *                 list_state,
 *                 channels,
 *             }
 *         }
 *     }
 *
 *     #[test]
 *     fn test_update_from_url() {
 *         let channels = vec![
 *             Channel::test("channel_1".into(), "tag_1".into(), "channel_1".into()),
 *             Channel::test("channel_2".into(), "tag_2".into(), "channel_2".into()),
 *         ];
 *
 *         let url_channels = vec![UrlFileCustomChannel::test(
 *             "channel_2".into(),
 *             "tag_2".into(),
 *             vec!["url_2".into()],
 *         )];
 *
 *         let url_file = UrlFile::test(url_channels);
 *         let mut channel_list = ChannelList::test(channels);
 *
 *         println!("{:#?}", url_file);
 *         println!("{:#?}", channel_list);
 *         assert_eq!(channel_list.get(0).unwrap().id(), &String::from("channel_1"));
 *
 *         channel_list.remove_old(&url_file);
 *
 *         println!("{:#?}", channel_list);
 *         assert_eq!(channel_list.get(0).unwrap().id(), &String::from("channel_2"));
 *     }
 * } */
