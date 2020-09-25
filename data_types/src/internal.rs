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
/* use fetch_data::write_history; */


// program structs
pub trait ToSpans {
    fn to_spans(&mut self) -> Spans;
}

//----------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelList {
    pub channels: Vec<Channel>,

    #[serde(skip)]
    pub list_state: ListState,
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

//----------------------------------

impl ChannelList {
    #[allow(dead_code)]
    pub fn new() -> ChannelList {
        ChannelList {
            channels: Vec::new(),
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
}

//----------------------------------

impl Channel {
    #[allow(dead_code)]
    pub fn new() -> Channel {
        Channel {
            name: String::from("New Channel"),
            link: String::new(),
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
    pub fn new() -> Video {
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
        Command::new("notify-send").arg("Open video").arg(&self.title).spawn().expect("failed");
        Command::new("setsid").arg("-f").arg("umpv").arg(link).spawn().expect("umpv stating failed");
    }
}

impl ToSpans for Video {
    fn to_spans(&mut self) -> Spans {
        let d = DateTime::parse_from_rfc3339(&self.pub_date).unwrap();

        let date = format!("{:>4} - ", &d.format("%d.%m"));
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
