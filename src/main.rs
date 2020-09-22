use std::{
    io::{stdout, stdin, Write},
    thread,
    sync::mpsc::{
        channel,
    }
};
use termion::{
    raw::IntoRawMode,
    screen::AlternateScreen,
    input::MouseTerminal,
    event::Key,
};
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Block, Borders, List, ListItem},
};
use Screen::*;
use fetch_data::{
    structs::{
        ChannelList,
        Channel,
        VideoItem,
    },
    fetch_data::{
        fetch_new_videos,
        fetch_history_videos,
        write_history,
    },
};

mod draw;
use draw::draw;

mod events;
use events::*;

pub struct App<W: Write> {
    pub terminal: Terminal<TermionBackend<W>>,
    current_screen: Screen,
    pub app_title: String,
    pub all_channels: ChannelList,
    pub current_selected: usize,
    pub update_line: String,
}

impl<W: Write> App<W> {
    fn update(&mut self) {
        draw(self);
    }
    fn get_selected_channel(&mut self) -> &mut Channel {
        let i = self.current_selected;
        &mut self.all_channels.channels[i]
    }
    fn get_selected_video(&mut self) -> &mut VideoItem {
        let c = self.get_selected_channel();
        let i = c.list_state.selected().unwrap();
        &mut c.videos[i]
    }
    fn close_right_block(&mut self) {
        self.current_screen = Channels;
        self.all_channels.list_state.select(Some(self.current_selected));
        self.update();
    }
    fn save(&self) {
        self.all_channels.save();
    }
}

#[derive(PartialEq, Clone)]
enum Screen {
    Channels,
    Videos,
}


const TITLE: &str = "Terminal-Youtube";

fn main() {
    let stdout = stdout().into_raw_mode().unwrap();
    let mouse_terminal = MouseTerminal::from(stdout);
    /* let screen = mouse_terminal; */
    let screen = AlternateScreen::from(mouse_terminal);
    let _stdin = stdin();
    let backend = TermionBackend::new(screen);
    let terminal = Terminal::new(backend).unwrap();

    let mut app = App {
        terminal,
        app_title: String::from(TITLE),
        current_screen: Channels,
        all_channels: fetch_history_videos(),
        current_selected: 0,
        update_line: String::new(),
    };

    let events = Events::new();

    let (mess, mesr) = channel();
    let (update_sender, update_receiver) = channel();

    thread::spawn(move|| {
        let new_chan = fetch_new_videos(update_sender);
        mess.send(new_chan.clone()).unwrap();
        write_history(&new_chan);
    });

    let mut update = true;

    loop {
        let event = events.next();

        if update {
            match mesr.try_recv() {
                Ok(v) => {
                    app.all_channels = v;
                    update = false;
                },
                Err(_) => {}
            }
        }

        match event.unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    match app.current_screen {
                        Channels => {
                            break;
                        },
                        Videos => {
                            app.close_right_block();
                        },
                    }
                },
              Key::Esc => {
                  match app.current_screen {
                      Channels => {},
                      Videos => {
                          app.close_right_block();
                      }
                  }
              }
              Key::Char('j') | Key::Down => {
                  match app.current_screen {
                      Channels => {
                          app.all_channels.next();
                      },
                      Videos => {
                          app.get_selected_channel().next();
                      }
                  }
                  app.update();
              },
              Key::Char('k') | Key::Up => {
                  match app.current_screen {
                      Channels => {
                          app.all_channels.prev();
                      },
                      Videos => {
                          app.get_selected_channel().prev();
                      }
                  }
                  app.update();
              },
              Key::Char('\n') => {  // ----------- open ---------------
                  match app.current_screen {
                      Channels => {
                          app.current_selected = app.all_channels.list_state.selected().unwrap();
                          app.current_screen = Videos;
                          app.all_channels.list_state.select(None);
                          app.update()
                      },
                      Videos => {}
                  }
              },
              Key::Char('o') => {
                  match app.current_screen {
                      Channels => {
                          app.current_selected = app.all_channels.list_state.selected().unwrap();
                          app.current_screen = Videos;
                          app.all_channels.list_state.select(None);
                      },
                      Videos => {
                          app.get_selected_video().open();
                      },
                  }
                  app.update();
              }
              Key::Char('m') => { // ----------- mark ---------------
                  match app.current_screen {
                      Channels => (),
                      Videos => {
                          app.get_selected_video().mark(true);
                          app.get_selected_channel().next();
                          app.update();
                          app.save();
                      },
                  }
              },
              Key::Char('M') => { // ----------- unmark -------------
                  match app.current_screen {
                      Channels => (),
                      Videos => {
                          app.get_selected_video().mark(false);
                          app.get_selected_channel().next();
                          app.update();
                          app.save();
                      },
                  }
              },
                _ => {}
            }
            Event::Tick => {
                app.update_line = match update_receiver.try_recv() {
                    Ok(v) => v,
                    Err(_) => String::new(),
                };
                app.update();
            }
        }
    }
}
