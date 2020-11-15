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
use data::{
    history::*,
    config::Config,
};
use crate::draw;

use Action::*;
use Screen::*;

// The main struct containing everything important
pub struct App {
    pub terminal: Terminal<TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>,
    pub config: Config,
    pub update_line: String,
    pub app_title: String,
    pub current_screen: Screen,
    pub filter: Filter,
    current_selected: usize, // channel
    channel_list: ChannelList,
    pub playback_history: Vec<MinimalVideo>,
}

#[derive(PartialEq)]
pub enum Action {
    Mark,
    Unmark,
    Up,
    Down,
    Enter,
    Back,
    NextChannel,
    PrevChannel,
    Open,
    Update,
}


#[derive(PartialEq, Clone)]
pub enum Screen {
    Channels,
    Videos,
}

impl App {
    pub fn new_with_channel_list(mut channel_list: ChannelList) -> Self {
        let stdout = stdout().into_raw_mode().unwrap();
        let mouse_terminal = MouseTerminal::from(stdout);
        /* let screen = mouse_terminal; */
        let screen = AlternateScreen::from(mouse_terminal);
        let _stdin = stdin();
        let backend = TermionBackend::new(screen);
        let terminal = Terminal::new(backend).unwrap();

        // ------------------------------------------
        let config = Config::read_config_file();

        let filter = if config.show_empty_channels {
            Filter::NoFilter
        } else {
            Filter::OnlyNew
        };

        // ------------------------------------------

        let playback_history = read_playback_history();

        // ------------------------------------------

        channel_list.list_state.select(Some(0));
        channel_list.filter(filter);

        // ------------------------------------------

        App {
            terminal,
            config: config.clone(),
            app_title: config.app_title,
            current_screen: Channels,
            channel_list,
            current_selected: 0,
            update_line: String::new(),
            filter,
            playback_history,
        }
    }

    #[doc = "Contains every possible action possible."]
    pub fn action(&mut self, action: Action) {
        match action {
            Mark | Unmark => {
                if self.current_screen == Videos {
                    let state = action == Mark;
                    match self.get_selected_video() {
                        Some(v) => v.mark(state),
                        None => return,
                    }

                    if !self.get_selected_channel().unwrap().has_new() && self.filter == Filter::OnlyNew {
                        self.action(Back);
                    } else if self.config.down_on_mark {
                        if let Some(e) = self.get_selected_channel() {
                            e.next();
                        }
                    }

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
            },
            Down => {
                match self.current_screen {
                    Channels => self.get_channel_list().next(),
                    Videos => if let Some(c) = self.get_selected_channel() {
                        c.next();
                    },
                }
            },
            Enter => {
                self.current_selected = match self.get_channel_list().list_state.selected() {
                    Some(selected) => selected,
                    None => return,
                };
                self.current_screen = Videos;
                /* self.get_channel_list().list_state.select(None); */
                if let Some(c) = self.get_selected_channel() {
                    c.list_state.select(Some(0));
                }
            },
            Back => {
                self.current_screen = Channels;
                let curr_sel = self.current_selected.clone();
                let len: usize = cmp::max(0, self.get_channel_list().channels.len() as isize - 1) as usize;
                let curr_sel = cmp::min(curr_sel, len);
                self.get_channel_list().list_state.select(Some(curr_sel));
            },
            NextChannel => {
                match self.current_screen {
                    Channels => {},
                    Videos => {
                        self.action(Back);
                        self.action(Down);
                        self.action(Enter);
                    }
                }
            }
            PrevChannel => {
                match self.current_screen {
                    Channels => {},
                    Videos => {
                        self.action(Back);
                        self.action(Up);
                        self.action(Enter);
                    }
                }
            }
            Open => {
                let video = match self.get_selected_video() {
                    Some(v) => v.clone(),
                    None => return
                };

                let channel = match self.get_selected_channel() {
                    Some(c) => c.name.clone(),
                    None => String::new(),
                };

                let history_video = video.to_minimal(channel);

                for i in 0..self.playback_history.len() {
                    if self.playback_history[i] == history_video {
                        self.playback_history.remove(i);
                        break
                    }
                }
                self.playback_history.push(history_video);

                write_playback_history(&self.playback_history);
                video.open();
            },
            Update => {
                draw(self)
            },
        }
    }

    #[doc = "Select the filter to use."]
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.set_channel_list(self.channel_list.clone());
    }

    #[doc = "Update the list of channels."]
    pub fn set_channel_list(&mut self, mut new_cl: ChannelList) {

        // apply current filter
        new_cl.filter(self.filter);

        // keep current selection based on currend focused screen
        match self.current_screen {
            Channels => {

                let len: usize = cmp::max(0, new_cl.channels.len() as isize - 1) as usize;
                let selected = self.channel_list.list_state.selected(); // type: Option<usize>

                self.channel_list = new_cl;

                let selected = cmp::min(selected, Some(len));
                self.channel_list.list_state.select(selected);
            },
            Videos => {
                let selected_video = match self.get_selected_channel() {
                    Some(c) => c.list_state.selected(),
                    None => return,
                };

                new_cl.list_state.select(None);
                self.channel_list = new_cl;
                
                match self.get_selected_channel() {
                    Some(c) => c.list_state.select(selected_video),
                    None => self.action(Back),
                };
            },
        }
    }
    pub fn get_channel_list(&mut self) -> &mut ChannelList {
        &mut self.channel_list
    }

    pub fn get_current_selected(&self) -> usize {
        self.current_selected
    }

    pub fn get_selected_video_link(&mut self) -> String {
        match self.get_selected_video() {
            Some(v) => v.link.clone(),
            None => String::from("none"),
        }
    }

    #[doc = "draw the screen."]
    pub fn update(&mut self) {
        draw(self);
    }

    //--------------

    #[doc = "returns the currently selected channel."]
    fn get_selected_channel(&mut self) -> Option<&mut Channel> {
        let i = self.current_selected;
        self.channel_list.channels.get_mut(i)
    }

    #[doc = "returns the currently selected Video."]
    fn get_selected_video(&mut self) -> Option<&mut Video> {
        let c = match self.get_selected_channel() {
            Some(c) => c,
            None => return None
        };
        let i = match c.list_state.selected() {
            Some(i) => i,
            None => return None
        };
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
