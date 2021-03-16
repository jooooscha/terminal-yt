use crate::{
    url_file::{read_urls_file, UrlFile, UrlFileItem},
    data_types::{video::Video, channel::Channel},
    Filter::{self, *},
    ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use tui::{
    widgets::{ListItem, ListState},
};


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

    pub fn get_by_id(&self, id: &String) -> Option<&Channel> {
        let p = self.get_position_by_id(id)?;
        self.channels.get(p)
    }

    pub fn get_mut_by_id(&mut self, id: &String) -> Option<&mut Channel> {
        let p = self.get_position_by_id(id)?;
        self.channels.get_mut(p)
    }

    pub fn get_position_by_id(&self, id: &String) -> Option<usize> {
        self.channels.iter().position(|channel| &channel.id == id)
    }

    pub fn get_spans_list(&self) -> Vec<ListItem> {
        self.channels
            .iter()
            .map(|channel| channel.to_list_item())
            .collect()
    }

    /// Filter all channels that are not in the UrlFile anymore
    fn remove_old(&mut self, url_file_content: &UrlFile) {
        self.channels = self.channels
            .iter()
            .cloned()
            .filter(|channel| {
                // test if in normal channels
                url_file_content.channels.iter().any(|url_channel| url_channel.id() == channel.id)
                    // test if in custom channels
                    || url_file_content.custom_channels.iter().any(|url_channel| url_channel.id() == channel.id)
            })
            .collect();

        // remove videos that belong to urls removed from a custom channel
        for custom_channel in url_file_content.custom_channels.iter() {
            let urls = &custom_channel.urls;
            if let Some(mut channel) = self.get_mut_by_id(&custom_channel.id()) {
                channel.videos = channel
                    .videos
                    .iter()
                    .filter(|video| urls.iter().any(|url| url == &video.origin_url))
                    .cloned()
                    .collect();
            }
        }
    }

    fn update_name_and_tag(&mut self, url_file_content: &UrlFile) {
        for item in url_file_content.channels.iter() {
            if let Some(mut chan) = self.get_mut_by_id(&item.id()) {
                chan.tag = item.tag().clone();
                if !item.name().is_empty() {
                    chan.name = item.name().clone();
                }
            }
        }

        for item in url_file_content.custom_channels.iter() {
            if let Some(mut chan) = self.get_mut_by_id(&item.id()) {
                chan.tag = item.tag().clone();
                if !item.name().is_empty() {
                    chan.name = item.name().clone();
                }
            }
        }
    }

    pub fn apply_url_file_changes(&mut self) {
        let url_file_content = read_urls_file();

        self.remove_old(&url_file_content);
        self.update_name_and_tag(&url_file_content);
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
                .filter(|video| !video.marked)
                .collect::<Vec<Video>>()
                .len();
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
            if !self.backup.iter().any(|c| c.id == chan.id) {
                self.backup.push(chan.clone());
            }
        }

        // sort
        if sort_by_tag {
            self.backup.sort_by_key(|channel|
                if channel.tag.is_empty() {
                    channel.name.clone().to_lowercase() // lowercase is sorted after uppercase
                /* if channel.has_new() {
                 *     channel.name.clone().to_lowercase() // lowercase is sorted after uppercase */
                } else {
                    format!("{}{}", channel.tag.clone().to_uppercase(), channel.name.clone().to_uppercase())
                }
            );
        } else {
            self.backup
                .sort_by_key(|channel| channel.name.clone().to_lowercase());
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
                    .filter(|c| c.videos.iter().any(|v| !v.marked))
                    .collect();
            }
        }
    }
}
