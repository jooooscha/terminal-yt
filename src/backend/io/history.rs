use crate::backend::{
    data::video::Video,
    io::{read_config, write_config, FileType::HistoryFile},
    ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};
use std::collections::HashMap;
use chrono::prelude::*;

#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct Stats {
    // starts: usize,
    pub watched: usize,
    pub channels: HashMap<String, usize>,
}

impl Stats {
    pub(crate) fn add(&mut self, video: &Video)  {
        self.watched += 1; // increase total counter

        let video_name = video.origin_channel_name();
        let channel = self.channels.get_mut(video_name);
        match channel {
            Some(number) => *number += 1, // channel already there
            None => { let _ = self.channels.insert(video_name.clone(), 1); },
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct History {
    list: Vec<MinimalVideo>,
    #[serde(default)]
    stats: HashMap<NaiveDate, Stats>,
}

impl History {

    /// load history file, parse, and return
    pub(crate) fn load() -> Self {
        let history = read_config(HistoryFile);

        // this is only for compatability reasons with old History struct
        match serde_json::from_str::<Vec<MinimalVideo>>(&history) {
            Ok(list) => {
                let stats = HashMap::default();
                return Self { list, stats };
            }
            _ => (),
        }

        serde_json::from_str::<History>(&history).unwrap_or_default()
    }

    fn save(&self) {
        let string = serde_json::to_string(&self).unwrap();
        write_config(HistoryFile, &string);
    }

    pub(crate) fn video_opened(&mut self, video: &Video) {
        let mimimal_video = MinimalVideo::from(video);

        // remove if already exist and put new one in
        for i in 0..self.list.len() {
            if self.list[i] == mimimal_video {
                self.list.remove(i);
                break;
            }
        }

        self.list.push(mimimal_video);

        match self.stat_today_mut() {
            Some(stat) => stat.add(video),
            None => {
                let mut stat = Stats::default();
                stat.add(video);
                self.stat_insert_today(stat);
            },
        }

        self.save()
    }

    pub(crate) fn stat_today(&self) -> Option<&Stats> {
        let now: NaiveDate = Local::now().date_naive();
        self.stats.get(&now)
    }

    pub fn stat_insert_today(&mut self, stat: Stats) {
        let now: NaiveDate = Local::now().date_naive();
        self.stats.insert(now, stat);
    }


    fn stat_today_mut(&mut self) -> Option<&mut Stats> {
        let now: NaiveDate = Local::now().date_naive();
        self.stats.get_mut(&now)
    }

    pub(crate) fn to_list_items(&self) -> Vec<ListItem> {
        self.list.iter().map(|v| v.to_list_item()).rev().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MinimalVideo {
    title: String,
    channel: String,
}

impl ToTuiListItem for MinimalVideo {
    fn to_list_item(&self) -> ListItem {
        let channel = format!("{} {} - ", tui::symbols::DOT, &self.channel);
        let title = (&self.title).to_string();

        let style = Style::default().fg(Color::DarkGray);

        ListItem::new(Spans::from(vec![
            Span::styled(channel, style),
            Span::styled(title, style.add_modifier(Modifier::ITALIC)),
        ]))
    }
}

impl From<&Video> for MinimalVideo {
    fn from(video: &Video) -> MinimalVideo {
        MinimalVideo {
            title: video.title().clone(),
            channel: video.origin_channel_name().clone(),
        }
    }
}

/*
 * #[cfg(test)]
 * pub mod tests {
 *     use super::*;
 *     use crate::data::{
 *         channel::factory::ChannelFactory, video::factory::tests::get_random_video_factory,
 *     };
 *     use std::fs::remove_file;
 *
 *     #[test]
 *     fn test_rw_history() {
 *         let mut channels = Vec::new();
 *         for _ in 0..10 {
 *             let mut cf = ChannelFactory::test();
 *
 *             let mut videos = Vec::new();
 *             for _ in 0..10 {
 *                 videos.push(get_random_video_factory());
 *             }
 *             cf.add_new_videos(videos);
 *
 *             let channel = cf.commit().unwrap();
 *             channels.push(channel);
 *         }
 *
 *         let input = ChannelList::test(channels);
 *
 *         let file = "./test_write_history";
 *
 *         write_history_intern(&input, file);
 *         let output = read_history_intern(file);
 *
 *         assert_eq!(input, output);
 *
 *         let _ = remove_file(file);
 *     }
 * } */
