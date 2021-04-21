use crate::ToTuiListItem;
use std::cmp::Ordering::{self, *};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct Video {
    pub(super) title: String,
    pub(super) link: String,
    pub(super) origin_url: String,
    pub(super) origin_channel_name: String,
    pub(super) marked: bool,

    #[serde(rename = "pubDate")]
    pub(super) pub_date: String,

    #[serde(skip)]
    pub(super) new: bool,
}

impl Video {
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn link(&self) -> &String {
        &self.link
    }

    pub fn origin_url(&self) -> &String {
        &self.origin_url
    }

    pub fn origin_channel_name(&self) -> &String {
        &self.origin_channel_name
    }

    pub fn marked(&self) -> bool {
        self.marked
    }

    pub fn pub_date(&self) -> &String {
        &self.pub_date
    }

    pub fn new(&self) -> bool {
        self.new
    }

    /* pub(super) fn add_origin(&mut self, url: &String, channel_name: &String) {
     *     self.origin_url = url.to_string();
     *     self.origin_channel_name = channel_name.to_string();
     * } */
}

impl Ord for Video {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.marked && !other.marked {
            Greater
        } else if !self.marked && other.marked {
            Less
        } else {
            Equal
        }
    }
}

impl PartialOrd for Video {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Video> for Video {
    fn eq(&self, other: &Video) -> bool {
        self.link == other.link
    }
}

impl ToTuiListItem for Video {
    fn to_list_item(&self) -> ListItem {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    /* use crate::url_file::Date; */

    impl Video {
        /* pub fn test() -> Self {
         * } */
    }
}
