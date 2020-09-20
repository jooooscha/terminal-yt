extern crate termion;

use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use quick_xml::de::from_str;
use std::{
    fs::File,
    io::BufReader,
    io::prelude::*,
    process::Command,
};
use dirs::home_dir;
use tui::{
    widgets::ListState,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

use crate::*;
use crate::draw::draw;

use chrono::DateTime;

const HISTORY_FILE_PATH: &str = ".config/tyt/history.json";
const URLS_FILE_PATH: &str = ".config/tyt/urls";

// Deserialize structs
#[derive(Debug, Deserialize)]
struct Feed {
    #[serde(rename = "entry")]
    entries: Vec<Video>,
    title: String,
    #[serde(rename = "channelId")]
    link: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Video {
    #[serde(rename = "videoId")]
    pub id: String,
    pub title: String,
    #[serde(rename = "published")]
    pub time: String,
}

// program structs
pub trait ToString {
    fn to_string(&mut self) -> Spans;
}

#[derive(Debug)]
pub struct ChannelList {
    // channels: Vec<Channel>,
    channels: Vec<Channel>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub name: String,
    link: String,
    pub videos: Vec<VideoItem>,
    pub list_state: ListState,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelSerial {
    name: String,
    link: String,
    videos: Vec<VideoItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoItem {
    video: Video,
    marked: bool,
}
//----------------------------------
impl ChannelList {
    fn new(channels: Vec<Channel>) -> ChannelList {
        ChannelList {
            channels,
            list_state: ListState::default(),
        }
    }
    pub fn show<W: Write>(&mut self, app: &mut App<W>) {
        draw(self.channels.clone(), app, &mut self.list_state);
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
    pub fn get_selected(&mut self) -> Option<&mut Channel> {
        match self.list_state.selected() {
            Some(i) => self.channels.get_mut(i),
            None => panic!("nothing selected"),
        }
    }
}
//----------------------------------
impl Channel {
    fn new() -> Channel {
        Channel {
            name: String::from("New Channel"),
            videos: Vec::new(),
            link: String::new(),
            list_state: ListState::default(),
        }
    }
    fn to_serial(&self) -> ChannelSerial {
        ChannelSerial {
            name: self.name.clone(),
            link: self.link.clone(),
            videos: self.videos.clone(),
        }
    }
    pub fn show<W: Write>(&mut self, app: &mut App<W>) {
        if self.list_state.selected() == None {
            self.list_state.select(Some(0));
        }
        draw(self.videos.clone(), app, &mut self.list_state);
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
    pub fn get_selected(&mut self) -> Option<&mut VideoItem> {
        match self.list_state.selected() {
            Some(i) => self.videos.get_mut(i),
            None => panic!("nothing selected"),
        }
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
impl ChannelSerial {
    fn to_internal(&self) -> Channel {
        Channel {
            list_state: ListState::default(),
            name: self.name.clone(),
            link: self.link.clone(),
            videos: self.videos.clone(),
        }
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
        let link = format!("https://www.youtube.com/watch?v={}", &self.video.id);
        Command::new("notify-send").arg("Open video").arg(&self.video.title).spawn().expect("failed");
        Command::new("umpv").arg(link).spawn().expect("failed");
    }
}

impl ToString for VideoItem {
    fn to_string(&mut self) -> Spans {
        let d = DateTime::parse_from_rfc3339(&self.video.time).unwrap();
        
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

//-------------------------------------
pub fn fetch_channel_list() -> ChannelList {
    let client = Client::builder().build().ok().unwrap();

    let urls = read_urls_file();

    let history: ChannelList = read_history();
    let mut channel_list = ChannelList::new(Vec::new());

    for url in urls.into_iter() {
        let body = match client.get(&url).send().ok() {
            Some(e) => e.text().ok().unwrap(),
            None => break
        };

        let feed: Feed = from_str(&body).unwrap();

        // ----------------------

        let mut channel = Channel::new();
        channel.name = feed.title;
        channel.link = feed.link;


        for h in history.channels.iter() {
            // match channel links
            if h.link == channel.link {
                // copy old video elements
                channel.videos = h.videos.clone();

                break
            }
        }
        // insert videos from feed, if not already in list
        for vid in feed.entries {
            if !channel.videos.iter().any(|video_item| video_item.video == vid) {
                channel.videos.push(
                    VideoItem::new(vid)
                );
            }
        }
        channel.videos.sort_by_key(|v| v.video.time.clone());
        channel.videos.reverse();
        channel_list.channels.push(channel);
    }

    channel_list.channels.sort_by_key(|c| c.name.clone());
    //TODO sort channel_list

    channel_list
}


fn write_history(channel_list: &ChannelList) {
    let list = channel_list.channels.clone();
    let serial: Vec<ChannelSerial> = list.into_iter().map(|channel| channel.to_serial()).collect();
    let json = serde_json::to_string(&serial).unwrap();

    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => panic!("history write error: {}", e),
    };
    file.write_all(json.as_bytes()).unwrap();
}

fn read_history() -> ChannelList {
    
    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let mut channels: Vec<ChannelSerial> = serde_json::from_str(&reader).unwrap();
            // morph into internal struct
            let list = channels.iter_mut().map(|channel_ser| channel_ser.to_internal()).collect();
            // return
            ChannelList::new(list)
        }
        Err(_) => {
            // write empty history
            write_history(&ChannelList::new(Vec::new()));
            // try again
            read_history()
        }
    }
}

fn read_urls_file() -> Vec<String> {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);
    match File::open(path) {
        Ok(file) => {
            let mut vec = Vec::new();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                vec.push(line.ok().unwrap());
            }
            vec
        }
        Err(_) => {
            Vec::new()
        }
    }
}
