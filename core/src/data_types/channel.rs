use crate::{
    data_types::{feed_types::*, video::Video},
    url_file::UrlFileItem,
    ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{ListItem, ListState},
};

//----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub id: String,
    pub(crate) videos: Vec<Video>,

    #[serde(skip)]
    pub tag: String,
    #[serde(skip)]
    list_state: ListState,
}

//----------------------------------

impl Channel {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Channel {
            name: String::from("placeholder_name"),
            id: String::from("placeholder_id"),
            videos: Vec::new(),
            list_state: ListState::default(),
            tag: String::from("placeholder_tag"),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_id(id: String) -> Channel {
        Channel {
            id: id,
            ..Self::new()
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
        self.videos
            .iter()
            .map(|e| e.to_list_item())
            .collect::<Vec<ListItem>>()
            .clone()
    }
}

impl From<rss::Feed> for Channel {
    fn from(rss_feed: rss::Feed) -> Channel {
        let feed = rss_feed.channel;

        let name = feed.name;
        let id = feed.link;
        let videos = feed
            .videos
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
        let videos = feed
            .videos
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

impl ToTuiListItem for Channel {
    fn to_list_item(&self) -> ListItem {
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
