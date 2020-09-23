use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use quick_xml::de::from_str;
use std::{
    fs::File,
    /* io::BufReader, */
    io::prelude::*,
    sync::{
        mpsc::Sender,
        mpsc::channel,
    },
};
use threadpool::ThreadPool;
use dirs::home_dir;

use super::structs::*;
use crate::atom;
use crate::rss;

#[derive(Deserialize, Serialize)]
struct UrlFile {
    atom: Vec<String>,
    rss: Vec<String>,
}

#[derive(Clone)]
enum FeedType {
    Atom(String),
    Rss(String),
}

impl UrlFile {
    fn len(&self) -> usize {
        self.atom.len() + self.rss.len()
    }
    fn get_mixed(&self) -> Vec<FeedType> {
        let mut arr: Vec<FeedType> = Vec::new();
        for url in self.atom.iter() {
            arr.push(FeedType::Atom(url.clone()));
        }
        for url in self.rss.iter() {
            arr.push(FeedType::Rss(url.clone()));
        }
        arr
    }

}

const HISTORY_FILE_PATH: &str = ".config/tyt/history.json";
const URLS_FILE_PATH: &str = ".config/tyt/urls";

//-------------------------------------
pub fn fetch_new_videos(sender: Sender<String>) -> ChannelList {
    let client = Client::builder().build().ok().unwrap();

    let urls = read_urls_file();

    let history: ChannelList = read_history();
    let mut channel_list = ChannelList::new(Vec::new());

    let worker_num = 4;
    let jobs_num = urls.len();
    let pool = ThreadPool::new(worker_num);

    let (tx, rx) = channel();

    for item in urls.get_mixed() {
        let url = match item.clone() {
            FeedType::Atom(s) => s,
            FeedType::Rss(s) => s,
        };

        let tx = tx.clone();
        let sender = sender.clone();
        let history = history.clone();
        let client = client.clone();
        pool.execute(move || {
            sender.send(format!("fetching... {}", url.clone())).unwrap();

            let body = match client.get(&url).send().ok() {
                Some(e) => e.text().ok().unwrap(),
                None => return
            };

            let mut channel = Channel::new();
            let temp_videos;

            match item {
                FeedType::Atom(_) => {
                    let feed: atom::Feed = from_str(&body).unwrap();
                    channel.name = feed.title.clone();
                    channel.link = feed.link.clone();
                    temp_videos = feed.get_videos();
                },
                FeedType::Rss(_) => {
                    let rss: rss::Feed = from_str(&body).unwrap();
                    channel.name = rss.channel.title;
                    channel.link = rss.channel.link;
                    temp_videos = rss.channel.videos;
                }
            }

            for h in history.channels.iter() {
                // match channel links
                if h.link == channel.link {
                    // copy old video elements
                    channel.videos = h.videos.clone();

                    break
                }
            }

            // insert videos from feed, if not already in list
            for vid in temp_videos.into_iter() {
                if !channel.videos.iter().any(|video_item| video_item.video == vid) {
                    channel.videos.push(
                        VideoItem::new(vid)
                    );
                }
            }

            channel.videos.sort_by_key(|v| v.video.time.clone());
            channel.videos.reverse();

            tx.send(channel).unwrap();
            /* channel_list.channels.push(channel); */
        });
    }
    for chan in rx.iter().take(jobs_num) {
        channel_list.channels.push(chan);
    }

    channel_list.channels.sort_by_key(|c| c.name.clone());

    channel_list
}

pub fn fetch_history_videos () -> ChannelList {
    read_history()
}


pub fn write_history(channel_list: &ChannelList) {
    let list = channel_list.channels.clone();
    let json = serde_json::to_string(&list).unwrap();

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
            let channels: Vec<Channel> = serde_json::from_str(&reader).unwrap();
            // return
            ChannelList::new(channels)
        }
        Err(_) => {
            // write empty history
            write_history(&ChannelList::new(Vec::new()));
            // try again
            read_history()
        }
    }
}

fn read_urls_file() -> UrlFile {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let urls: UrlFile = toml::from_str(&reader).unwrap();

            urls
        }
        Err(e) => {
            panic!("somthig is wrong with the url file: {}", e);
        }
    }
}
