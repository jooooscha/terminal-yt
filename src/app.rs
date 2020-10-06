use std::io::{
    stdout,
    stdin,
};
use tui::{
    Terminal,
    backend::TermionBackend,
};
use termion::{
    raw::IntoRawMode,
    screen::AlternateScreen,
    input::MouseTerminal,
};
use data_types::internal::*;
use fetch_data::fetch_data::*;
use crate::config::Config;
use crate::draw;

use Action::*;
use Screen::*;

pub struct App {
    pub terminal: Terminal<TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>,
    pub config: Config,
    pub update_line: String,
    pub channel_list: ChannelList,
    pub app_title: String,
    pub current_screen: Screen,
    current_selected: usize,
}

#[derive(PartialEq)]
pub enum Action {
    Mark,
    Unmark,
    Up,
    Down,
    Enter,
    Back,
    Open,
    Update,
}

#[derive(PartialEq, Clone)]
pub enum Screen {
    Channels,
    Videos,
}

impl App {
    pub fn new_with_channel_list(channel_list: ChannelList) -> App {
        let stdout = stdout().into_raw_mode().unwrap();
        let mouse_terminal = MouseTerminal::from(stdout);
        /* let screen = mouse_terminal; */
        let screen = AlternateScreen::from(mouse_terminal);
        let _stdin = stdin();
        let backend = TermionBackend::new(screen);
        let terminal = Terminal::new(backend).unwrap();

        let config = Config::default();

        App {
            terminal,
            config: config.clone(),
            app_title: config.app_title,
            current_screen: Channels,
            channel_list,
            current_selected: 0,
            update_line: String::new(),
        }
    }
    pub fn action(&mut self, action: Action) {
        match action {
            Mark | Unmark => {
                let state = action == Mark;
                if self.current_screen == Videos {
                    self.get_selected_video().mark(state);
                    if self.config.down_on_mark {
                        self.get_selected_channel().next();
                    }
                    self.update();
                    self.save();
                }
            },
            Up => {
                match self.current_screen {
                    Channels => self.channel_list.prev(),
                    Videos => self.get_selected_channel().prev(),
                }
                self.update();
            },
            Down => {
                match self.current_screen {
                    Channels => self.channel_list.next(),
                    Videos => self.get_selected_channel().next(),
                }
                self.update();
            },
            Enter => {
                self.current_selected = match self.channel_list.list_state.selected() {
                    Some(selected) => selected,
                    None => return,
                };
                self.current_screen = Videos;
                self.channel_list.list_state.select(None);
                self.get_selected_channel().list_state.select(Some(0));
                self.update();
            },
            Back => {
                self.current_screen = Channels;
                self.channel_list.list_state.select(Some(self.current_selected));
                self.update();
            },
            Open => {
                self.get_selected_video().open();
            },
            Update => {
                draw(self)
            },
        }
    }
    pub fn get_current_selected(&self) -> usize {
        self.current_selected
    }
    fn update(&mut self) {
        draw(self);
    }
    //--------------
    fn get_selected_channel(&mut self) -> &mut Channel {
        let i = self.current_selected;
        &mut self.channel_list.channels[i]
    }
    fn get_selected_video(&mut self) -> &mut Video {
        let c = self.get_selected_channel();
        let i = c.list_state.selected().unwrap();
        &mut c.videos[i]
    }
    //---------------
    fn save(&self) {
        write_history(&self.channel_list);
    }
}
