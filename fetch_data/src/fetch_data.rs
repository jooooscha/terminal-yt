/* extern crate termion; */
use reqwest::blocking::Client;
use quick_xml::de::from_str;
use std::{
    fs::File,
    io::BufReader,
    io::prelude::*,
    sync::{
        mpsc::Sender,
        mpsc::channel,
    },
};
use threadpool::ThreadPool;
use dirs::home_dir;


use super::structs::*;

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
    for url in urls.into_iter() {
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
            let list = channels.iter_mut().map(|serial| Channel::from_serial(serial.clone())).collect();
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
