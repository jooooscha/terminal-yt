use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use quick_xml::de::from_str;
use std::{
    fs::File,
    io::prelude::*,
    sync::{
        mpsc::Sender,
        mpsc::channel,
    },
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
use crate::history::{
    read_history,
};
use notification::notify::notify_user;

const URLS_FILE_PATH: &str = ".config/tyt/urls";

#[derive(Clone)]
enum FeedType {
    Atom(String),
    Rss(String),
}

#[derive(Serialize, Deserialize)]
struct UrlFile {
    atom: Vec<String>,
    rss: Vec<String>,
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

#[allow(dead_code)]
pub fn fetch_new_videos(sender: Sender<String>) -> ChannelList {
    let client = Client::builder().build().ok().unwrap();

    let urls = read_urls_file();

    let history: ChannelList = match read_history() {
        Some(content) => content,
        None => ChannelList::new(),
    };

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

            let response = match client.get(&url).send() {
                Ok(r) => match r.text() {
                    Ok(e) => Some(e),
                    Err(_) => None,
                },
                Err(_) => {
                    /* notify_user(format!("could not GET url: {}", e)); */
                    None
                }
            };

            let fetched_channel: Option<Channel>;

            if let Some(body) = response {
                match item {
                    FeedType::Atom(_) => {
                        let atom_feed: atom::Feed = match from_str(&body) {
                            Ok(feed) => feed,
                            Err(e) => {
                                notify_user(&format!("could not paarse atom feed: {}", e));
                                tx.send(None).unwrap();
                                return
                            }
                        };
                        fetched_channel = Some(atom_feed.to_internal_channel());
                    },
                    FeedType::Rss(_) => {
                        let rss_feed: rss::Feed = match from_str(&body) {
                            Ok(feed) => feed,
                            Err(e) => {
                                notify_user(&format!("could not parse rss feed: {}", e));
                                tx.send(None).unwrap();
                                return
                            }
                        };
                        fetched_channel = Some(rss_feed.to_internal_channel());
                    }
                }
            } else {
                fetched_channel = None;
            }

            let mut channel = Channel::new_with_url(&url);

            match fetched_channel {
                Some(content) => {
                    channel.name = content.name;
                    /* channel.link = content.link; */

                    for h in history.channels.iter() {
                        // match channel links
                        if h.link == channel.link && h.name == channel.name {
                            // copy old video elements
                            channel.videos = h.videos.clone();

                            break
                        }
                    }

                    for vid in content.videos.into_iter() {
                        if !channel.videos.iter().any(|video| video.link == vid.link) {
                            channel.videos.push(vid);
                        }
                    }
                },
                None => {
                    let mut found = false;
                    for h in history.channels.iter() {
                        // match channel links
                        if h.link == channel.link {
                            // copy old video elements

                            channel.name = h.name.clone();
                            /* channel.link = h.link.clone(); */
                            channel.videos = h.videos.clone();

                            found = true;
                            break
                        }
                    }
                    if !found {
                        tx.send(None).unwrap();
                        return
                    }
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

    channel_list.sort();
    channel_list.list_state.select(Some(0));

    channel_list
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
