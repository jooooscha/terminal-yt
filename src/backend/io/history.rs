use crate::backend::{
    io::{read_config, write_config, FileType::HistoryFile},
    data::video::Video,
    ToTuiListItem,
};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::ListItem,
};

#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct History {
    list: Vec<MinimalVideo>,
}

impl History {
    pub(crate) fn load() -> Self {
        let history = read_config(HistoryFile);
        match serde_json::from_str(&history) {
            Ok(list) => Self { list },
            Err(_) =>  Self::default(),
        }
    }

    fn save(&self) {
        let string = serde_json::to_string(&self.list).unwrap();
        write_config(HistoryFile, &string);
    }

    pub(crate) fn add(&mut self, video: Video) {
        let mimimal_video = MinimalVideo::from(video);

        // remove if already exist and put new one in
        for i in 0..self.list.len() {
            if self.list[i] == mimimal_video {
                self.list.remove(i);
                break;
            }
        }

        self.list.push(mimimal_video);

        self.save()
    }

    pub(crate) fn to_list_items(&self) -> Vec<ListItem> {
        self.list.iter()
            .map(|v| v.to_list_item())
            .rev()
            .collect()
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

impl From<Video> for MinimalVideo {
    fn from(video: Video) -> MinimalVideo {
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
