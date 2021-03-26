use crate::{data_types::feed_types::*, ToTuiListItem};
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
impl ToTuiListItem for MinimalVideo {
    fn to_list_item(&self) -> ListItem {
        let channel = format!("{} {} - ", tui::symbols::DOT, &self.channel);
        let title = format!("{}", &self.title);

        let style = Style::default().fg(Color::DarkGray);

        ListItem::new(Spans::from(vec![
            Span::styled(channel, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC)),
        ]))
    }
}

#[cfg(test)]
pub mod tests {

/*     impl Video {
 *         fn test(input: Vec<(String, String, )>) -> Self {
 *
 *
 *
 *         }
 *     } */

}
