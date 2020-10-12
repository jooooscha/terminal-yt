use std::{
    io::{
        stdout,
        stdin,
    },
    cmp,
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
use data_types::internal::{
    *,
    Filter,
};
use fetch_data::{
    fetch_data::*,
    config::Config,
};
use crate::draw;

use Action::*;
use Screen::*;

pub struct App {
    pub terminal: Terminal<TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>,
    /* pub terminal: Terminal<TermionBackend<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>, */
    pub config: Config,
    pub update_line: String,
    pub app_title: String,

    pub current_screen: Screen,
    pub filter: Filter,
    current_selected: usize, // channel

    channel_list: ChannelList,
    /* backup_list: ChannelList, */
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
    pub fn new_with_channel_list(mut channel_list: ChannelList) -> App {
        let stdout = stdout().into_raw_mode().unwrap();
        let mouse_terminal = MouseTerminal::from(stdout);
        /* let screen = mouse_terminal; */
        let screen = AlternateScreen::from(mouse_terminal);
        let _stdin = stdin();
        let backend = TermionBackend::new(screen);
        let terminal = Terminal::new(backend).unwrap();

        let config = Config::read_config_file();

        let filter = if config.show_empty_channels {
            Filter::NoFilter
        } else {
            Filter::OnlyNew
        };

        channel_list.list_state.select(Some(0));

        App {
            terminal,
            config: config.clone(),
            app_title: config.app_title,
            current_screen: Channels,
            channel_list,
            /* backup_list: ChannelList::new(), */
            current_selected: 0,
            update_line: String::new(),
            filter,
        }
    }
    pub fn action(&mut self, action: Action) {
        match action {
            Mark | Unmark => {
                if self.current_screen == Videos {
                    let state = action == Mark;
                    match self.get_selected_video() {
                        Some(v) => v.mark(state),
                        None => return,
                    }
                    if !self.get_selected_channel().unwrap().has_new() {
                        self.action(Back);
                    } else if self.config.down_on_mark {
                        if let Some(e) = self.get_selected_channel() {
                            e.next();
                        }
                    }
                    self.update();
                    self.save();
                }
            },
            Up => {
                match self.current_screen {
                    Channels => self.get_channel_list().prev(),
                    Videos => {
                        if let Some(c) = self.get_selected_channel() {
                            c.prev();
                        }
                    },
                }
                self.update();
            },
            Down => {
                match self.current_screen {
                    Channels => self.get_channel_list().next(),
                    Videos => if let Some(c) = self.get_selected_channel() {
                        c.next();
                    },
                }
                self.update();
            },
            Enter => {
                self.current_selected = match self.get_channel_list().list_state.selected() {
                    Some(selected) => selected,
                    None => return,
                };
                self.current_screen = Videos;
                self.get_channel_list().list_state.select(None);
                if let Some(c) = self.get_selected_channel() {
                    c.list_state.select(Some(0));
                }
                self.update();
            },
            Back => {
                self.current_screen = Channels;
                let curr_sel = self.current_selected.clone();
                let len: usize = cmp::max(0, self.get_channel_list().channels.len() as isize - 1) as usize;
                let curr_sel = cmp::min(curr_sel, len);
                self.get_channel_list().list_state.select(Some(curr_sel));
                self.update();
            },
            Open => {
                if let Some(v) = self.get_selected_video() { v.open() };
            },
            Update => {
                draw(self)
            },
        }
    }
    /* pub fn update_channel_list(&mut self, cl: ChannelList) {
     *     self.set_channel_list(cl);
     *     self.filter_channel_list(self.current_filter);
     * } */

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.set_channel_list(self.channel_list.clone());
    }

    pub fn set_channel_list(&mut self, mut new_cl: ChannelList) {

        // copy old filter
        new_cl.filter(self.filter);

        // use correct selection
        match self.current_screen {
            Channels => {
                let len: usize = cmp::max(0, new_cl.channels.len() as isize - 1) as usize;
                let selected = new_cl.list_state.selected();

                self.channel_list = new_cl;

                let selected = cmp::min(selected, Some(len));
                self.channel_list.list_state.select(selected)
            },
            Videos => {
                let pos_in_chan = if let Some(c) = self.get_selected_channel() {
                    c.list_state.selected()
                } else {
                    return
                };
                self.channel_list = new_cl;
                match self.get_selected_channel() {
                    Some(c) => c.list_state.select(pos_in_chan),
                    None => self.action(Back),
                };
            },
        }
    }
    pub fn get_channel_list(&mut self) -> &mut ChannelList {
        &mut self.channel_list
    }
/*     pub fn filter_channel_list(&mut self, filter: Filter) {
 *
 *         // base list
 *         let backup = &self.backup_list;
 *
 *         // start new with current list (may be filtered)
 *         let mut new = self.channel_list.clone();
 *
 *         // add all channels that are not in the list
 *         for chan in backup.channels.iter() {
 *             if !new.channels.iter().any(|c| c.link == chan.link) {
 *                 new.channels.push(chan.clone());
 *             }
 *         }
 *
 *         new.sort();
 *
 *         // save as new base list
 *         self.backup_list = new.clone();
 *
 *         // filter channels that have only marked videos
 *         [> if self.current_screen == Channels { // only filter if on channel screen <]
 *             self.current_filter = filter;
 *             if filter == Filter::OnlyNew {
 *                 if !self.config.show_empty_channels {
 *                     new.channels = new.channels.into_iter().filter(|c| c.videos.iter().any(|v| !v.marked)).collect();
 *                 }
 *             }
 *         [> } <]
 *
 *         [> self.channel_list = new; <]
 *         self.set_channel_list(new);
 *     } */
    pub fn get_current_selected(&self) -> usize {
        self.current_selected
    }
    fn update(&mut self) {
        draw(self);
    }
    //--------------
    fn get_selected_channel(&mut self) -> Option<&mut Channel> {
        let i = self.current_selected;
        self.channel_list.channels.get_mut(i)
    }
    fn get_selected_video(&mut self) -> Option<&mut Video> {
        let c = match self.get_selected_channel() {
            Some(c) => c,
            None => return None
        };
        let i = c.list_state.selected().unwrap();
        c.videos.get_mut(i)
    }
    //---------------
    fn save(&mut self) {
        let f = self.filter;
        self.set_filter(Filter::NoFilter);
        write_history(self.get_channel_list());
        self.set_filter(f);
    }
}
