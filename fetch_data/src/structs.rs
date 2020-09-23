use serde::{Deserialize, Serialize};
use std::{
    process::Command,
};
use tui::{
    widgets::ListState,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};
use chrono::DateTime;
use super::fetch_data::*;

use crate::rss::{
    Video,
};

// program structs
pub trait ToString {
    fn to_string(&mut self) -> Spans;
}

#[derive(Clone)]
pub struct ChannelList {
    pub channels: Vec<Channel>,
    pub list_state: ListState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub name: String,
    pub link: String,
    pub videos: Vec<VideoItem>,
    #[serde(skip)]
    pub list_state: ListState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoItem {
    pub video: Video,
    marked: bool,
}
//----------------------------------
impl ChannelList {
    pub fn new(channels: Vec<Channel>) -> ChannelList {
        let mut state = ListState::default();
        state.select(Some(0));
        ChannelList {
            channels,
            list_state: state,
        }
    }
    pub fn save(&self) {
        write_history(self);
    }

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
}
//----------------------------------
impl Channel {
    pub fn new() -> Channel {
        let mut state = ListState::default();
        state.select(Some(0));
        Channel {
            name: String::from("New Channel"),
            videos: Vec::new(),
            link: String::new(),
            list_state: state,
        }
    }
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
}

impl ToString for Channel {
    fn to_string(&mut self) -> Spans {
        let num_marked = &self.videos.clone().into_iter().filter(|video| !video.marked).collect::<Vec<VideoItem>>().len();
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
impl VideoItem {
    pub fn new(video: Video) -> VideoItem {
        VideoItem {
            video,
            marked: false,
        }
    }
    pub fn mark(&mut self, value: bool) {
        self.marked = value;
    }
    pub fn open(&self) {
        // open with mpv
        let link = &self.video.link;
        Command::new("notify-send").arg("Open video").arg(&self.video.title).spawn().expect("failed");
        Command::new("setsid").arg("-f").arg("umpv").arg(link).spawn().expect("umpv stating failed");
    }
}

impl ToString for VideoItem {
    fn to_string(&mut self) -> Spans {
        let d = DateTime::parse_from_rfc2822(&self.video.time).unwrap();

        let date = format!("{:>4} - ", &d.format("%d.%m"));
        let title = format!("{}", &self.video.title);

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
