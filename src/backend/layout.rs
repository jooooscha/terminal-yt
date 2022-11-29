use crate::backend::{
    Backend, Screen,
    Screen::*,
};
use tui::{
    layout::{Constraint::*, Direction, Layout, Rect},
    Frame,
};

// pub enum Region {
// }

pub struct AppLayout {
    main: Vec<Rect>,
    content: Vec<Rect>,
    // history: Vec<Rect>,
}

impl AppLayout {
    pub fn load(f: &mut Frame<'_, Backend>, screen: &Screen) -> Self {
        let video_size = match screen {
            Channels => 0,
            Videos => 75,
        };

        let main_split = vec![Percentage(80), Percentage(19), Percentage(1)];
        let content_split = vec![Percentage(100 - video_size), Percentage(video_size)];
        // let history_split = vec![Percentage(75), Percentage(25)];

        let main = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(main_split)
            .split(f.size());

        let content = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(content_split)
            .split(main[0]);

        // let history = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .margin(0)
        //     .constraints(history_split)
        //     .split(main[1]);

        Self {
            main,
            content,
            // history
        }
    }

    pub fn info(&self) -> Rect {
        self.main[2]
    }

    pub fn history(&self) -> Rect {
        // self.history[0]
        self.main[1]
    }

    // pub fn stats(&self) -> Rect {
    //     self.history[1]
    // }

    pub fn channels(&self) -> Rect {
        self.content[0]
    }

    pub fn videos(&self) -> Rect {
        self.content[1]
    }
}
