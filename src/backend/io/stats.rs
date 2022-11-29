use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use crate::backend::data::video::Video;

/// Statistics collected in total
#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct Stats {
    pub per_day: HashMap<NaiveDate, DailyStats>,
}

/// Statistics collected per day
#[derive(Clone, Deserialize, Serialize, Default)]
pub(crate) struct DailyStats {
    pub starts: usize,
    pub watched: usize,
    pub channels: HashMap<String, usize>,
}

impl Stats {
    pub fn today(&self) -> Option<&DailyStats> {
        let now: NaiveDate = Local::now().date_naive();
        self.per_day.get(&now)
    }

    pub fn today_mut(&mut self) -> &mut DailyStats {
        let now: NaiveDate = Local::now().date_naive();


        if let None = self.per_day.get_mut(&now) {
            let daily_stat = DailyStats::default();
            self.per_day.insert(now, daily_stat);
        }

        self.per_day.get_mut(&now).unwrap()
    }

    pub fn add(&mut self, video: &Video)  {
        let stat_today = self.today_mut();
        stat_today.add_video(video);
    }

    pub fn add_start(&mut self) {
        self.today_mut().starts += 1;
    }

    // pub(crate) fn stat_today(&self) -> Option<&Stats> {
    //     let now: NaiveDate = Local::now().date_naive();
    //     self.stats.get(&now)
    // }

    // pub fn stat_insert_today(&mut self, stat: Stats) {
    //     let now: NaiveDate = Local::now().date_naive();
    //     self.stats.insert(now, stat);
    // }


    // fn stat_today_mut(&mut self) -> Option<&mut Stats> {
    //     let now: NaiveDate = Local::now().date_naive();
    //     self.stats.get_mut(&now)
    // }
}

impl DailyStats {
    fn add_video(&mut self, video: &Video) {
        self.watched += 1; // increase total counter
        let video_name = video.origin_channel_name();
        let channel = self.channels.get_mut(video_name);
        match channel {
            Some(number) => *number += 1, // channel already there
            None => { let _ = self.channels.insert(video_name.clone(), 1); },
        }
    }
}
