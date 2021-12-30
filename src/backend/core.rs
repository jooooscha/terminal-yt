use crate::backend::{
    config::Config,
    data::{channel::Channel, channel_list::ChannelList, video::Video},
    draw::draw,
    history::{
        read_history, read_playback_history, write_history, write_playback_history, MinimalVideo,
    },
    Action,
    Action::*,
    Filter, Screen,
    Screen::*,
    Terminal,
};
use std::{
    cmp,
    process::{Command, Stdio},
    sync::mpsc::{channel, Receiver, Sender},
};

/* #[cfg(not(test))]
 * use termion::raw::IntoRawMode; */

// The main struct containing everything important
pub struct Core {
    pub(crate) terminal: Terminal,
    pub(crate) config: Config,
    pub(crate) update_line: String,
    pub(crate) current_screen: Screen,
    pub(crate) current_filter: Filter,
    channel_list: ChannelList,
    pub(crate) playback_history: Vec<MinimalVideo>,
    pub(crate) status_sender: Sender<String>,
    pub(crate) status_receiver: Receiver<String>,
}

impl Core {
    pub fn new_with_history() -> Self {
        let terminal = Terminal::default();

        // ------------------------------------------
        let config = Config::init();

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

        Core {
            terminal,
            config,
            current_screen: Channels,
            channel_list,
            update_line: String::new(),
            current_filter,
            playback_history,
            status_sender,
            status_receiver,
        }
    }

    pub fn post(&mut self, msg: String) {
        self.status_sender.send(msg).unwrap();
    }

    // gettter and setter

    pub fn update_status_line(&mut self) -> bool {
        if let Ok(line) = self.status_receiver.try_recv() {
            self.update_line = line;
        } else if !self.update_line.is_empty() {
            self.update_line = String::new();
        } else {
            return false;
        }
        true
    }

    pub fn get_show_empty(&self) -> bool {
        self.config.show_empty_channels
    }
    pub fn set_show_empty(&mut self, b: bool) {
        self.config.show_empty_channels = b;

        let new_filter = match self.current_filter {
            Filter::NoFilter => Filter::OnlyNew,
            Filter::OnlyNew => Filter::NoFilter,
        };
        self.set_filter(new_filter);
    }

    pub fn update_at_start(&self) -> bool {
        self.config.update_at_start
    }

    pub fn get_current_screen(&self) -> &Screen {
        &self.current_screen
    }

    // --- actions -----

    /// Contains every possible action.
    pub fn action(&mut self, action: Action) -> Option<String> {
        match action {
            Mark(state) => {
                if self.current_screen == Videos {
                    if let Some(video) = self.get_selected_video_mut() {
                        video.mark(state);
                    }

                    if !self.get_selected_channel_mut().has_new() && self.current_filter == Filter::OnlyNew {
                        self.action(Leave);
                    } else if self.config.down_on_mark {
                        self.get_selected_channel_mut().next();
                    }

                    let pos = self.get_selected_channel_index();
                    self.save();
                    self.select(pos);
                }
            }
            Up => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().prev(),
                Videos => self.get_selected_channel_mut().prev()
            },
            Down => match self.current_screen {
                Channels => self.get_filtered_channel_list_mut().next(),
                Videos => self.get_selected_channel_mut().next(),
            },
            Enter => {
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
            SetVideoFav => {
                if let Some(ref mut video) = self.get_selected_video_mut() {
                    video.set_fav(!video.is_fav());
                    self.save();
                }
            }
            Open => {
                // get video
                let video = match self.get_selected_video_mut() {
                    Some(v) => v.clone(),
                    None => return None,
                };

                // mark video
                if self.config.mark_on_open {
                    self.action(Mark(true));
                }

                // for playback history
                let mimimal_video = MinimalVideo::from(video.clone());

                for i in 0..self.playback_history.len() {
                    if self.playback_history[i] == mimimal_video {
                        self.playback_history.remove(i);
                        break;
                    }
                }
                self.playback_history.push(mimimal_video);

                write_playback_history(&self.playback_history);

                // call video player

                if let Err(error) = Command::new("setsid")
                    .arg("-f")
                    .arg(&self.config.video_player)
                    .arg(video.link())
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .spawn()
                {
                    self.post(error.to_string());
                }
                return Some(video.get_details())
            }
        }
        None
    }

    pub fn draw(&self) {
        draw(self.into());
    }

    /// Set a filter
    fn set_filter(&mut self, filter: Filter) {
        self.current_filter = filter;
        self.set_channel_list(self.channel_list.clone());
    }

    fn set_channel_list(&mut self, mut new_channel_list: ChannelList) {
        if new_channel_list.len() == 0 {
            return;
        }

        // remember selected screen
        let on_videos = self.current_screen == Videos;

        let mut video_pos = None;
        if on_videos {
            self.action(Leave);
            video_pos = self.get_selected_channel().selected();
        }

        let selected_channel_index = self.get_selected_channel_index();
        let selected_channel_id = if self.get_filtered_channel_list().len() > 0 {
            self.get_selected_channel_mut().id().clone()
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

    pub fn select(&mut self, p: usize) {
        let pos = cmp::min(self.channel_list.len()-1, p);
        self.channel_list.select(Some(pos));
    }

    /// Search for the channel in channel_list by id. If found insert videos that are not already in channel.videos; else insert channel to channel_list.
    pub fn update_channel(&mut self, updated_channel: Channel) {
        let mut channel_list = self.get_filtered_channel_list().clone();

        self.post(format!("Updated: {}", &updated_channel.name()));

        if let Some(channel) = channel_list.get_mut_by_id(updated_channel.id()) {
            channel.merge_videos(updated_channel.videos); // add video to channel
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
            Some(v) => v.link().clone(),
            None => String::from("none"),
        }
    }

    pub fn get_selected_channel_index(&self) -> usize {
        self.get_filtered_channel_list().selected().unwrap_or(0)
    }

    pub fn get_selected_channel(&self) -> &Channel {
        let i = self.get_selected_channel_index();
        self.get_filtered_channel_list().get(i).unwrap()
    }

    pub fn get_selected_channel_mut(&mut self) -> &mut Channel {
        let i = self.get_selected_channel_index();
        self.get_filtered_channel_list_mut().get_mut(i).unwrap()
    }

    /* pub fn get_selected_video(&self) -> Option<&Video> {
     *     let i = self.get_selected_channel().selected()?;
     *     self.get_selected_channel().get(i)
     * } */

    pub fn get_selected_video_mut(&mut self) -> Option<&mut Video> {
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
    use crate::data::{
        channel::factory::ChannelFactory,
        video::factory::tests::{
            get_marked_video_factory, get_random_video_factory, get_unmarked_video_factory,
        },
    };

    fn random_string() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(9)
            .map(char::from)
            .collect()
    }

    fn test_core() -> Core {
        let channel_list: ChannelList = ChannelList::new();
        let mut c = Core::new_with_history();

        c.channel_list = channel_list;

        c
    }

    fn draw(core: &mut Core, gui_mode: bool) {
        if gui_mode {
            core.draw();
            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    #[test]
    fn test_init() {
        let mut core = test_core();

        const CHANNEL_COUNT: usize = 10;

        let hidden_video_count = 5;
        let not_hidden_video_count = 30;

        let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
            true, true, false, false, false, true, false, true, false, false,
        ];
        let _trues = 4;
        let _falses = 6;

        for i in 0..CHANNEL_COUNT {
            let mut cf = ChannelFactory::test();

            let mut videos = Vec::new();
            if channel_has_unmarked[i] {
                videos.push(get_unmarked_video_factory());
                for _ in 0..not_hidden_video_count - 1 {
                    videos.push(get_random_video_factory());
                }
            } else {
                for _ in 0..hidden_video_count - 1 {
                    videos.push(get_marked_video_factory());
                }
            }
            cf.add_new_videos(videos);

            let channel = cf.commit().unwrap();

            core.update_channel(channel);
        }

        core.set_filter(Filter::NoFilter);

        assert_eq!(core.get_filtered_channel_list().len(), CHANNEL_COUNT);

        core.set_filter(Filter::OnlyNew);

        assert_eq!(core.get_filtered_channel_list().len(), _trues);

        core.set_filter(Filter::NoFilter);

        assert_eq!(core.get_filtered_channel_list().len(), CHANNEL_COUNT);
    }

    #[test]
    fn test_move() {
        let channel_count = 10;
        let mut core = test_core();

        for _ in 0..channel_count {
            let mut cf = ChannelFactory::test();

            for _ in 0..10 {
                cf.add_new_videos(vec![get_random_video_factory()]);
            }

            let channel = cf.commit().unwrap();
            core.update_channel(channel);
        }

        core.set_filter(Filter::NoFilter);

        // simple down
        for _ in 0..3 {
            core.action(Down);
        }

        assert_eq!(core.channel_list.selected().unwrap(), 3);

        // simple up
        for _ in 0..2 {
            core.action(Up);
        }

        assert_eq!(core.channel_list.selected().unwrap(), 1);

        // too far up
        for _ in 0..5 {
            core.action(Up);
        }

        assert_eq!(core.channel_list.selected().unwrap(), 0);

        // too far down
        for _ in 0..channel_count + 1 {
            core.action(Down);
        }

        assert_eq!(core.channel_list.selected().unwrap(), channel_count - 1);
    }

    #[test]
    fn test_enter_leave() {
        let mut core = test_core();

        const CHANNEL_COUNT: usize = 10;
        let hidden_video_count = 5;
        let not_hidden_video_count = 100;

        let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
            false, false, true, false, true, true, false, true, true, false,
        ];
        let _trues = 5;
        let _falses = 5;

        for i in 0..CHANNEL_COUNT {
            let mut cf = ChannelFactory::test();

            let mut videos = Vec::new();
            if channel_has_unmarked[i] {
                videos.push(get_unmarked_video_factory());
                for _ in 0..not_hidden_video_count - 1 {
                    videos.push(get_random_video_factory());
                }
            } else {
                for _ in 0..hidden_video_count - 1 {
                    videos.push(get_marked_video_factory());
                }
            }

            cf.add_new_videos(videos);

            let channel = cf.commit().unwrap();

            core.update_channel(channel);
        }

        core.set_filter(Filter::NoFilter);

        // --------------------------------------------------------------------------

        assert_eq!(core.get_selected_channel_index(), 0);

        core.action(Down);
        core.action(Down);
        core.action(Down);

        assert_eq!(core.get_selected_channel_index(), 3);

        core.action(Enter);

        assert_eq!(core.get_selected_channel_index(), 3);

        assert_eq!(core.current_screen, Screen::Videos);

        core.action(Down);
        core.action(Down);
        core.action(Down);
        core.action(Up);

        core.action(Leave);

        assert_eq!(core.current_screen, Screen::Channels);
        assert_eq!(core.get_selected_channel_index(), 3);
    }

    #[test]
    fn test_toggle_filter() {
        let mut core = test_core();
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
            let mut cf = ChannelFactory::test();
            cf.set_name(random_string());

            let mut videos = Vec::new();
            if rand::random() {
                trues += 1;
                videos.push(get_unmarked_video_factory());
                for _ in 0..not_hidden_video_count - 1 {
                    videos.push(get_random_video_factory());
                }
            } else {
                falses += 1;
                for _ in 0..hidden_video_count {
                    videos.push(get_marked_video_factory());
                }
            }

            cf.add_new_videos(videos);

            let channel = cf.commit().unwrap();

            channel_list.push(channel);
        }

        core.set_channel_list(channel_list);

        core.set_filter(Filter::NoFilter);

        draw(&mut core, gui_mode);

        //-------------------------------------------------------------------------------

        assert_eq!(core.get_filtered_channel_list().len(), trues + falses);
        core.set_filter(Filter::OnlyNew);
        assert_eq!(core.get_filtered_channel_list().len(), trues);

        draw(&mut core, gui_mode);

        let number = rng.gen::<f32>() * 3.0;
        let number = number.floor() as usize + 1;

        assert_eq!(core.get_selected_channel_index(), 0);

        for _ in 0..number {
            core.action(Down);
        }

        draw(&mut core, gui_mode);

        assert_eq!(core.get_selected_channel_index(), number);

        let channel_id = core.get_selected_channel().id().clone();
        core.set_filter(Filter::NoFilter);

        draw(&mut core, gui_mode);

        assert_eq!(core.get_filtered_channel_list().len(), trues + falses);

        assert_eq!(core.get_selected_channel().id().clone(), channel_id);

        // add one  marked channel at end
        let mut cf = ChannelFactory::test();
        cf.set_name("zzzzzzzzzzzz".to_owned());
        cf.add_new_videos(vec![get_marked_video_factory()]);
        let channel = cf.commit().unwrap();

        core.update_channel(channel);

        for _ in 0..100 {
            core.action(Down);
        }

        draw(&mut core, gui_mode);

        core.set_filter(Filter::OnlyNew);

        draw(&mut core, gui_mode);

        assert_eq!(
            core.get_filtered_channel_list().len() - 1,
            core.get_selected_channel_index()
        );
    }

    #[test]
    fn test_marked() {
        let gui_mode = match &env::args().collect::<Vec<String>>().get(2) {
            Some(text) => text.clone().clone() == "gui".to_owned(),
            None => false,
        };

        let mut core = test_core();

        for _ in 0..10 {
            let mut cf = ChannelFactory::test();

            let mut videos = Vec::new();
            for _ in 0..3 {
                videos.push(get_unmarked_video_factory());
            }

            cf.add_new_videos(videos);

            let channel = cf.commit().unwrap();

            core.update_channel(channel);
        }

        // ---------------------------------------------------------

        draw(&mut core, gui_mode);

        core.action(Down);
        core.action(Down);
        core.action(Down);

        draw(&mut core, gui_mode);

        core.action(Enter);

        draw(&mut core, gui_mode);

        let channel_id = core.get_selected_channel_index();

        println!("{}", channel_id);

        for _ in 0..3 {
            core.action(Mark(true));
            draw(&mut core, gui_mode);
        }

        draw(&mut core, gui_mode);

        assert_eq!(channel_id, core.get_selected_channel_index());
    }
}
