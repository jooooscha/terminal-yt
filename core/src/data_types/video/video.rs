use crate::ToTuiListItem;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub title: String,
    pub link: String,
    pub origin_url: String,
    pub origin_channel_name: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub marked: bool,

    #[serde(skip)]
    pub new: bool,
}

impl Video {
    #[allow(dead_code)]
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }

    pub fn add_origin(&mut self, url: &String, channel_name: &String) {
        self.origin_url = url.to_string();
        self.origin_channel_name = channel_name.to_string();
    }

    pub(super) fn new() -> Self {
        Video {
            title: String::new(),
            link: String::new(),
            origin_url: String::new(),
            origin_channel_name: String::new(),
            pub_date: String::new(),
            marked: false,
            new: true,
        }
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
    use crate::url_file::Date;

    impl Video {
        pub fn test(
            name: String,
            link: String,
            marked: bool,
            new: bool,
            origin_url: String,
            pub_date: Date,
            title: String,
        ) -> Self {
        }
    }
}
