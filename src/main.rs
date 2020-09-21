use std::io::{stdout, stdin, Write};
use termion::{
    raw::IntoRawMode,
    screen::AlternateScreen,
    input::{MouseTerminal, TermRead},
    event::{Key, Event},
};
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Block, Borders, List, ListItem},
};

mod draw;
mod fetch_data;
use draw::*;

use fetch_data::*;

use Screen::*;

pub struct App<W: Write> {
    pub terminal: Terminal<TermionBackend<W>>,
    current_screen: Screen,
    pub app_title: String,
    pub all_channels: ChannelList,
    pub current_selected: usize,
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
    let stdin = stdin();
    let backend = TermionBackend::new(screen);
    let terminal = Terminal::new(backend).unwrap();

    let mut app = App {
        terminal,
        app_title: String::from(TITLE),
        current_screen: Channels,
        all_channels: fetch_channel_list(),
        current_selected: 0,
    };

    app.update();

    for event in stdin.events() {
        match event.unwrap() {
            Event::Key(Key::Char('q')) => { // --------- close ---------------
                match app.current_screen {
                    Channels => {
                        break;
                    },
                    Videos => {
                        app.close_right_block();
                    },
                }
            },
            Event::Key(Key::Esc) => {
                match app.current_screen {
                    Channels => {},
                    Videos => {
                        app.close_right_block();
                    }
                }
            }
            Event::Key(Key::Char('j')) | Event::Key(Key::Down) => {
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
            Event::Key(Key::Char('k')) | Event::Key(Key::Up) => {
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
            Event::Key(Key::Char('\n')) => {  // ----------- open ---------------
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
            Event::Key(Key::Char('o')) => {
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
            Event::Key(Key::Char('m')) => { // ----------- mark ---------------
                match app.current_screen {
                    Channels => (),
                    Videos => {
                        app.get_selected_video().mark(true);
                        app.get_selected_channel().next();
                        app.update();
                    },
                }
            },
            Event::Key(Key::Char('M')) => { // ----------- unmark -------------
                match app.current_screen {
                    Channels => (),
                    Videos => {
                        app.get_selected_video().mark(false);
                        app.get_selected_channel().next();
                        app.update();
                    },
                }
            },
            Event::Key(Key::Char('R')) => {
                let _ = app.terminal.autoresize();
                app.update();
            },
            _ => {}
        }
    }
}
