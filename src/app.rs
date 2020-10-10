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
use fetch_data::{
    fetch_data::*,
    config::Config,
};
use crate::draw;

use Action::*;
use Screen::*;

pub struct App {
    pub terminal: Terminal<TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>,
    pub config: Config,
    pub update_line: String,
    pub app_title: String,
    pub current_screen: Screen,
    channel_list: ChannelList,
    backup_list: ChannelList,
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

#[derive(PartialEq)]
pub enum Filter {
    All,
    Visible,
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

        let config = Config::read_config_file();

        let mut app = App {
            terminal,
            config: config.clone(),
            app_title: config.app_title,
            current_screen: Channels,
            channel_list: ChannelList::new(),
            backup_list: ChannelList::new(),
            current_selected: 0,
            update_line: String::new(),
        };
        app.set_channel_list(channel_list);

        app
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
                    Channels => self.get_filtered_chan(Filter::Visible).prev(),
                    Videos => self.get_selected_channel().prev(),
                }
                self.update();
            },
            Down => {
                match self.current_screen {
                    Channels => self.get_filtered_chan(Filter::Visible).next(),
                    Videos => self.get_selected_channel().next(),
                }
                self.update();
            },
            Enter => {
                self.current_selected = match self.get_filtered_chan(Filter::Visible).list_state.selected() {
                    Some(selected) => selected,
                    None => return,
                };
                self.current_screen = Videos;
                self.get_filtered_chan(Filter::Visible).list_state.select(None);
                self.get_selected_channel().list_state.select(Some(0));
                self.update();
            },
            Back => {
                self.current_screen = Channels;
                let curr_sel = self.current_selected.clone();
                self.get_filtered_chan(Filter::Visible).list_state.select(Some(curr_sel));
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
    pub fn set_channel_list(&mut self, cl: ChannelList) {
        self.channel_list = cl;
        self.channel_list.list_state.select(Some(0));
    }
    pub fn get_current_selected(&self) -> usize {
        self.current_selected
    }
    pub fn get_filtered_chan(&mut self, filter: Filter) -> &mut ChannelList {

        // base list
        let backup = &self.backup_list;

        // start new with current list (may be filtered)
        let mut new = self.channel_list.clone();

        // add all channels that are not in the list
        for chan in backup.channels.iter() {
            if !new.channels.iter().any(|c| c.link == chan.link) {
                new.channels.push(chan.clone());
            }
        }

        new.sort();

        // save as new base list
        self.backup_list = new.clone();

        // filter channels that have only marked videos
        if filter == Filter::Visible && self.current_screen == Channels { // only filter if on channel screen
            if !self.config.show_empty_channels {
                new.channels = new.channels.into_iter().filter(|c| c.videos.iter().any(|v| !v.marked)).collect();
            }
        }

        self.channel_list = new;

        &mut self.channel_list
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
    fn save(&mut self) {
        write_history(self.get_filtered_chan(Filter::All));
    }
}
