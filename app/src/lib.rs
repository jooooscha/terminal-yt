mod config;
mod draw;
pub mod fetch_data;
mod history;
mod url_file;
pub mod data_types {
    pub(crate) mod feed_types;
    pub mod internal;
}

use self::{Action::*, Screen::*};
use config::Config;
use data_types::internal::{Channel, ChannelList, Filter, MinimalVideo, Video};
use draw::draw;
use history::{read_history, read_playback_history, write_history, write_playback_history};
use std::{
    cmp,
    io::{stdin, stdout},
    process::{Command, Stdio},
    sync::mpsc::{channel, Receiver, Sender},
    /* time, */
};
use tui::{backend::TermionBackend, Terminal};

#[cfg(test)]
use rand::{distributions::Alphanumeric, prelude::*, Rng};
#[cfg(test)]
use std::env;
#[cfg(test)]
use std::{thread, time};
#[cfg(not(test))]
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

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
    pub(crate) update_line: String,
    /* pub msg_array: Vec<String>, */
    pub current_screen: Screen,
    pub current_filter: Filter,
    channel_list: ChannelList,
    pub(crate) playback_history: Vec<MinimalVideo>,
    pub status_sender: Sender<String>,
    pub(crate) status_receiver: Receiver<String>,
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
}

#[derive(PartialEq, Clone, Debug)]
pub enum Screen {
    Channels,
    Videos,
}

impl App {
    pub fn new_from_history() -> Self {
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

        let mut channel_list = read_history();
        let playback_history = read_playback_history();

        // ------------------------------------------

        channel_list.select(Some(0));
        channel_list.filter(current_filter, config.sort_by_tag);

        // ------------------------------------------

        let (status_sender, status_receiver) = channel();

        // ------------------------------------------

        App {
            terminal,
            config: config.clone(),
            current_screen: Channels,
            channel_list,
            update_line: String::new(),
            /* msg_array: Vec::new(), */
            current_filter,
            playback_history,
            status_sender,
            status_receiver,
        }
    }

    fn post(&mut self, _msg: String) {
        /* self.status_sender.send(msg); */
    }

    pub fn update_status_line(&mut self) -> bool {
        if let Ok(line) = self.status_receiver.try_recv() {
            self.update_line = line;
        } else if !self.update_line.is_empty() {
            self.update_line = String::new();
        } else {
            return false
        }

        true
    }

    #[doc = "Contains every possible action possible."]
    pub fn action(&mut self, action: Action) {
        match action {
            Mark | Unmark => {
                if self.current_screen == Videos {
                    let state = action == Mark;
                    match self.get_selected_video_mut() {
                        Some(video) => video.mark(state),
                        None => return,
                    }

                    if !self.get_selected_channel_mut().has_new()
                        && self.current_filter == Filter::OnlyNew
                    {
                        self.action(Leave);
                    } else if self.config.down_on_mark {
                        self.get_selected_channel_mut().next();
                    }

                    self.save();
                }
            }
            Up => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().prev(),
                Videos => {
                    self.get_selected_channel_mut().prev();
                }
            },
            Down => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().next(),
                Videos => self.get_selected_channel_mut().next(),
            },
            Enter => {
                /* notify_user(&format!("{}, {}", self.get_selected_channel_index(), self.get_selected_channel().id)); */
                self.current_screen = Videos;
                self.get_selected_channel_mut().select(Some(0));
            }
            Leave => {
                self.current_screen = Channels;
                let i = self.get_selected_channel_index();
                self.get_filtered_channel_list_mut().select(Some(i));
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
                let video = match self.get_selected_video_mut() {
                    Some(v) => v.clone(),
                    None => return,
                };

                let channel = self.get_selected_channel_mut().name.clone();

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
        }
    }

    pub fn draw(&mut self) {
        draw(self);
    }

    /// Set a filter
    pub fn set_filter(&mut self, filter: Filter) {
        self.current_filter = filter;
        self.set_channel_list(self.channel_list.clone());
    }

    fn set_channel_list(&mut self, mut new_channel_list: ChannelList) {
        if new_channel_list.len() == 0 {
            return;
        }

        // keep current selection based on currend focused screen
        let on_videos = self.current_screen == Videos;

        let mut video_pos = None;
        if on_videos {
            self.action(Leave);
            video_pos = self.get_selected_channel().selected();
        }

        let selected_channel_index = self.get_selected_channel_index();
        let selected_channel_id = if self.get_filtered_channel_list().len() > 0 {
            self.get_selected_channel_mut().id.clone()
        } else {
            String::new() // will not match later: intended
        };

        // apply current filter to new list
        new_channel_list.filter(self.current_filter, self.config.sort_by_tag);

        self.channel_list = new_channel_list;

        let position = self
            .get_filtered_channel_list()
            .get_position_by_id(&selected_channel_id);

        let selection = match position {
            Some(i) => i,
            None => {
                let l = cmp::max(1, self.get_filtered_channel_list().len());
                cmp::min(selected_channel_index, l - 1)
            }
        };

        #[cfg(test)]
        println!(
            "{:?}, selection: {}, selected_channel_index: {}",
            position, selection, selected_channel_index
        );

        self.channel_list.select(Some(selection));

        if on_videos && position.is_some() {
            self.action(Enter);
            self.get_selected_channel_mut().select(video_pos);
        }
    }

    /// Search for the channel in channel_list by id. If found insert videos that are not already in channel.videos; else insert channel to channel_list.
    pub fn update_channel(&mut self, updated_channel: Channel) {
        let mut channel_list = self.get_filtered_channel_list().clone();

        self.status_sender
            .send(format!("Ready: {}", &updated_channel.name))
            .unwrap();

        if let Some(channel) = channel_list.get_mut_by_id(&updated_channel.id) {
            channel.merge_videos(updated_channel); // add video to channel
        } else {
            channel_list.push(updated_channel); // insert new channel
        }

        self.set_channel_list(channel_list);
    }

    fn get_filtered_channel_list_mut(&mut self) -> &mut ChannelList {
        &mut self.channel_list
    }

    pub fn get_filtered_channel_list(&self) -> &ChannelList {
        &self.channel_list
    }

    pub fn get_selected_video_link(&mut self) -> String {
        match self.get_selected_video_mut() {
            Some(v) => v.link.clone(),
            None => String::from("none"),
        }
    }

    /* #[doc = "draw the screen."]
     * pub fn update(&mut self) {
     *     draw(self);
     * } */

    //--------------

    pub fn get_selected_channel_index(&self) -> usize {
        match self.get_filtered_channel_list().selected() {
            Some(i) => i,
            None => 0,
        }
    }

    fn get_selected_channel(&self) -> &Channel {
        let i = self.get_selected_channel_index();
        self.get_filtered_channel_list().get(i).unwrap()
    }

    fn get_selected_channel_mut(&mut self) -> &mut Channel {
        let i = self.get_selected_channel_index();
        self.get_filtered_channel_list_mut().get_mut(i).unwrap()
    }

    fn get_selected_video_mut(&mut self) -> Option<&mut Video> {
        let i = self.get_selected_channel().selected()?;
        self.get_selected_channel_mut().get_mut(i)
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

    fn get_random_video() -> Video {
        let mut rng = thread_rng();
        if rng.gen::<f64>() > 0.5 {
            get_unmarked_video()
        } else {
            get_unmarked_video()
        }
    }

    fn get_marked_video() -> Video {
        let mut video = get_unmarked_video();
        video.mark(true);

        video
    }

    fn get_unmarked_video() -> Video {
        let mut video = Video::new();
        video.link = random_string();

        video
    }

    fn random_string() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(9)
            .map(char::from)
            .collect()
    }

    fn test_app() -> App {
        let channel_list: ChannelList = ChannelList::new();
        App::new_from_channel_list(channel_list)
    }

    #[test]
    fn test_init() {
        let mut app = test_app();

        const CHANNEL_COUNT: usize = 10;

        let hidden_video_count = 5;
        let not_hidden_video_count = 30;

        let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
            true, true, false, false, false, true, false, true, false, false,
        ];
        let _trues = 4;
        let _falses = 6;

        for i in 0..CHANNEL_COUNT {
            let mut channel = Channel::new();
            channel.id = random_string();

            if channel_has_unmarked[i] {
                channel.push(get_unmarked_video());
                for _ in 0..not_hidden_video_count - 1 {
                    channel.push(get_random_video());
                }
            } else {
                for _ in 0..hidden_video_count - 1 {
                    channel.push(get_marked_video());
                }
            }

            app.update_channel(channel);
        }

        app.set_filter(Filter::NoFilter);

        assert_eq!(app.get_filtered_channel_list().len(), CHANNEL_COUNT);

        app.set_filter(Filter::OnlyNew);

        assert_eq!(app.get_filtered_channel_list().len(), _trues);

        app.set_filter(Filter::NoFilter);

        assert_eq!(app.get_filtered_channel_list().len(), CHANNEL_COUNT);
    }

    #[test]
    fn test_move() {
        let channel_count = 10;
        let mut app = test_app();

        for _ in 0..channel_count {
            let mut channel = Channel::new();
            channel.id = random_string();

            for _ in 0..10 {
                channel.push(get_random_video());
            }

            app.update_channel(channel);
        }

        app.set_filter(Filter::NoFilter);

        // simple down
        for _ in 0..3 {
            app.action(Down);
        }

        assert_eq!(app.channel_list.selected().unwrap(), 3);

        // simple up
        for _ in 0..2 {
            app.action(Up);
        }

        assert_eq!(app.channel_list.selected().unwrap(), 1);

        // too far up
        for _ in 0..5 {
            app.action(Up);
        }

        assert_eq!(app.channel_list.selected().unwrap(), 0);

        // too far down
        for _ in 0..channel_count + 1 {
            app.action(Down);
        }

        assert_eq!(app.channel_list.selected().unwrap(), channel_count - 1);
    }

    #[test]
    fn test_enter_leave() {
        let mut app = test_app();

        const CHANNEL_COUNT: usize = 10;
        let hidden_video_count = 5;
        let not_hidden_video_count = 100;

        let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
            false, false, true, false, true, true, false, true, true, false,
        ];
        let _trues = 5;
        let _falses = 5;

        for i in 0..CHANNEL_COUNT {
            let mut channel = Channel::new();
            channel.id = random_string();

            if channel_has_unmarked[i] {
                channel.push(get_unmarked_video());
                for _ in 0..not_hidden_video_count - 1 {
                    channel.push(get_random_video());
                }
            } else {
                for _ in 0..hidden_video_count - 1 {
                    channel.push(get_marked_video());
                }
            }

            app.update_channel(channel);
        }

        app.set_filter(Filter::NoFilter);

        // --------------------------------------------------------------------------

        assert_eq!(app.get_selected_channel_index(), 0);

        app.action(Down);
        app.action(Down);
        app.action(Down);

        assert_eq!(app.get_selected_channel_index(), 3);

        app.action(Enter);

        assert_eq!(app.get_selected_channel_index(), 3);

        assert_eq!(app.current_screen, Screen::Videos);

        app.action(Down);
        app.action(Down);
        app.action(Down);
        app.action(Up);

        app.action(Leave);

        assert_eq!(app.current_screen, Screen::Channels);
        assert_eq!(app.get_selected_channel_index(), 3);
    }

    #[test]
    fn test_toggle_filter() {
        let mut app = test_app();
        let mut rng = thread_rng();

        let gui_mode = match &env::args().collect::<Vec<String>>().get(2) {
            Some(text) => text.clone().clone() == "gui".to_owned(),
            None => false,
        };

        const CHANNEL_COUNT: usize = 30;
        let hidden_video_count = 30;
        let not_hidden_video_count = 70;

        let mut trues = 0;
        let mut falses = 0;

        let mut channel_list = ChannelList::new();

        for _ in 0..CHANNEL_COUNT {
            let mut channel = Channel::new();
            channel.id = random_string();
            channel.name = random_string();

            if rand::random() {
                trues += 1;
                channel.push(get_unmarked_video());
                for _ in 0..not_hidden_video_count - 1 {
                    channel.push(get_random_video());
                }
            } else {
                falses += 1;
                for _ in 0..hidden_video_count {
                    channel.push(get_marked_video());
                }
            }

            // app.update_channel_list(channel);
            channel_list.push(channel);
        }

        app.set_channel_list(channel_list);

        app.set_filter(Filter::NoFilter);

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        //-------------------------------------------------------------------------------

        assert_eq!(app.get_filtered_channel_list().len(), trues + falses);
        app.set_filter(Filter::OnlyNew);
        assert_eq!(app.get_filtered_channel_list().len(), trues);

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        let number = rng.gen::<f32>() * 3.0;
        let number = number.floor() as usize + 1;

        assert_eq!(app.get_selected_channel_index(), 0);

        for _ in 0..number {
            app.action(Down);
        }

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        assert_eq!(app.get_selected_channel_index(), number);

        let channel_id = app.get_selected_channel().id.clone();
        app.set_filter(Filter::NoFilter);

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        assert_eq!(app.get_filtered_channel_list().len(), trues + falses);

        assert_eq!(app.get_selected_channel().id.clone(), channel_id);

        // add one  marked channel at end
        let mut channel = Channel::new();
        channel.id = random_string();
        channel.name = "zzzzzzzzzzzzzzzzzzzz".to_owned();
        channel.push(get_marked_video());
        app.update_channel(channel);

        for _ in 0..100 {
            app.action(Down);
        }

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        app.set_filter(Filter::OnlyNew);

        if gui_mode {
            app.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }

        assert_eq!(
            app.get_filtered_channel_list().len() - 1,
            app.get_selected_channel_index()
        );
    }
}
