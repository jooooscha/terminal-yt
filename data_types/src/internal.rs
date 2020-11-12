use serde::{Deserialize, Serialize};
use std::{
    process::{
        Command,
        Stdio,
    },
};
use tui::{
    widgets::ListState,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};
use chrono::DateTime;
/* use fetch_data::write_history; */

use Filter::*;


#[derive(PartialEq, Clone, Copy)]
pub enum Filter {
    NoFilter,
    OnlyNew,
}

// program structs
pub trait ToSpans {
    fn to_spans(&mut self) -> Spans;
}

//----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelList {
    pub channels: Vec<Channel>,

    #[serde(skip)]
    pub list_state: ListState,
    #[serde(skip)]
    backup: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub link: String,
    pub videos: Vec<Video>,

    #[serde(skip)]
    pub list_state: ListState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Video {
    pub title: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub marked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinimalVideo {
    pub title: String,
    pub pub_date: String,
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
            Some(i) => if i < self.channels.len() -1 {
                i + 1
            } else {
                i
            },
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    #[allow(dead_code)]
    pub fn prev(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => if i > 0 {
                i - 1
            } else {
                i
            },
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    #[allow(dead_code)]
    pub fn get_not_empty(&self) -> ChannelList {
        let mut channels = Vec::new();
        for channel in self.channels.iter().cloned() {
            let num_marked = channel.videos.clone().into_iter().filter(|video| !video.marked).collect::<Vec<Video>>().len();
            if num_marked != 0 {
                channels.push(channel);
            }
        }
        ChannelList {
            channels,
            ..ChannelList::new()
        }
    }

    pub fn sort(&mut self) {
        self.channels.sort_by_key(|c| c.name.clone());
    }

    pub fn filter(&mut self, filter: Filter) {
        // merge changes to backup
        let tmp = self.backup.clone();
        self.backup = self.channels.clone();
        for chan in tmp.iter() {
            if !self.backup.iter().any(|c| c.link == chan.link) {
                self.backup.push(chan.clone());
            }
        }

        self.backup.sort_by_key(|c| c.name.clone());

        // aply new changes
        match filter {
            NoFilter => {
                self.channels = self.backup.clone();
            }
            OnlyNew => {
                self.channels = self.backup.iter().cloned().filter(|c| c.videos.iter().any(|v| !v.marked)).collect();
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
            link: String::new(),
            videos: Vec::new(),
            list_state: ListState::default(),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_url(url: &String) -> Channel {
        Channel {
            name: String::from("New Channel"),
            link: url.clone(),
            videos: Vec::new(),
            list_state: ListState::default(),
        }
    }

    #[allow(dead_code)]
    pub fn next(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => if i < self.videos.len() -1 {
                i + 1
            } else {
                i
            },
            None => 0,
        };
        self.list_state.select(Some(index));
    }

    #[allow(dead_code)]
    pub fn prev(&mut self) {
        let state = &self.list_state;
        let index = match state.selected() {
            Some(i) => if i > 0 {
                i - 1
            } else {
                i
            },
            None => 0,
        };
        self.list_state.select(Some(index));
    }
    #[allow(dead_code)]
    pub fn has_new(&self) -> bool {
        self.videos.iter().any(|v| !v.marked)
    }
}

impl ToSpans for Channel {
    fn to_spans(&mut self) -> Spans {
        let num_marked = &self.videos.clone().into_iter().filter(|video| !video.marked).collect::<Vec<Video>>().len();
        let num = format!("{:>3}/{:<4}|  ", num_marked, &self.videos.len());
        let name = format!("{}", &self.name);
        let style = match num_marked {
            0 => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::Yellow)
        };
        Spans::from(vec![
                Span::styled(num, style),
                Span::styled(name, style.add_modifier(Modifier::ITALIC))
        ])
    }
}

//------------------------------------

impl Video {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Video {
            title: String::from("VideoTitle"),
            link: String::from("video_link"),
            pub_date: String::from("DATUM"),
            marked: false,
        }
    }

    #[allow(dead_code)]
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }

    #[allow(dead_code)]
    pub fn open(&self) {
        // open with mpv
        let link = &self.link;
        Command::new("notify-send").arg("Open video").arg(&self.title).stderr(Stdio::null()).spawn().expect("failed");
        Command::new("setsid").arg("-f").arg("umpv").arg(link).stderr(Stdio::null()).spawn().expect("umpv stating failed");
    }
    #[allow(dead_code)]
    pub fn to_minimal(&self) -> MinimalVideo {
        MinimalVideo {
            title: self.title.clone(),
            pub_date: self.pub_date.clone(),
        }
    }

}

impl ToSpans for Video {
    fn to_spans(&mut self) -> Spans {
        /* let d = match DateTime::parse_from_rfc3339(&self.pub_date); */
        let date = if let Ok(date_) = DateTime::parse_from_rfc3339(&self.pub_date) {
            format!("{:>4} - ", &date_.format("%d.%m.%y"))
        } else {
            String::from("NODATE - ")
        };

        let title = format!("{}", &self.title);

        let style = match self.marked {
            true => Style::default().fg(Color::DarkGray),
            false => Style::default().fg(Color::Yellow),
        };
        Spans::from(vec![
            Span::styled(date, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC))
        ])
    }
}
impl ToSpans for MinimalVideo {
    fn to_spans(&mut self) -> Spans {
        /* let d = match DateTime::parse_from_rfc3339(&self.pub_date); */
        let date = if let Ok(date_) = DateTime::parse_from_rfc3339(&self.pub_date) {
            format!("{:>4} - ", &date_.format("%d.%m.%y"))
        } else {
            String::from("NODATE - ")
        };

        let title = format!("{}", &self.title);

        let style = Style::default().fg(Color::DarkGray);

        Spans::from(vec![
            Span::styled(date, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC))
        ])
    }
}
