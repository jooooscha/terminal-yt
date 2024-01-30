use crate::{
    backend::{
        data::{channel::Channel, channel_list::ChannelList, video::Video},
        io::config::Config,
        io::{history::History, write_config, FileType::DbFile},
        Action,
        Action::*,
        Filter, Result, Screen,
        Screen::*,
        Terminal,
    },
    notification::{notify_error, notify_open},
};
use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
pub enum FetchState {
    DownloadsFailure(usize),
    Scheduled,
    Loading,
    FetchingDearrow,
    Fetched,
}

impl Default for FetchState {
    fn default() -> Self {
        Self::Scheduled
    }
}

#[derive(Clone)]
pub(crate) struct StateUpdate {
    text: String,
    state: self::FetchState,
}

impl StateUpdate {
    pub(crate) fn new(text: String, status: self::FetchState) -> Self {
        Self { text, state: status }
    }
}

// The main struct containing everything important
pub(crate) struct Core {
    pub(crate) terminal: Terminal,
    pub(crate) config: Config,
    pub(crate) current_screen: Screen,
    channel_list: ChannelList,
    pub(crate) playback_history: History,
}

impl Core {
    /// Load core
    pub(crate) fn load() -> Result<Self> {
        let terminal = Terminal::default();

        let config = Config::read()?;

        let current_filter = if config.show_empty_channels {
            Filter::NoFilter
        } else {
            Filter::OnlyNew
        };

        let mut channel_list = ChannelList::load().unwrap_or_else(|error| {
            notify_error(&format!("Could not load DB file: {:?}", error));
            ChannelList::default()
        });
        channel_list.select(Some(0));
        channel_list.set_filter(current_filter);

        let playback_history = History::load();

        let core = Core {
            terminal,
            config,
            current_screen: Channels,
            channel_list,
            playback_history,
        };

        Ok(core)
    }

    pub(crate) fn save(&mut self) {
        let string = serde_json::to_string(self.channel_list()).unwrap();
        write_config(DbFile, &string);
    }

    /// receive all status updates from status channel
    pub(crate) fn update_status_line(&mut self, item: StateUpdate) {
        if let Some(channel) = self.channel_list.get_unfiltered_mut_by_id(&item.text) {
            channel.fetch_state = item.state.clone();
        }
    }

    pub(crate) fn update_at_start(&self) -> bool {
        self.config.update_at_start
    }

    pub(crate) fn get_current_screen(&self) -> &Screen {
        &self.current_screen
    }

    // --- actions -----

    /// Contains every possible action.
    pub(crate) fn action(&mut self, action: Action) {
        let _ = || -> Option<()> {
            match action {
                Mark(state) => {
                    if self.current_screen == Videos {

                        let current_channel = self.get_selected_channel_mut()?;
                        let selected = current_channel.selected()?;

                        if let Some(video) = current_channel.get_mut(selected) {
                            video.mark(state);
                        }

                        let has_new = !current_channel.has_new();
                        let is_only_new = self.channel_list.get_filter() == Filter::OnlyNew;
                        if has_new && is_only_new {
                            self.action(Leave);
                        } else if self.config.down_on_mark {
                            self.get_selected_channel_mut()?.next();
                        }


                        // let pos = self.get_selected_channel_index();
                        let pos = self.get_selected_channel()?.selected();
                        self.save();
                        self.get_selected_channel_mut().as_mut()?.select(pos);
                    }
                }
                Up => match self.current_screen {
                    Channels => self.channel_list.prev(),
                    Videos => self.get_selected_channel_mut()?.prev(),
                },
                Down => match self.current_screen {
                    Channels => self.channel_list.next(),
                    Videos => self.get_selected_channel_mut()?.next(),
                },
                Enter => {
                    if self.get_selected_channel().is_some() {
                        self.get_selected_channel_mut().unwrap().select(Some(0));
                        self.current_screen = Videos;
                    }
                }
                Leave => {
                    self.current_screen = Channels;
                    let i = self.get_selected_channel_index();
                    self.channel_list.select(i);
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
                    let video = self.get_selected_video_mut()?.clone();

                    // mark video
                    if self.config.mark_on_open {
                        self.action(Mark(true));
                    }

                    // call video player
                    let command = Command::new("setsid")
                        .arg("-f")
                        .arg(&self.config.video_player)
                        .arg(video.link())
                        .stderr(Stdio::null())
                        .stdout(Stdio::null())
                        .spawn();

                    self.playback_history.add(video.clone());

                    match command {
                        Ok(_) => notify_open(&video.get_details()),
                        Err(error) => notify_error(&error.to_string()),
                    };
                }
            }
            None
        }();
    }

    // pub(crate) fn draw(&self) {
        // draw(self.into());
    // }

    pub fn toggle_filter(&mut self) {
        self.channel_list.toggle_filter();
    }

    /// Search for the channel in channel_list by id. If found insert videos that are not already in channel.videos; else insert channel to channel_list.
    pub(crate) fn update_channel(&mut self, updated_channel: Channel) {
        self.channel_list.update_channel(updated_channel, self.config.sort_channels);
    }

    pub(crate) fn get_selected_video_link(&mut self) -> String {
        match self.get_selected_video_mut() {
            Some(v) => v.link().clone(),
            None => String::from("none"),
        }
    }

    pub fn channel_list(&self) -> &ChannelList {
        &self.channel_list
    }

    pub fn channel_list_mut(&mut self) -> &mut ChannelList {
        &mut self.channel_list
    }

    pub(crate) fn get_selected_channel_index(&self) -> Option<usize> {
        self.channel_list.selected()
    }

    pub(crate) fn get_selected_channel(&self) -> Option<&Channel> {
        let i = self.get_selected_channel_index()?;
        self.channel_list.get(i)
    }

    pub(crate) fn get_selected_channel_mut(&mut self) -> Option<&mut Channel> {
        let i = self.get_selected_channel_index()?;
        self.channel_list.get_mut(i)
    }

    pub(crate) fn get_selected_video_mut(&mut self) -> Option<&mut Video> {
        let i = self.get_selected_channel()?.selected()?;
        self.get_selected_channel_mut()?.get_mut(i)
    }
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *     use crate::data::{
 *         channel::factory::ChannelFactory,
 *         video::factory::tests::{
 *             get_marked_video_factory, get_random_video_factory, get_unmarked_video_factory,
 *         },
 *     };
 *
 *     fn random_string() -> String {
 *         rand::thread_rng()
 *             .sample_iter(&Alphanumeric)
 *             .take(9)
 *             .map(char::from)
 *             .collect()
 *     }
 *
 *     fn test_core() -> Core {
 *         let channel_list: ChannelList = ChannelList::new();
 *         let mut c = Core::new_with_history();
 *
 *         c.channel_list = channel_list;
 *
 *         c
 *     }
 *
 *     fn draw(core: &mut Core, gui_mode: bool) {
 *         if gui_mode {
 *             core.draw();
 *             thread::sleep(time::Duration::from_millis(1000));
 *         }
 *     }
 *
 *     #[test]
 *     fn test_init() {
 *         let mut core = test_core();
 *
 *         const CHANNEL_COUNT: usize = 10;
 *
 *         let hidden_video_count = 5;
 *         let not_hidden_video_count = 30;
 *
 *         let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
 *             true, true, false, false, false, true, false, true, false, false,
 *         ];
 *         let _trues = 4;
 *         let _falses = 6;
 *
 *         for i in 0..CHANNEL_COUNT {
 *             let mut cf = ChannelFactory::test();
 *
 *             let mut videos = Vec::new();
 *             if channel_has_unmarked[i] {
 *                 videos.push(get_unmarked_video_factory());
 *                 for _ in 0..not_hidden_video_count - 1 {
 *                     videos.push(get_random_video_factory());
 *                 }
 *             } else {
 *                 for _ in 0..hidden_video_count - 1 {
 *                     videos.push(get_marked_video_factory());
 *                 }
 *             }
 *             cf.add_new_videos(videos);
 *
 *             let channel = cf.commit().unwrap();
 *
 *             core.update_channel(channel);
 *         }
 *
 *         core.set_filter(Filter::NoFilter);
 *
 *         assert_eq!(core.get_filtered_channel_list().len(), CHANNEL_COUNT);
 *
 *         core.set_filter(Filter::OnlyNew);
 *
 *         assert_eq!(core.get_filtered_channel_list().len(), _trues);
 *
 *         core.set_filter(Filter::NoFilter);
 *
 *         assert_eq!(core.get_filtered_channel_list().len(), CHANNEL_COUNT);
 *     }
 *
 *     #[test]
 *     fn test_move() {
 *         let channel_count = 10;
 *         let mut core = test_core();
 *
 *         for _ in 0..channel_count {
 *             let mut cf = ChannelFactory::test();
 *
 *             for _ in 0..10 {
 *                 cf.add_new_videos(vec![get_random_video_factory()]);
 *             }
 *
 *             let channel = cf.commit().unwrap();
 *             core.update_channel(channel);
 *         }
 *
 *         core.set_filter(Filter::NoFilter);
 *
 *         // simple down
 *         for _ in 0..3 {
 *             core.action(Down);
 *         }
 *
 *         assert_eq!(core.channel_list.selected().unwrap(), 3);
 *
 *         // simple up
 *         for _ in 0..2 {
 *             core.action(Up);
 *         }
 *
 *         assert_eq!(core.channel_list.selected().unwrap(), 1);
 *
 *         // too far up
 *         for _ in 0..5 {
 *             core.action(Up);
 *         }
 *
 *         assert_eq!(core.channel_list.selected().unwrap(), 0);
 *
 *         // too far down
 *         for _ in 0..channel_count + 1 {
 *             core.action(Down);
 *         }
 *
 *         assert_eq!(core.channel_list.selected().unwrap(), channel_count - 1);
 *     }
 *
 *     #[test]
 *     fn test_enter_leave() {
 *         let mut core = test_core();
 *
 *         const CHANNEL_COUNT: usize = 10;
 *         let hidden_video_count = 5;
 *         let not_hidden_video_count = 100;
 *
 *         let channel_has_unmarked: [bool; CHANNEL_COUNT] = [
 *             false, false, true, false, true, true, false, true, true, false,
 *         ];
 *         let _trues = 5;
 *         let _falses = 5;
 *
 *         for i in 0..CHANNEL_COUNT {
 *             let mut cf = ChannelFactory::test();
 *
 *             let mut videos = Vec::new();
 *             if channel_has_unmarked[i] {
 *                 videos.push(get_unmarked_video_factory());
 *                 for _ in 0..not_hidden_video_count - 1 {
 *                     videos.push(get_random_video_factory());
 *                 }
 *             } else {
 *                 for _ in 0..hidden_video_count - 1 {
 *                     videos.push(get_marked_video_factory());
 *                 }
 *             }
 *
 *             cf.add_new_videos(videos);
 *
 *             let channel = cf.commit().unwrap();
 *
 *             core.update_channel(channel);
 *         }
 *
 *         core.set_filter(Filter::NoFilter);
 *
 *         // --------------------------------------------------------------------------
 *
 *         assert_eq!(core.get_selected_channel_index(), 0);
 *
 *         core.action(Down);
 *         core.action(Down);
 *         core.action(Down);
 *
 *         assert_eq!(core.get_selected_channel_index(), 3);
 *
 *         core.action(Enter);
 *
 *         assert_eq!(core.get_selected_channel_index(), 3);
 *
 *         assert_eq!(core.current_screen, Screen::Videos);
 *
 *         core.action(Down);
 *         core.action(Down);
 *         core.action(Down);
 *         core.action(Up);
 *
 *         core.action(Leave);
 *
 *         assert_eq!(core.current_screen, Screen::Channels);
 *         assert_eq!(core.get_selected_channel_index(), 3);
 *     }
 *
 *     #[test]
 *     fn test_toggle_filter() {
 *         let mut core = test_core();
 *         let mut rng = thread_rng();
 *
 *         let gui_mode = match &env::args().collect::<Vec<String>>().get(2) {
 *             Some(text) => text.clone().clone() == "gui".to_owned(),
 *             None => false,
 *         };
 *
 *         const CHANNEL_COUNT: usize = 30;
 *         let hidden_video_count = 30;
 *         let not_hidden_video_count = 70;
 *
 *         let mut trues = 0;
 *         let mut falses = 0;
 *
 *         let mut channel_list = ChannelList::new();
 *
 *         for _ in 0..CHANNEL_COUNT {
 *             let mut cf = ChannelFactory::test();
 *             cf.set_name(random_string());
 *
 *             let mut videos = Vec::new();
 *             if rand::random() {
 *                 trues += 1;
 *                 videos.push(get_unmarked_video_factory());
 *                 for _ in 0..not_hidden_video_count - 1 {
 *                     videos.push(get_random_video_factory());
 *                 }
 *             } else {
 *                 falses += 1;
 *                 for _ in 0..hidden_video_count {
 *                     videos.push(get_marked_video_factory());
 *                 }
 *             }
 *
 *             cf.add_new_videos(videos);
 *
 *             let channel = cf.commit().unwrap();
 *
 *             channel_list.push(channel);
 *         }
 *
 *         core.set_channel_list(channel_list);
 *
 *         core.set_filter(Filter::NoFilter);
 *
 *         draw(&mut core, gui_mode);
 *
 *         //-------------------------------------------------------------------------------
 *
 *         assert_eq!(core.get_filtered_channel_list().len(), trues + falses);
 *         core.set_filter(Filter::OnlyNew);
 *         assert_eq!(core.get_filtered_channel_list().len(), trues);
 *
 *         draw(&mut core, gui_mode);
 *
 *         let number = rng.gen::<f32>() * 3.0;
 *         let number = number.floor() as usize + 1;
 *
 *         assert_eq!(core.get_selected_channel_index(), 0);
 *
 *         for _ in 0..number {
 *             core.action(Down);
 *         }
 *
 *         draw(&mut core, gui_mode);
 *
 *         assert_eq!(core.get_selected_channel_index(), number);
 *
 *         let channel_id = core.get_selected_channel().id().clone();
 *         core.set_filter(Filter::NoFilter);
 *
 *         draw(&mut core, gui_mode);
 *
 *         assert_eq!(core.get_filtered_channel_list().len(), trues + falses);
 *
 *         assert_eq!(core.get_selected_channel().id().clone(), channel_id);
 *
 *         // add one  marked channel at end
 *         let mut cf = ChannelFactory::test();
 *         cf.set_name("zzzzzzzzzzzz".to_owned());
 *         cf.add_new_videos(vec![get_marked_video_factory()]);
 *         let channel = cf.commit().unwrap();
 *
 *         core.update_channel(channel);
 *
 *         for _ in 0..100 {
 *             core.action(Down);
 *         }
 *
 *         draw(&mut core, gui_mode);
 *
 *         core.set_filter(Filter::OnlyNew);
 *
 *         draw(&mut core, gui_mode);
 *
 *         assert_eq!(
 *             core.get_filtered_channel_list().len() - 1,
 *             core.get_selected_channel_index()
 *         );
 *     }
 *
 *     #[test]
 *     fn test_marked() {
 *         let gui_mode = match &env::args().collect::<Vec<String>>().get(2) {
 *             Some(text) => text.clone().clone() == "gui".to_owned(),
 *             None => false,
 *         };
 *
 *         let mut core = test_core();
 *
 *         for _ in 0..10 {
 *             let mut cf = ChannelFactory::test();
 *
 *             let mut videos = Vec::new();
 *             for _ in 0..3 {
 *                 videos.push(get_unmarked_video_factory());
 *             }
 *
 *             cf.add_new_videos(videos);
 *
 *             let channel = cf.commit().unwrap();
 *
 *             core.update_channel(channel);
 *         }
 *
 *         // ---------------------------------------------------------
 *
 *         draw(&mut core, gui_mode);
 *
 *         core.action(Down);
 *         core.action(Down);
 *         core.action(Down);
 *
 *         draw(&mut core, gui_mode);
 *
 *         core.action(Enter);
 *
 *         draw(&mut core, gui_mode);
 *
 *         let channel_id = core.get_selected_channel_index();
 *
 *         println!("{}", channel_id);
 *
 *         for _ in 0..3 {
 *             core.action(Mark(true));
 *             draw(&mut core, gui_mode);
 *         }
 *
 *         draw(&mut core, gui_mode);
 *
 *         assert_eq!(channel_id, core.get_selected_channel_index());
 *     }
 * } */
