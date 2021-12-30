pub(crate) mod channel;
pub(crate) mod video;
pub(crate) mod channel_list;
mod feed;

use crate::backend::history::read_history;
use crate::backend::url_file::{read_urls_file, UrlFileItem};
use crate::backend::{
    data::channel::Channel,
    data::feed::Feed,
};
use self::channel_list::ChannelList;
use reqwest::blocking::Client;
use std::sync::{mpsc::channel, mpsc::{Sender, Receiver, TryRecvError}};
use threadpool::ThreadPool;

pub(crate) struct Data {
    sender: Sender<Channel>,
    receiver: Receiver<Channel>,
}

impl Data {
    /// Init
    pub(crate) fn init() -> Self {
        let (sender, receiver) = channel();

        Self {
            sender,
            receiver,
        }
    }

    /// try receive data that was newly fetched
    pub(crate) fn try_recv(&self) -> Result<Channel, TryRecvError> {
        self.receiver.try_recv()
    }

    /// start fetching process
    pub(crate) fn update(&self) {
        let url_file_content = read_urls_file();

        // load already known items
        let history: ChannelList = read_history();

        // prepate threads
        let worker_num = 4;
        let pool = ThreadPool::new(worker_num);

        // load "normal" channels
        for item in url_file_content.channels {
            let sender_clone = self.sender.clone();
            let hc = history.clone();
            let item = item.clone();
            let urls = vec![item.url.clone()];

            pool.execute(move || {
                fetch_channel_updates(sender_clone, hc, item, urls); // updates will be send with `channel_sender`
            })
        }

        // load custom channels
        for item in url_file_content.custom_channels {
            let sender_clone = self.sender.clone();
            let hc = history.clone();
            let item = item.clone();
            let urls = item.urls.clone();

            pool.execute(move || {
                fetch_channel_updates(sender_clone, hc, item, urls); // updates will be send with `channel_sender`
            })
        }
    }
}

fn fetch_channel_updates<T: 'static + UrlFileItem + std::marker::Send>(
    channel_sender: Sender<Channel>,
    history: ChannelList,
    item: T,
    urls: Vec<String>,
) {
    // get videos from history file
    let (history_videos, history_name) = match history.get_by_id(&item.id()) {
        Some(h) => (h.videos.clone(), h.name().clone()),
        None => (Vec::new(), String::new()),
    };

    // download feed (if active)
    let feed = if item.active() {
        download_feed(&urls)
    } else {
        Feed::default()
    };

    // choose item name first; if not given take feed name; take history name as last resort
    let name = if !item.name().is_empty() {
        item.name()
    } else if !feed.name.is_empty() {
        feed.name.clone()
    } else {
        history_name
    };

    let channel = Channel::builder()
        .add_from_feed(feed)
        .with_old_videos(history_videos)
        .with_name(name)
        .with_id(item.id())
        .with_tag(item.tag())
        .with_sorting(item.sorting_method())
        .build();

    let _ = channel_sender.send(channel);
}

// download xml and parse
fn download_feed(urls: &[String]) -> Feed {
    let client = Client::builder().build().unwrap();

    let mut feed_final = Feed::default();

    // one internal feed can consist of seveal "normal" feeds
    for url in urls.iter() {

        // download feed
        let text = match client.get(url).send() {
            Ok(res) => res.text().unwrap_or_default(),
            Err(_) => continue,
        };

        // parse feed
        let mut feed = match Feed::parse_text(text) {
            Ok(f) => f,
            Err(_) => continue, // notify that feed failed
        };

        // set some meta on videos
        for vf in feed.videos.iter_mut() {
            vf.set_origin_url(url);
            vf.set_origin_channel_name(&feed.name);
        }

        // add to final feed
        feed_final.add_videos(feed.videos);
        feed_final.set_name(&feed.name);
    }

    feed_final
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use crate::data::video::Video;
 *
 *     fn test_feed() -> String {
 *         String::from("<rss><channel><title>TITLE</title><link>http://example.com</link><description>DESCRIPTION</description><ttl>123</ttl>
 *            <item>
 *                 <title>VIDEO TITLE</title>
 *                 <link>VIDEO LINK</link>
 *                 <description>VIDEO DESCRIPTION</description>
 *                 <guid isPermaLink=\"false\">123</guid>
 *                 <pubDate>Tue, 02 Mar 2021 18:55:52 UT</pubDate>
 *                 <category>CATEGORY</category>
 *            </item>
 *            </channel>
 *         </rss>")
 *     }
 *
 *     #[test]
 *     fn parser_test_err() {
 *         let output = parse_feed_to_channel(&String::new());
 *
 *         assert!(output.is_err());
 *     }
 *
 *     #[test]
 *     fn parser_test_ok() {
 *         let string = test_feed();
 *
 *         let output = parse_feed_to_channel(&String::from(string));
 *
 *         assert!(output.is_ok());
 *     }
 *
 *     #[test]
 *     fn get_channel_from_history_test() {
 *         let url = String::from("URL");
 *         let mut channel = Channel::new();
 *         channel.id = url.clone();
 *
 *         let mut history_channels = Vec::new();
 *         history_channels.push(channel);
 *
 *         [> let channel = get_channel_from_history(&url, &history_channels); <]
 *
 *         assert!(channel.is_some());
 *     }
 *
 *         #[test]
 *         fn update_existing_channel_test() {
 *             let id = String::from("ID");
 *             let tag = String::from("new_tag");
 *             let name = String::from("new_name");
 *
 *             let video = Video::new();
 *
 *             let old = Channel::new_with_id(&id);
 *
 *             let mut updates = old.clone();
 *             updates.videos.push(video);
 *
 *             let url_file_channel = UrlFileChannel {
 *                 url: String::from("URL"),
 *                 name,
 *                 updates
 *             };
 *
 *             let ret_channel = update_channel(&vec![old]);
 *
 *             assert_eq!(ret_channel.tag, tag);
 *             assert_eq!(ret_channel.name, name);
 *             assert_eq!(ret_channel.id, id);
 *             assert_eq!(ret_channel.videos.len(), 1);
 *         }
 * } */
