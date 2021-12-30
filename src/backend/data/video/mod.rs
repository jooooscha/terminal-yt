pub(in super) mod builder;

use crate::backend::ToTuiListItem;
use std::cmp::Ordering;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Default)]
pub struct Video {
    pub(super) title: String,
    pub(super) link: String,
    pub(super) origin_url: String,
    pub(super) origin_channel_name: String,
    pub(super) marked: bool,
    #[serde(default)]
    pub(super) fav: bool,

    #[serde(rename = "pubDate")]
    pub(super) pub_date: String,

    #[serde(skip)]
    pub(super) is_new: bool,
}

impl Video {
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }

    pub fn is_fav(&self) -> bool {
        self.fav
    }

    pub fn set_fav(&mut self, is_fav: bool) {
        self.fav = is_fav;
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

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn get_details(&self) -> String {
        self.title.to_string()
    }
}

impl Ord for Video {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut i = 0; // self
        let mut j = 0; // other

        // fav has highest prio
        if !self.is_fav()  { i += 100; }
        if !other.is_fav() { j += 100; }

        // marked has less prio
        if self.marked     { i +=  10; }
        if other.marked    { j +=  10; }

        i.cmp(&j)
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

        let new = if self.is_fav() {
            " ⭐ ".to_string()
        } else if self.is_new {
            " * ".to_string()
        } else {
            String::from(" ")
        };
        let title = self.title.to_string();
        let date = match DateTime::parse_from_rfc3339(&self.pub_date) {
            Ok(date_) => format!("{:>4}", &date_.format("%d.%m.%y")),
            Err(_) => String::new(),
        };

        let spacer = String::from(" - ");

        let yellow = Style::default().fg(Color::Yellow);
        let gray = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);

        if self.marked {
            ListItem::new(Spans::from(vec![
                Span::styled(new, gray),
                Span::styled(title, gray),
                Span::styled(spacer, gray),
                Span::styled(date, gray),
            ]))
        } else {
            ListItem::new(Spans::from(vec![
                Span::styled(new, yellow),
                Span::styled(title, yellow),
                Span::styled(spacer, gray),
                Span::styled(date, gray),
            ]))
        }
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
