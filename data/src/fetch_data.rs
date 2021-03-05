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
use dirs_next::home_dir;

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

// url file video type
#[derive(Deserialize, Serialize)]
struct UrlFile {
    videos: Vec<UrlFileChannel>,
    /* custom_channel: Vec<UrlFileCustomChannel>, */
}

// url file video type
#[derive(Clone, Deserialize, Serialize)]
struct UrlFileChannel {
    url: String,
    feed_type: FeedType,
    #[serde(default = "empty_string")]
    name: String,
    #[serde(default = "date_always")]
    update_on: Vec<Date>,
    #[serde(default = "empty_string")]
    tag: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct UrlFileCustomChannel {
    url: Vec<String>,
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
impl UrlFile {
    fn len(&self) -> usize {
        self.videos.len()
    }
}

#[allow(dead_code)]
pub fn fetch_new_videos(sender: Sender<String>) -> ChannelList {

    let today = Local::now().weekday();

    let url_file_content = read_urls_file();

    let history: ChannelList = match read_history() {
        Some(content) => content,
        None => ChannelList::new(),
    };

    let mut channel_list = ChannelList::new();

    let worker_num = 4;
    let jobs_num = url_file_content.len();
    let pool = ThreadPool::new(worker_num);

    let (tx, rx) = channel();

    for item in url_file_content.videos {
        let url = item.url.clone();

        let tx = tx.clone();
        let sender = sender.clone();
        let history = history.clone();

        pool.execute(move || {

            let mut channel: Option<Channel>;

            if item.update_on.iter().any(|w| w.eq_to(&today)) {
                sender.send(format!("fetching... {}", url.clone())).unwrap();

                channel = match download_channel_updates(&url, &history.channels, item) {
                    Ok(channel) => Some(channel),
                    Err(err_text) => {
                        notify_user(&format!("Could not update channel: {}", &err_text));
                        get_channel_from_history(&url, history.channels)
                    }
                }

            } else {
                channel = get_channel_from_history(&url, history.channels);
            };

            if channel.is_some() {
                let mut ch = channel.unwrap();
                ch.videos.sort_by_key(|video| video.pub_date.clone());
                ch.videos.reverse();
                channel = Some(ch);

            }

            tx.send(channel).unwrap(); // channel is type Option
        }); // end pool
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
fn read_urls_file() -> UrlFile {
    let mut path = home_dir().unwrap();
    path.push(URLS_FILE_PATH);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut reader = String::new();
            file.read_to_string(&mut reader).unwrap();
            let urls: Vec<UrlFileChannel> = match serde_yaml::from_str(&reader) {
                Ok(file) => file,
                Err(e) => panic!("could nor parse yaml url-file: {}", e),
            };

            let urls = UrlFile {
                videos: urls,
            };

            urls
        }
        Err(_) => {
            let mut file = File::create(path).unwrap();
            let videos: Vec<UrlFileChannel> = Vec::new();
            let string = serde_yaml::to_string(&videos).unwrap();
            match file.write_all(string.as_bytes()) {
                Ok(_) => read_urls_file(),
                Err(e) => panic!("{}", e),
            }
        }
    }
}

fn download_channel_updates(url: &String, history_channels: &Vec<Channel>, url_file_entry: UrlFileChannel) -> Result<Channel, String> {
    let feed = match fetch_url(&url) {
        Ok(text) => text,
        Err(e) => {
            return Err(format!("Could not GET url: {}", e))
        }
    };

    let fetched_channel = match parse_feed_to_channel(&feed) {
        Ok(channel) => channel,
        Err(err_text) => {
            return Err(format!("Could not parse: {}", err_text))
        }
    };

    Ok(update_existing_channel(&url, url_file_entry, fetched_channel, &history_channels))
}

fn fetch_url(url: &String) -> Result<String, reqwest::Error> {
    let client = Client::builder().build().ok().unwrap();

    match client.get(url).send() {
        Ok(r) => return r.text(),
        Err(e) => return Err(e),
    }
}

fn parse_feed_to_channel(body: &String) -> Result<Channel, String> {

    // try to parse as atom
    match from_str::<atom::Feed>(body) {
        Ok(feed) => return Ok(feed.to_internal_channel()),
        Err(_) => (),
    }

    // try to parse as rss
    match from_str::<rss::Feed>(body) {
        Ok(feed) => return Ok(feed.to_internal_channel()),
        Err(_) => (),
    }

    Err(String::from("Could not parse feed"))
}

fn update_existing_channel(url: &String, url_file_entry: UrlFileChannel, channel_updates: Channel, history_channels: &Vec<Channel>) -> Channel {

    // create template
    let mut channel = Channel::new_with_url(&url);

    // set name
    channel.name = if url_file_entry.name == String::new() {
        channel_updates.name
    } else {
        url_file_entry.name
    };

    // set tag
    channel.tag = url_file_entry.tag;

    // match with history item
    for h in history_channels.iter() {
        // match channel links
        if h.link == channel.link && h.name == channel.name {

            // copy old video elements
            channel.videos = h.videos.clone();

            break
        }
    }

    for mut vid in channel_updates.videos.into_iter() {
        if !channel.videos.iter().any(|video| video.link == vid.link) {
            vid.new = true;
            channel.videos.push(vid);
        }
    }

    channel
}

fn get_channel_from_history(url: &String, history_channels: Vec<Channel>) -> Option<Channel> {
    // create template

    for h in history_channels.iter() {

        // match channel links
        if &h.link == url {

            let mut channel = Channel::new_with_url(url);

            // copy old video elements
            channel.name = h.name.clone();

            /* channel.link = h.link.clone(); */
            channel.videos = h.videos.clone();

            channel.tag = h.tag.clone();

            return Some(channel)
        }
    }

    None
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn parser_test_err() {
       let output = parse_feed_to_channel(&String::new());

       assert!(output.is_err());
    }

    #[test]
    fn parser_test_ok() {
       let string = "
           <rss><channel><title>TITLE</title><link>http://example.com</link><description>DESCRIPTION</description><ttl>123</ttl>
           <item>
                <title>VIDEO TITLE</title>
                <link>VIDEO LINK</link>
                <description>VIDEO DESCRIPTION</description>
                <guid isPermaLink=\"false\">123</guid>
                <pubDate>Tue, 02 Mar 2021 18:55:52 UT</pubDate>
                <category>CATEGORY</category>
           </item>
           </channel></rss>";

       let output = parse_feed_to_channel(&String::from(string));

       assert!(output.is_ok());
    }

    #[test]
    fn get_channel_from_history_test() {
        let url = String::from("URL");
        let mut channel = Channel::new();
        channel.link = url.clone();

        let mut history_channels = Vec::new();
        history_channels.push(channel);

        let channel = get_channel_from_history(&url, history_channels);

        assert!(channel.is_some());
    }
}
