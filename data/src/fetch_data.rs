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

use chrono::prelude::*;

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

const URLS_FILE_PATH: &str = ".config/tyt/urls.yaml";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum FeedType {
    Atom,
    Rss,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Date {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
    Workday,
    Weekend,
    Always,
    Never,
}

impl Date {
    fn eq_to(&self, other: &Weekday) -> bool {
        match (self, other) {
            (Date::Mon, Weekday::Mon) |
            (Date::Tue, Weekday::Tue) |
            (Date::Wed, Weekday::Wed) |
            (Date::Thu, Weekday::Thu) |
            (Date::Fri, Weekday::Fri) |
            (Date::Sat, Weekday::Sat) |
            (Date::Sun, Weekday::Sun) |

            (Date::Workday, Weekday::Mon) |
            (Date::Workday, Weekday::Tue) |
            (Date::Workday, Weekday::Wed) |
            (Date::Workday, Weekday::Thu) |
            (Date::Workday, Weekday::Fri) |

            (Date::Weekend, Weekday::Sat) |
            (Date::Weekend, Weekday::Sun) |

            (Date::Always, _) => true,

            _ => false
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Videos {
    videos: Vec<Video>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Video {
    url: String,
    feed_type: FeedType,
    #[serde(default = "empty_string")]
    name: String,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default = "empty_string")]
    tag: String,
}

fn date_always() -> Vec<Date> {
    vec![Date::Always]
}

fn empty_string() -> String {
    String::new()
}


// impl UrlFile {
impl Videos {
    fn len(&self) -> usize {
        self.videos.len()
    }
}

#[allow(dead_code)]
pub fn fetch_new_videos(sender: Sender<String>) -> ChannelList {

    let today = Local::now().weekday();

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

    for item in urls.videos {
        /* if !item.update_on.iter().any(|w| w.eq_to(&today)) {
         *     jobs_num -= 1;
         *     continue
         * } */

        let url = item.url.clone();

        let tx = tx.clone();
        let sender = sender.clone();
        let history = history.clone();
        let client = client.clone();

        pool.execute(move || {
            let mut fetched_channel: Option<Channel> = None;

            if item.update_on.iter().any(|w| w.eq_to(&today)) {
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


                if let Some(body) = response {
                    match item.feed_type {
                        FeedType::Atom => {
                            let atom_feed: atom::Feed = match from_str(&body) {
                                Ok(feed) => feed,
                                Err(e) => {
                                    notify_user(&format!("could not parse atom feed: {}", e));
                                    tx.send(None).unwrap();
                                    return
                                }
                            };
                            fetched_channel = Some(atom_feed.to_internal_channel());
                        },
                        FeedType::Rss => {
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
                }
            }

            let mut channel = Channel::new_with_url(&url);

            match fetched_channel {
                Some(content) => {
                    // prefer name from url file
                    channel.name = if item.name == String::new() {
                        content.name
                    } else {
                        item.name
                    };
                    channel.tag = item.tag;

                    for h in history.channels.iter() {
                        // match channel links
                        if h.link == channel.link && h.name == channel.name {
                            // copy old video elements
                            
                            channel.videos = h.videos.clone();

                            break
                        }
                    }

                    for mut vid in content.videos.into_iter() {
                        if !channel.videos.iter().any(|video| video.link == vid.link) {
                            vid.new = true;
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

    channel_list.list_state.select(Some(0));

    channel_list
}

// fn read_urls_file() -> UrlFile {
fn read_urls_file() -> Videos {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let urls: Vec<Video> = match serde_yaml::from_str(&reader) {
                Ok(file) => file,
                Err(e) => panic!("could nor parse yaml url-file: {}", e),
            };

            let urls = Videos {
                videos: urls,
            };

            urls
        }
        Err(_) => {
            let mut file = File::create(path).unwrap();
            let videos: Vec<Video> = Vec::new();
            let string = serde_yaml::to_string(&videos).unwrap();
            match file.write_all(string.as_bytes()) {
                Ok(_) => read_urls_file(),
                Err(e) => panic!("{}", e),
            }
        }
    }
}
