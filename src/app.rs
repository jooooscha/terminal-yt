use crate::draw;
use data::{config::Config, history::*};
use data_types::internal::{Filter, *};
#[cfg(test)]
use rand::prelude::*;
use std::{
    io::{stdin, stdout},
    process::{Command, Stdio},
};
#[cfg(not(test))]
use termion::raw::IntoRawMode;
use termion::{input::MouseTerminal, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};
use Action::*;
use Screen::*;

// The main struct containing everything important
pub struct App {
    #[cfg(not(test))]
    pub terminal: Terminal<
        TermionBackend<
            termion::screen::AlternateScreen<
                termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
            >,
        >,
    >,
    #[cfg(test)]
    pub terminal: Terminal<
        TermionBackend<
            termion::screen::AlternateScreen<termion::input::MouseTerminal<std::io::Stdout>>,
        >,
    >,
    pub config: Config,
    pub update_line: String,
    pub msg_array: Vec<String>,
    pub app_title: String,
    pub current_screen: Screen,
    pub current_filter: Filter,
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
    Leave,
    NextChannel,
    PrevChannel,
    Open,
    Update,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Screen {
    Channels,
    Videos,
}

impl App {
    pub fn new_from_channel_list(mut channel_list: ChannelList) -> Self {
        #[cfg(not(test))]
        let stdout = stdout().into_raw_mode().unwrap();
        #[cfg(test)]
        let stdout = stdout();
        let mouse_terminal = MouseTerminal::from(stdout);
        /* let screen = mouse_terminal; */
        let screen = AlternateScreen::from(mouse_terminal);
        let _stdin = stdin();
        let backend = TermionBackend::new(screen);
        let terminal = Terminal::new(backend).unwrap();

        // ------------------------------------------
        let config = Config::read_config_file();

        let current_filter = if config.show_empty_channels {
            Filter::NoFilter
        } else {
            Filter::OnlyNew
        };

        // ------------------------------------------

        let playback_history = read_playback_history();

        // ------------------------------------------

        channel_list.list_state.select(Some(0));
        channel_list.filter(current_filter, config.sort_by_tag);

        // ------------------------------------------

        App {
            terminal,
            config: config.clone(),
            app_title: config.app_title,
            current_screen: Channels,
            channel_list,
            update_line: String::new(),
            msg_array: Vec::new(),
            current_filter,
            playback_history,
        }
    }

    fn post(&mut self, msg: String) {
        self.msg_array.push(msg);
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

                    if !self.get_selected_channel().has_new()
                        && self.current_filter == Filter::OnlyNew
                    {
                        self.action(Leave);
                    } else if self.config.down_on_mark {
                        self.get_selected_channel().next();
                    }

                    self.save();
                }
            }
            Up => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().prev(),
                Videos => {
                    self.get_selected_channel().prev();
                }
            },
            Down => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().next(),
                Videos => self.get_selected_channel().next(),
            },
            Enter => {
                self.current_screen = Videos;
                self.get_selected_channel().list_state.select(Some(0));
            }
            Leave => {
                self.current_screen = Channels;
                let i = self.get_selected_channel_index();
                self.get_filtered_channel_list_mut()
                    .list_state
                    .select(Some(i));
            }
            NextChannel => match self.current_screen {
                Channels => {}
                Videos => {
                    self.action(Leave);
                    self.action(Down);
                    self.action(Enter);
                }
            },
            PrevChannel => match self.current_screen {
                Channels => {}
                Videos => {
                    self.action(Leave);
                    self.action(Up);
                    self.action(Enter);
                }
            },
            Open => {
                let video = match self.get_selected_video() {
                    Some(v) => v.clone(),
                    None => return,
                };

                let channel = self.get_selected_channel().name.clone();

                let history_video = video.to_minimal(channel);

                for i in 0..self.playback_history.len() {
                    if self.playback_history[i] == history_video {
                        self.playback_history.remove(i);
                        break;
                    }
                }
                self.playback_history.push(history_video);

                write_playback_history(&self.playback_history);
                if let Err(err) = video.open() {
                    self.post(err.to_string());
                };
                if self.config.use_notify_send {
                    if let Err(err) = Command::new("notify-send")
                        .arg("Open video")
                        .arg(video.title)
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        self.post(format!("Could not start notify-send: {}", err))
                    };
                }
            }
            Update => draw(self),
        }
    }

    #[doc = "Select a filter."]
    pub fn set_filter(&mut self, filter: Filter) {
        self.current_filter = filter;
        self.set_channel_list(self.channel_list.clone());
    }

    fn set_channel_list(&mut self, new_cl: ChannelList) {
        let mut filtered_channel_list = new_cl.clone();

        // apply current filter
        filtered_channel_list.filter(self.current_filter, self.config.sort_by_tag);

        // keep current selection based on currend focused screen
        match self.current_screen {
            Channels => {
                let selected_channel = self.get_selected_channel_index();

                self.channel_list = filtered_channel_list;

                self.channel_list.list_state.select(Some(selected_channel));
            }
            Videos => {
                let selected_video = self.get_selected_channel().list_state.selected();

                self.channel_list = filtered_channel_list;

                self.get_selected_channel()
                    .list_state
                    .select(selected_video);
            }
        }
    }

    #[doc = "Update the list of channels."]
    pub fn update_channel_list(&mut self, updated_channel: Channel) {
        let mut channel_list = self.get_filtered_channel_list().clone();

        let position: Option<usize> = channel_list
            .channels
            .iter()
            .position(|channel| channel.id == updated_channel.id);

        match position {
            Some(i) => channel_list.channels[i] = updated_channel,
            None => channel_list.channels.push(updated_channel),
        }

        self.set_channel_list(channel_list);
    }

    fn get_filtered_channel_list_mut(&mut self) -> &mut ChannelList {
        &mut self.channel_list
    }

    pub fn get_filtered_channel_list(&self) -> &ChannelList {
        &self.channel_list
    }

    pub fn get_selected_channel_index(&self) -> usize {
        match self.get_filtered_channel_list().list_state.selected() {
            Some(i) => i,
            None => 0,
        }
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

    fn get_selected_channel(&mut self) -> &mut Channel {
        let i = self.get_selected_channel_index();
        self.channel_list.channels.get_mut(i).unwrap()
    }

    fn get_selected_video(&mut self) -> Option<&mut Video> {
        let c = self.get_selected_channel();
        let i = match c.list_state.selected() {
            Some(i) => i,
            None => return None,
        };
        c.videos.get_mut(i)
    }
    //---------------
    pub fn save(&mut self) {
        let f = self.current_filter;
        self.set_filter(Filter::NoFilter);
        write_history(self.get_filtered_channel_list());
        self.set_filter(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(channel_count: usize) -> App {
        let mut rng = rand::thread_rng();
        let channel_list: ChannelList = ChannelList::new();

        let mut app = App::new_from_channel_list(channel_list);

        for i in 0..channel_count {
            let mut test_channel: Channel = Channel::new();
            test_channel.id = format!("channel {}", i);

            if i % 2 == 0 {
                let mut video = Video::new();
                if rng.gen::<f64>() > 0.0 {
                    video.mark(true);
                };
                test_channel.videos.push(Video::new());
            }
            app.update_channel_list(test_channel);
        }

        app
    }

    #[test]
    fn test_init() {
        let channel_count = 10;
        let mut app = test_app(channel_count);

        app.set_filter(Filter::NoFilter);

        assert_eq!(
            app.get_filtered_channel_list().channels.len(),
            channel_count
        );
    }

    #[test]
    fn test_move() {
        let channel_count = 10;
        let mut app = test_app(channel_count);

        app.set_filter(Filter::NoFilter);

        // simple down
        for _ in 0..3 {
            app.action(Down);
        }

        assert_eq!(app.channel_list.list_state.selected().unwrap(), 3);

        // simple up
        for _ in 0..2 {
            app.action(Up);
        }

        assert_eq!(app.channel_list.list_state.selected().unwrap(), 1);

        // too far up
        for _ in 0..5 {
            app.action(Up);
        }

        assert_eq!(app.channel_list.list_state.selected().unwrap(), 0);

        // too far down
        for _ in 0..channel_count + 1 {
            app.action(Down);
        }

        assert_eq!(
            app.channel_list.list_state.selected().unwrap(),
            channel_count - 1
        );
    }

    #[test]
    fn test_enter_leave() {
        let channel_count = 100;
        let mut app = test_app(channel_count);

        assert_eq!(app.get_selected_channel_index(), 0);

        app.action(Down);
        app.action(Down);
        app.action(Down);

        assert_eq!(app.get_selected_channel_index(), 3);

        app.action(Enter);

        assert_eq!(app.current_screen, Screen::Videos);

        app.action(Down);
        app.action(Down);
        app.action(Down);

        app.action(Leave);

        assert_eq!(app.current_screen, Screen::Channels);
        assert_eq!(app.get_selected_channel_index(), 3);

        app.set_filter(Filter::OnlyNew);

        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Down);

        let channel_id = app.get_selected_channel().id.clone();

        app.set_filter(Filter::NoFilter);

        assert_eq!(app.get_selected_channel().id.clone(), channel_id);
    }
}
