use crate::{data_types::video::video::Video, url_file::UrlFileItem, SortingMethod, ToTuiListItem};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{ListItem, ListState},
};
use alphanumeric_sort;

//----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub(super) name: String,
    pub(super) id: String,
    pub(crate) videos: Vec<Video>,

    #[serde(skip_deserializing)]
    pub sorting_method: SortingMethod,
    #[serde(skip)]
    pub(super) tag: String,
    #[serde(skip)]
    list_state: ListState,
}

//----------------------------------

impl Channel {
    pub(super) fn new() -> Self {
        Channel {
            name: String::from("placeholder_name"),
            id: String::from("placeholder_id"),
            videos: Vec::new(),
            list_state: ListState::default(),
            tag: String::from("placeholder_tag"),
            sorting_method: SortingMethod::Date,
        }
    }

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

    pub fn has_new(&self) -> bool {
        self.videos.iter().any(|v| !v.marked())
    }

    /* pub fn add_origin(&mut self, url: &String) {
     *     for video in self.videos.iter_mut() {
     *         video.add_origin(url, &self.name.clone());
     *     }
     * } */

    pub(crate) fn update_from_url_file(&mut self, url_file_channel: &dyn UrlFileItem) {
        // set name - prefere name declard in url-file
        if !url_file_channel.name().is_empty() {
            self.name = url_file_channel.name().clone();
        }

        // set tag
        self.tag = url_file_channel.tag().clone();

        // set sort order
        self.sorting_method = url_file_channel.sorting_method();
    }

    //-------------------------------------------------

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn tag(&self) -> &String {
        &self.tag
    }

    pub fn videos(&self) -> &Vec<Video> {
        &self.videos
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

    pub fn merge_videos(&mut self, other_videos: Vec<Video>) {
        for video in other_videos.into_iter() {
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
        match self.sorting_method {
            SortingMethod::Date => {
                self.videos.sort();
                self.videos.sort_by_key(|video| video.pub_date().clone());
                self.videos.reverse();
            },
            SortingMethod::Text => {
                self.videos.sort_by(|video_a, video_b| alphanumeric_sort::compare_str(video_a.title().clone(), video_b.title().clone()));
            },
            SortingMethod::UnseenDate => {
                self.videos.sort_by_key(|video| video.pub_date().clone());
                self.videos.reverse();
                self.videos.sort();
            },
            SortingMethod::UnseenText => {
                self.videos.sort_by(|video_a, video_b| alphanumeric_sort::compare_str(video_a.title().clone(), video_b.title().clone()));
                self.videos.sort();
            },
        }
    }

    pub fn get_spans_list(&self) -> Vec<ListItem> {
        self.videos
            .iter()
            .map(|e| e.to_list_item())
            .collect::<Vec<ListItem>>()
            .clone()
    }
}

impl PartialEq<Channel> for Channel {
    fn eq(&self, other: &Channel) -> bool {
        let eq_id = self.id == other.id;
        let eq_name = self.name == other.name;
        let eq_tag = self.tag == other.tag;
        let eq_videos = self.videos == other.videos;

        eq_videos && eq_tag && eq_id && eq_name
    }
}

impl ToTuiListItem for Channel {
    fn to_list_item(&self) -> ListItem {
        let num_marked = &self
            .videos
            .clone()
            .into_iter()
            .filter(|video| !video.marked())
            .collect::<Vec<Video>>()
            .len();
        let has_new = self.videos.iter().any(|video| video.new());

        let tag = if self.tag.is_empty() {
            String::from("")
        } else {
            format!(" [{}]", &self.tag)
        };

        let video_count = format!("{}", &self.videos.len());

        let new = if has_new {
            format!(" * ")
        } else {
            String::from(" ")
        };
        let name = format!("{}", &self.name);

        let spacer = String::from(" - ");

        if num_marked > &0 {
            let light_green = Style::default().fg(Color::LightGreen);
            let yellow = Style::default().fg(Color::Yellow);
            let blue = Style::default().fg(Color::Blue);
            let gray = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);

            ListItem::new(Spans::from(vec![
                Span::styled(new, light_green),
                Span::styled(name, yellow),
                Span::styled(tag, blue),
                Span::styled(spacer, gray),
                Span::styled(video_count, gray),
            ]))
        } else {
            let style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);

            ListItem::new(Spans::from(vec![
                Span::styled(new, style),
                Span::styled(name, style),
                Span::styled(tag, style),
                Span::styled(spacer, style),
                Span::styled(video_count, style),
            ]))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::SortingMethod;

    impl Channel {
        pub fn test(name: String, tag: String, id: String) -> Self {
            let list_state = ListState::default();

            let videos = Vec::new();

            Channel {
                name,
                tag,
                id,
                list_state,
                videos,
                sorting_method: SortingMethod::Date,
            }
        }
    }
}
