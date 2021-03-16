use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::{
    process::{Command, Stdio},
};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{ListState, ListItem},
};
use Filter::*;
use crate::url_file::{read_urls_file, UrlFileItem, UrlFile};
use crate::data_types::feed_types::*;

#[derive(PartialEq, Clone, Copy)]
pub enum Filter {
    NoFilter,
    OnlyNew,
}

// program structs
pub trait ToSpans {
    fn to_spans(&self) -> ListItem;
}

//----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelList {
    channels: Vec<Channel>,
    #[serde(skip)]
    list_state: ListState,
    #[serde(skip)]
    backup: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub id: String,
    // #[serde(default = "empty_string")]
    videos: Vec<Video>,

    #[serde(skip)]
    pub tag: String,
    #[serde(skip)]
    list_state: ListState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub title: String,
    pub link: String,
    pub origin_url: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub marked: bool,

    #[serde(skip)]
    pub new: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinimalVideo {
    pub title: String,
    pub channel: String,
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
        self.channels.iter().map(|channel| channel.to_spans()).collect()
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
                channel.videos = channel.videos.iter().filter(
                    |video| urls.iter().any(
                        |url| url == &video.origin_url
                    )
                ).cloned().collect();
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

//----------------------------------

impl Channel {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Channel {
            name: String::from("New Channel"),
            id: String::from("placeholder_id"),
            videos: Vec::new(),
            list_state: ListState::default(),
            tag: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_id(id: String) -> Channel {
        Channel {
            name: String::from("New Channel"),
            id: id,
            videos: Vec::new(),
            list_state: ListState::default(),
            tag: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn next(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => {
                if i + 1 < self.videos.len() {
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
    #[allow(dead_code)]
    pub fn has_new(&self) -> bool {
        self.videos.iter().any(|v| !v.marked)
    }

    pub fn add_origin_url(&mut self, url: &String) {
        for video in self.videos.iter_mut() {
            video.add_origin_url(url);
        }
    }

    pub fn update_information(&mut self, url_file_channel: &dyn UrlFileItem) {
        // set name - prefere name declard in url-file
        if !url_file_channel.name().is_empty() {
            self.name = url_file_channel.name().clone();
        }

        // set tag
        /* println!("{},{}", self.tag, url_file_channel.tag().clone()); */
        self.tag = url_file_channel.tag().clone();
    }

    //-------------------------------------------------

    pub fn len(&self) -> usize {
        self.videos.len()
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

    pub fn push(&mut self, video: Video) {
        self.videos.push(video);
    }

    pub fn get(&self, index: usize) -> Option<&Video> {
        self.videos.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Video> {
        self.videos.get_mut(index)
    }

    pub fn append(&mut self, videos: &mut Vec<Video>) {
        self.videos.append(videos);
    }

    pub fn merge_videos(&mut self, other: Channel) {
        for video in other.videos.into_iter() {
            if !self.contains(&video) {
                self.push(video);
            }
        }
        self.sort();
    }

    pub fn contains(&self, video: &Video) -> bool {
        self.videos.contains(video)
    }

    pub fn push_if_not_contains(&mut self, channel: Video) {
        if self.videos.contains(&channel) {
            self.videos.push(channel);
        }
    }

    pub fn sort(&mut self) {
        self.videos.sort_by_key(|video| video.pub_date.clone());
        self.videos.reverse();
    }

    pub fn get_spans_list(&self) -> Vec<ListItem> {
        self.videos.iter().map(|e| e.to_spans()).collect::<Vec<ListItem>>().clone()
    }
}

impl From<rss::Feed> for Channel {
    fn from(rss_feed: rss::Feed) -> Channel {
        let feed = rss_feed.channel;

        let name = feed.name;
        let id = feed.link;
        let videos = feed.videos
            .into_iter()
            .map(|rss_vid| Video::from(rss_vid))
            .collect();

        Channel {
            name,
            id,
            videos,
            ..Channel::new()
        }
    }
}

impl From<atom::Feed> for Channel {
    fn from(feed: atom::Feed) -> Channel {

        let name = feed.name;
        let id = format!("htttps://www.youtube.com/channel/{}", feed.channel_id);
        let videos = feed.videos
            .into_iter()
            .map(|atom_vid| Video::from(atom_vid))
            .collect();

        Channel {
            name,
            id,
            videos,
            ..Channel::new()
        }
    }
}

impl ToSpans for Channel {
    fn to_spans(&self) -> ListItem {
        let num_marked = &self
            .videos
            .clone()
            .into_iter()
            .filter(|video| !video.marked)
            .collect::<Vec<Video>>()
            .len();
        let has_new = self.videos.iter().any(|video| video.new);

        let num = format!("{:>3}/{:<4}", num_marked, &self.videos.len());
        let bar = String::from(" | ");
        let new = if has_new {
            format!(" new")
        } else {
            String::new()
        };
        let name = format!("{}", &self.name);
        let tag = if self.tag.is_empty() {
            String::from("")
        } else {
            format!("[{}] ", &self.tag)
        };

        let base_style;
        let tag_style;
        let new_style;

        if num_marked > &0 {
            base_style = Style::default().fg(Color::Yellow);
            tag_style = Style::default().fg(Color::Blue);
            new_style = Style::default().fg(Color::LightGreen);
        } else {
            base_style = Style::default().fg(Color::DarkGray);
            new_style = base_style.clone();
            tag_style = base_style.clone();
        }

        ListItem::new(Spans::from(vec![
            Span::styled(num, base_style),
            Span::styled(bar, base_style),
            Span::styled(tag, tag_style),
            Span::styled(name, base_style.add_modifier(Modifier::ITALIC)),
            Span::styled(new, new_style),
        ]))
    }
}

//------------------------------------

impl Video {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Video {
            title: String::from("VideoTitle"),
            link: String::from("video_link"),
            origin_url: String::from("origin_url"),
            pub_date: String::from("DATUM"),
            marked: false,
            new: true,
        }
    }

    #[allow(dead_code)]
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }

    #[allow(dead_code)]
    pub fn open(&self) -> Result<(), String> {
        // open with mpv
        if let Err(err) = Command::new("setsid")
            .arg("-f")
            .arg("umpv")
            .arg(&self.link)
            .stderr(Stdio::null())
            .spawn()
        {
            return Err(format!("Could not start umpv: {}", err));
        };

        Ok(())
    }
    #[allow(dead_code)]
    pub fn to_minimal(&self, channel: String) -> MinimalVideo {
        MinimalVideo {
            title: self.title.clone(),
            channel,
        }
    }

    pub fn add_origin_url(&mut self, url: &String) {
        self.origin_url = url.to_string();
    }
}

impl PartialEq<Video> for Video {
    fn eq(&self, other: &Video) -> bool {
        self.link == other.link
    }
}

impl From<rss::Video> for Video {
    fn from(rss_video: rss::Video) -> Video {

        let title = rss_video.title;
        let link = rss_video.link;
        let pub_date = rss_video.pub_date;

        Video {
            title,
            link,
            pub_date,
            ..Video::new()
        }
    }
}

impl From<atom::Video> for Video {
    fn from(atom_vid: atom::Video) -> Video {

        let title = atom_vid.title;
        let link = format!("https://www.youtube.com/watch?v={}", atom_vid.id);
        let pub_date = atom_vid.pub_date;

        Video {
            title,
            link,
            pub_date,
            ..Video::new()
        }
    }
}


impl ToSpans for Video {
    fn to_spans(&self) -> ListItem {
        /* let d = match DateTime::parse_from_rfc3339(&self.pub_date); */
        let pre_title = if self.new && !self.marked {
            String::from("   new   - ")
        } else {
            if let Ok(date_) = DateTime::parse_from_rfc3339(&self.pub_date) {
                format!("{:>4} - ", &date_.format("%d.%m.%y"))
            } else {
                String::from(" NODATE  - ")
            }
        };

        let title = format!("{}", &self.title);

        let style_title;
        let style_new;

        if self.marked {
            style_title = Style::default().fg(Color::DarkGray);
            style_new = style_title.clone();
        } else if self.new {
            style_title = Style::default().fg(Color::Yellow);
            style_new = Style::default().fg(Color::LightGreen);
        } else {
            style_title = Style::default().fg(Color::Yellow);
            style_new = style_title.clone();
        }

        ListItem::new(Spans::from(vec![
            Span::styled(pre_title, style_new),
            Span::styled(title, style_title.add_modifier(Modifier::ITALIC)),
        ]))
    }
}
impl ToSpans for MinimalVideo {
    fn to_spans(&self) -> ListItem {
        let channel = format!("{} {} - ", tui::symbols::DOT, &self.channel);
        let title = format!("{}", &self.title);

        let style = Style::default().fg(Color::DarkGray);

        ListItem::new(Spans::from(vec![
            Span::styled(channel, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC)),
        ]))
    }
}
