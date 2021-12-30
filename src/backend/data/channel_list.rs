use crate::backend::{
    data::channel::Channel,
    url_file::{read_urls_file, UrlFile, UrlFileItem},
    Filter::{self, *},
    ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use tui::widgets::{ListItem, ListState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelList {
    channels: Vec<Channel>,
    #[serde(skip)]
    list_state: ListState,
    #[serde(skip)]
    backup: Vec<Channel>,
}

//----------------------------------

impl ChannelList {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ChannelList {
            channels: Vec::new(),
            backup: Vec::new(),
            list_state: ListState::default(),
        }
    }

    #[allow(dead_code)]
    pub fn next(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => {
                if i + 1 < self.channels.len() {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    #[allow(dead_code)]
    pub fn prev(&mut self) {
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

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn select(&mut self, i: Option<usize>) {
        self.list_state.select(i);
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn state(&self) -> ListState {
        self.list_state.clone()
    }

    pub fn push(&mut self, channel: Channel) {
        self.channels.push(channel);
    }

    pub fn get(&self, index: usize) -> Option<&Channel> {
        self.channels.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Channel> {
        self.channels.get_mut(index)
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Channel> {
        let p = self.get_position_by_id(id)?;
        self.channels.get(p)
    }

    pub fn get_mut_by_id(&mut self, id: &str) -> Option<&mut Channel> {
        let p = self.get_position_by_id(id)?;
        self.channels.get_mut(p)
    }

    pub fn get_position_by_id(&self, id: &str) -> Option<usize> {
        self.channels.iter().position(|channel| channel.id() == id)
    }

    pub fn get_spans_list(&self) -> Vec<ListItem> {
        self.channels
            .iter()
            .map(|channel| channel.to_list_item())
            .collect()
    }

    /// Filter all channels that are not in the UrlFile anymore
    fn remove_old(&mut self, url_file: &UrlFile) {
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

    fn update_channels_from_url_file(&mut self, url_file_content: &UrlFile) {
        // update all "normal" channels
        for item in url_file_content.channels.iter() {
            if let Some(ref mut chan) = self.get_mut_by_id(&item.id()) {
                chan.update_from_url_file(item as &dyn UrlFileItem);
            }
        }

        // update all custom channels
        for item in url_file_content.custom_channels.iter() {
            if let Some(ref mut chan) = self.get_mut_by_id(&item.id()) {
                chan.update_from_url_file(item as &dyn UrlFileItem);
            }
        }
    }

    pub fn apply_url_file_changes(&mut self) {
        let url_file_content = read_urls_file();

        self.remove_old(&url_file_content);
        self.update_channels_from_url_file(&url_file_content);
    }

    //---------------------------------------------------------------

    #[allow(dead_code)]
    pub fn get_not_empty(&self) -> ChannelList {
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
        ChannelList {
            channels,
            ..ChannelList::new()
        }
    }

    pub fn filter(&mut self, filter: Filter, sort_by_tag: bool) {
        // merge changes to backup
        let tmp = self.backup.clone();
        self.backup = self.channels.clone();
        for chan in tmp.iter() {
            if !self.backup.iter().any(|c| c.id() == chan.id()) {
                self.backup.push(chan.clone());
            }
        }

        // sort
        if sort_by_tag {
            self.backup.sort_by_key(|channel|
                if channel.tag().is_empty() {
                    channel.name().clone().to_lowercase() // lowercase is sorted after uppercase
                /* if channel.has_new() {
                 *     channel.name.clone().to_lowercase() // lowercase is sorted after uppercase */
                } else {
                    format!("{}{}", channel.tag().clone().to_uppercase(), channel.name().clone().to_uppercase())
                }
            );
        } else {
            self.backup
                .sort_by_key(|channel| channel.name().clone().to_lowercase());
        }

        // aply new changes
        match filter {
            NoFilter => {
                self.channels = self.backup.clone();
            }
            OnlyNew => {
                self.channels = self
                    .backup
                    .iter()
                    .cloned()
                    .filter(|c| c.videos.iter().any(|v| !v.marked()))
                    .collect();
            }
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
