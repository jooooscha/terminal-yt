use crate::data_types::feed_types::{atom, rss};
use crate::data_types::{channel::Channel, channel_list::ChannelList};
use crate::history::read_history;
use crate::url_file::{read_urls_file, UrlFileItem};
use chrono::prelude::*;
use notification::notify::notify_user;
use quick_xml::de::from_str;
use reqwest::blocking::Client;
use std::sync::{mpsc::channel, mpsc::Sender};
use threadpool::ThreadPool;

pub fn fetch_new_videos(
    _status_update_sender: Sender<String>,
    channel_update_sender: Sender<Channel>,
) {
    let url_file_content = read_urls_file();

    // load already known items
    let history: ChannelList = read_history();

    // prepate threads
    let worker_num = 4;
    let jobs_num = url_file_content.len();
    let pool = ThreadPool::new(worker_num);

    // prepare channel pipes
    let (channel_sender, channel_receiver) = channel();

    // load "normal" channels
    for item in url_file_content.channels {
        let channel_sender = channel_sender.clone();
        let hc = history.clone();
        let item = item.clone();
        let urls = vec![item.url.clone()];

        fetch_channel_updates(channel_sender, &pool, hc, item, urls, false); // updates will be send with `channel_sender`
    }

    // load custom channels
    for item in url_file_content.custom_channels {
        let channel_sender = channel_sender.clone();
        let hc = history.clone();
        let item = item.clone();
        let urls = item.urls.clone();

        fetch_channel_updates(channel_sender, &pool, hc, item, urls, true); // updates will be send with `channel_sender`
    }

    // receive channels from `update_video_from_url`
    for chan_opt in channel_receiver.iter().take(jobs_num) {
        match chan_opt {
            Some(chan) => {
                channel_update_sender.send(chan).unwrap();
            }
            None => (),
        }
    }
}

fn fetch_channel_updates<T: 'static + UrlFileItem + std::marker::Send>(
    channel_sender: Sender<Option<Channel>>,
    pool: &ThreadPool,
    history: ChannelList,
    item: T,
    urls: Vec<String>,
    is_custom: bool,
) {
    pool.execute(move || {
        let today = Local::now().weekday();

        let mut channel = if item.update_on().iter().any(|w| w.eq_to(&today)) {
            // download updates
            match download_channel_updates(&urls) {
                Ok(channel_updates) => {

                    let mut c: Channel;

                    // merge history into updates
                    if let Some(history_channel) = history.get_by_id(&item.id()) {
                        c = Channel::new_from_history(history_channel);
                        c.merge_videos(channel_updates.videos);
                    } else {
                        c = channel_updates;
                    }

                    if is_custom {
                        c.id = item.name();
                    } else {
                        c.id = urls.first().unwrap().clone();
                    }

                    Some(c)
                },
                Err(err_text) => {
                    notify_user(&format!("Could not update {}: {}", &item.id(), &err_text));
                    history.get_by_id(&item.id()).cloned()
                }
            }
        } else {
            history.get_by_id(&item.id()).cloned()
        };

        if let Some(ref mut ch) = channel {
            ch.update_from_url_file(&item);
            ch.sort();
        }

        match channel_sender.send(channel) {
            Ok(_) => (),
            Err(error) => panic!("error on sending channel: {}", error),
        }
    }); // end pool
}

fn download_channel_updates(urls: &Vec<String>) -> Result<Channel, String> {
    let mut new_channel = None;

    for url in urls.iter() {
        let feed = match fetch_feed(url) {
            Ok(text) => text,
            Err(e) => return Err(format!("Could not GET url: {}", e)),
        };

        let fetched_channel = match parse_feed_to_channel(&feed, &url.clone()) {
            Ok(channel) => channel,
            Err(err_text) => return Err(format!("Could not parse: {}", err_text)),
        };

        if new_channel.is_none() {
            new_channel = Some(fetched_channel);
        } else {
            let mut chan_temp = new_channel.clone().unwrap();
            chan_temp.merge_videos(fetched_channel.videos);
            new_channel = Some(chan_temp);
        }
    }

    match new_channel {
        Some(channel) => return Ok(channel),
        None => return Err(String::from("No new content found")),
    }
}

fn fetch_feed(url: &String) -> Result<String, reqwest::Error> {
    let client = Client::builder().build()?;

    match client.get(url).send() {
        Ok(r) => return r.text(),
        Err(e) => return Err(e),
    }
}

fn parse_feed_to_channel(body: &String, origin_url: &String) -> Result<Channel, String> {
    let mut channel: Option<Channel> = None;

    // try to parse as atom
    if channel.is_none() {
        channel = match from_str::<atom::Feed>(body) {
            Ok(feed) => Some(Channel::from(feed)),
            Err(_) => None,
        };
    }

    // try to parse as rss
    if channel.is_none() {
        channel = match from_str::<rss::Feed>(body) {
            Ok(feed) => Some(Channel::from(feed)),
            Err(_) => None,
        };
    }

    match channel {
        Some(mut ch) => {
            ch.add_origin_url(origin_url);
            Ok(ch)
        }
        None => Err(String::from("Could not parse feed")),
    }
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use crate::url_file::Date::*;
 *     use crate::url_file::UrlFileChannel;
 *
 *     #[test]
 *     fn test_fetch() {
 *
 *         let (cs, cr) = channel();
 *         let urls = vec![String::from("https://www.youtube.com/feeds/videos.xml?channel_id=UC8uT9cgJorJPWu7ITLGo9Ww")];
 *         let pool = ThreadPool::new(4);
 *         let history = ChannelList::new();
 *         let item = UrlFileChannel {
 *             url: urls[0].clone(),
 *             name: String::from("test name"),
 *             update_on: vec![Always],
 *             tag: String::from("test tag"),
 *         };
 *
 *         update_videos_from_url(cs, &pool, history, item, urls);
 *
 *         for channel in cr.try_iter() {
 *             println!("{:?}", channel);
 *         }
 *
 *         assert!(false)
 *
 *     }
 * } */

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use crate::data_types::video::Video;
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
