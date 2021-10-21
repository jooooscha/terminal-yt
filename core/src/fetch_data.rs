use crate::history::read_history;
use crate::url_file::{read_urls_file, UrlFileItem};
use crate::{
    data_types::{
        channel::{channel::Channel, factory::ChannelFactory},
        channel_list::ChannelList,
        feed_types::{atom, rss, Feed},
    },
};
use chrono::prelude::*;
use quick_xml::de::from_str;
use reqwest::blocking::Client;
use std::sync::{mpsc::channel, mpsc::Sender};
use threadpool::ThreadPool;

pub fn fetch_new_videos(
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

        fetch_channel_updates(channel_sender, &pool, hc, item, urls); // updates will be send with `channel_sender`
    }

    // load custom channels
    for item in url_file_content.custom_channels {
        let channel_sender = channel_sender.clone();
        let hc = history.clone();
        let item = item.clone();
        let urls = item.urls.clone();

        fetch_channel_updates(channel_sender, &pool, hc, item, urls); // updates will be send with `channel_sender`
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
) {
    pool.execute(move || {
        let today = Local::now().weekday();

        let mut channel_factory = if item.update_on().iter().any(|w| w.eq_to(&today)) {
            download_channel_updates(&urls)
        } else {
            ChannelFactory::create()
        };

        // set videos from history
        let (history_videos, history_name) = match history.get_by_id(&item.id()) {
            Some(h) => (h.videos.clone(), h.name().clone()),
            None => (Vec::new(), String::new()),
        };

        channel_factory.set_old_videos(history_videos);

        if !channel_factory.new_videos_added() {
            channel_factory.add_new_videos(Vec::new());
        }

        // set id
        channel_factory.set_id(item.id());

        if !item.name().is_empty() {
            channel_factory.set_name(item.name());
        } else if !channel_factory.name_set() {
            channel_factory.set_name(history_name);
        }

        if item.tag().is_empty() {
            channel_factory.set_tag(String::new());
        } else {
            channel_factory.set_tag(item.tag().clone());
        }

        channel_factory.set_sorting(item.sorting_method());

        let channel = match channel_factory.commit() {
            Ok(channel) => channel,
            Err(error) => panic!("{}", error),
        };

        match channel_sender.send(Some(channel)) {
            Ok(_) => (),
            Err(error) => panic!("error on sending channel: {}", error),
        }
    }); // end pool
}

fn download_channel_updates(urls: &Vec<String>) -> ChannelFactory {
    let mut cf = ChannelFactory::create();

    for url in urls.iter() {
        let mut feed = match fetch_feed(url) {
            Ok(f) => f,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("Could not GET url: {}", _e);
                continue;
            }
        };

        cf.set_name(feed.name.clone());
        for vf in feed.videos.iter_mut() {
            vf.set_origin_url(url.clone());
            vf.set_origin_channel_name(feed.name.clone());
            vf.set_marked(false);
            vf.set_new(true);
            vf.set_fav(false);
        }
        cf.add_new_videos(feed.videos);
    }

    cf
}

fn fetch_feed(url: &String) -> Result<Feed, String> {
    let client = Client::builder().build().unwrap();

    let feed: String = match client.get(url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err(e.to_string()),
    };

    // ------------------------------------------------------------------

    // try to parse as atom
    match from_str::<atom::Feed>(&feed) {
        Ok(feed) => return Ok(feed.into()), //Some(ChannelFactory::from((feed, url.clone()))),
        Err(_) => (),
    };

    // try to parse as rss
    match from_str::<rss::Feed>(&feed) {
        Ok(feed) => return Ok(feed.into()), //Some(ChannelFactory::from((feed, url.clone()))),
        Err(_) => (),
    }

    Err(String::from("Could not parse feed"))
}

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
