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
    process::Command,
};
use threadpool::ThreadPool;
use dirs::home_dir;

use data_types::{
    internal::{
        ChannelList,
        Channel,
    },
    rss,
    atom,
};

#[derive(Serialize, Deserialize)]
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
#[allow(dead_code)]
pub fn fetch_new_videos(sender: Sender<String>) -> ChannelList {
    let client = Client::builder().build().ok().unwrap();

    let urls = read_urls_file();

    let history: ChannelList = read_history();
    let mut channel_list = ChannelList::new();

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

            let body = match client.get(&url).send() {
                Ok(response) => match response.text() {
                    Ok(e) => e,
                    Err(_) => {
                        tx.send(None).unwrap();
                        return
                    },
                },
                Err(e) => {
                    notify_user(format!("could not GET url: {}", e));
                    tx.send(None).unwrap();
                    return
                }
            };

            let fetched_channel: Channel;

            match item {
                FeedType::Atom(_) => {
                    let atom_feed: atom::Feed = match from_str(&body) {
                        Ok(feed) => feed,
                        Err(e) => {
                            notify_user(format!("could not paarse atom feed: {}", e));
                            tx.send(None).unwrap();
                            return
                        }
                    };
                    fetched_channel = atom_feed.to_internal_channel();
                },
                FeedType::Rss(_) => {
                    let rss_feed: rss::Feed = match from_str(&body) {
                        Ok(feed) => feed,
                        Err(e) => {
                            notify_user(format!("could not parse rss feed: {}", e));
                            tx.send(None).unwrap();
                            return
                        }
                    };
                    fetched_channel = rss_feed.to_internal_channel();
                }
            }

            let mut channel = Channel::new();
            channel.name = fetched_channel.name;
            channel.link = fetched_channel.link;

            for h in history.channels.iter() {
                // match channel links
                if h.link == channel.link && h.name == channel.name {
                    // copy old video elements
                    channel.videos = h.videos.clone();

                    break
                }
            }

            // insert videos from feed, if not already in list
            for vid in fetched_channel.videos.into_iter() {
                if !channel.videos.iter().any(|video| video.link == vid.link) {
                    channel.videos.push(
                        vid
                    );
                }
            }

            channel.videos.sort_by_key(|video| video.pub_date.clone());
            channel.videos.reverse();

            tx.send(Some(channel)).unwrap();
        });
    }
    for chan_opt in rx.iter().take(jobs_num) {
        match chan_opt {
            Some(chan) => channel_list.channels.push(chan),
            None => (),
        }
    }

    channel_list.channels.sort_by_key(|c| c.name.clone());
    channel_list.list_state.select(Some(0));

    channel_list
}

fn notify_user(msg: String) {
    Command::new("notify-send").arg(msg).spawn().expect("failed");
}

pub fn write_history(channel_list: &ChannelList) {
    let json = serde_json::to_string(channel_list).unwrap();

    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => panic!("history write error: {}", e),
    };
    file.write_all(json.as_bytes()).unwrap();
}

pub fn read_history() -> ChannelList {
    let mut path = home_dir().unwrap();
    path.push(HISTORY_FILE_PATH);

    match File::open(path) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let mut channel_list: ChannelList = match serde_json::from_str(&reader) {
                Ok(channels) => channels,
                Err(e) => panic!("could not read history file: {}", e),
            };

            channel_list.list_state.select(Some(0));

            // return
            channel_list
        }
        Err(_) => {
            // write empty history
            write_history(&ChannelList::new());
            // try again
            read_history()
        }
    }
}

fn read_urls_file() -> UrlFile {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let urls: UrlFile = match toml::from_str(&reader) {
                Ok(file) => file,
                Err(e) => panic!("could not parse url file: {}", e),
            };

            urls
        }
        Err(_) => {
            let mut file = File::create(path).unwrap();
            let url_file = UrlFile {
                atom: Vec::new(),
                rss: Vec::new(),
            };
            let string = toml::to_string(&url_file).unwrap();
            match file.write_all(string.as_bytes()) {
                Ok(_) => read_urls_file(),
                Err(e) => panic!("{}", e),
            }
        }
    }
}
