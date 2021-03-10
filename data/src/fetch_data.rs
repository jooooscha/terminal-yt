use reqwest::blocking::Client;
use quick_xml::de::from_str;
use std::{
    sync::{
        mpsc::Sender,
        mpsc::channel,
    },
};

use threadpool::ThreadPool;

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
use crate::url_file::{
    UrlFileItem,
    ChannelId,
    read_urls_file,
};

use notification::notify::notify_user;

pub fn fetch_new_videos(status_update_sender: Sender<String>, channel_update_sender: Sender<Channel>) {

    let url_file_content = read_urls_file();

    // load already known items
    let history: ChannelList = match read_history() {
        Some(content) => content,
        None => ChannelList::new(),
    };

    // prepate threads
    let worker_num = 4;
    let jobs_num = url_file_content.len();
    let pool = ThreadPool::new(worker_num);

    // prepare channel pipes
    let (channel_sender, channel_receiver) = channel();

    // load "normal" channels
    for item in url_file_content.channels {
        let channel_sender = channel_sender.clone();
        let history = history.clone();
        let url = item.url.clone();

        update_videos_from_url(channel_sender, &pool, history.channels, item.clone(), vec![url]); // updates will be send with `channel_sender`
    }

    // load custom channels
    for item in url_file_content.custom_channels {
        let cs = channel_sender.clone();
        let hc = history.channels.clone();
        let item = item.clone();
        let urls = item.urls.clone();

        update_videos_from_url(cs, &pool, hc, item, urls); // updates will be send with `channel_sender`
    }

    // receive channels from `update_video_from_url`
    for (i, chan_opt) in channel_receiver.iter().take(jobs_num).enumerate() {

        status_update_sender.send(format!("Updated: {}/{}", i+1, jobs_num)).unwrap();

        match chan_opt {
            // Some(chan) => channel_list.channels.push(chan),
            Some(chan) => channel_update_sender.send(chan).unwrap(),
            None => (),
        }
    }

/*     channel_list.list_state.select(Some(0));
 *
 *     channel_list */
}

fn update_videos_from_url<T: 'static + UrlFileItem + std::marker::Send>(
         channel_sender: Sender<Option<Channel>>,
         pool: &ThreadPool,
         history_channels: Vec<Channel>,
         item: T,
         urls: Vec<String>
    ) {

    pool.execute(move || {
        let today = Local::now().weekday();

        let mut channel: Option<Channel>;

        if item.update_on().iter().any(|w| w.eq_to(&today)) {

            channel = match download_channel_updates(&urls) {
                Ok(channel_updates) => {
                    let merged_channel = update_channel(&item, channel_updates, &history_channels);
                    Some(merged_channel)
                }
                Err(err_text) => {
                    notify_user(&format!("Could not update channel: {}", &err_text));
                    get_channel_from_history(&item.id(), &history_channels)
                }
            }

        } else {
            channel = get_channel_from_history(&item.id(), &history_channels);
        };

        if channel.is_some() {
            let mut ch = channel.unwrap();

            // filter videos from removed urls in custom channel
            ch.videos = ch.videos.into_iter().filter(
                |video| urls.iter().any(
                    |url| url == &video.channel_link
                )
            ).collect();

            ch.videos.sort_by_key(|video| video.pub_date.clone());
            ch.videos.reverse();
            channel = Some(ch);
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
            Err(e) => {
                return Err(format!("Could not GET url: {}", e))
            }
        };

        let mut fetched_channel = match parse_feed_to_channel(&feed, &url.clone()) {
            Ok(channel) => channel,
            Err(err_text) => {
                return Err(format!("Could not parse: {}", err_text))
            }
        };

        if new_channel.is_none() {
            new_channel = Some(fetched_channel);
        } else {
            let mut chan_temp = new_channel.clone().unwrap();
            chan_temp.videos.append(&mut fetched_channel.videos);
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

    // try to parse as atom
    match from_str::<atom::Feed>(body) {
        Ok(feed) => return Ok(feed.to_internal_channel(origin_url)),
        Err(_) => (),
    }

    // try to parse as rss
    match from_str::<rss::Feed>(body) {
        Ok(feed) => return Ok(feed.to_internal_channel(origin_url)),
        Err(_) => (),
    }

    Err(String::from("Could not parse feed"))
}

fn update_channel<T: 'static + UrlFileItem>(
        item: &T,
        channel_updates: Channel,
        history_channels: &Vec<Channel>
    ) -> Channel {

    // create template
    let mut channel = Channel::new_with_id(item.id());

    // set name - prefere name declard in url-file
    channel.name = if item.name().is_empty() {
        channel_updates.name.clone()
    } else {
        item.name().clone()
    };

    // set tag
    channel.tag = item.tag().clone();

    // insert already known videos
    match get_channel_from_history(&item.id(), history_channels) {
        Some(channel_h) => channel.videos = channel_h.videos.clone(),
        None => (),
    }

    // insert new videos
    for mut vid in channel_updates.videos.into_iter() {
        if !channel.videos.iter().any(|video| video.link == vid.link) {
            vid.new = true;
            channel.videos.push(vid);
        }
    }

    channel // return updated channel
}

fn get_channel_from_history(id: &ChannelId, history_channels: &Vec<Channel>) -> Option<Channel> {
    for channel in history_channels.iter() {
        if &channel.id == id {
            return Some(channel.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests{
    use super::*;
    use data_types::internal::Video;

    fn test_feed() -> String {
        String::from("<rss><channel><title>TITLE</title><link>http://example.com</link><description>DESCRIPTION</description><ttl>123</ttl>
           <item>
                <title>VIDEO TITLE</title>
                <link>VIDEO LINK</link>
                <description>VIDEO DESCRIPTION</description>
                <guid isPermaLink=\"false\">123</guid>
                <pubDate>Tue, 02 Mar 2021 18:55:52 UT</pubDate>
                <category>CATEGORY</category>
           </item>
           </channel>
        </rss>")
    }

    #[test]
    fn parser_test_err() {
       let output = parse_feed_to_channel(&String::new());

       assert!(output.is_err());
    }

    #[test]
    fn parser_test_ok() {
       let string = test_feed();

       let output = parse_feed_to_channel(&String::from(string));

       assert!(output.is_ok());
    }

    #[test]
    fn get_channel_from_history_test() {
        let url = String::from("URL");
        let mut channel = Channel::new();
        channel.id = url.clone();

        let mut history_channels = Vec::new();
        history_channels.push(channel);

        let channel = get_channel_from_history(&url, &history_channels);

        assert!(channel.is_some());
    }

/*     #[test]
 *     fn update_existing_channel_test() {
 *         let id = String::from("ID");
 *         let tag = String::from("new_tag");
 *         let name = String::from("new_name");
 *
 *         let video = Video::new();
 *
 *         let old = Channel::new_with_id(&id);
 *
 *         let mut updates = old.clone();
 *         updates.videos.push(video);
 *
 *         let url_file_channel = UrlFileChannel (
 *             url: String::from("URL");
 *             name,
 *             updates
 *         )
 *
 *         let ret_channel = update_channel_attr(, updates, &vec![old]);
 *
 *
 *         assert_eq!(ret_channel.tag, tag);
 *         assert_eq!(ret_channel.name, name);
 *         assert_eq!(ret_channel.id, id);
 *         assert_eq!(ret_channel.videos.len(), 1);
 *     } */
}
