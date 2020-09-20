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

use fetch_data::*;

use Screens::*;

pub struct App<W: Write> {
    pub terminal: Terminal<TermionBackend<W>>,
    pub current_title: String,
}

enum Screens {
    Channels,
    Videos,
}

const TITLE: &str = "Terminal Youtube";

fn main() {
    let stdout = stdout().into_raw_mode().unwrap();
    let mouse_terminal = MouseTerminal::from(stdout);
    let screen = AlternateScreen::from(mouse_terminal);
    let stdin = stdin();
    let backend = TermionBackend::new(screen);
    let terminal = Terminal::new(backend).unwrap();

    let mut app = App {
        terminal,
        current_title: String::from(TITLE),
    };

    let mut channel_list = fetch_channel_list();

    channel_list.next(); // select first item
    channel_list.show(&mut app);

    let mut current_screen: Screens = Channels;
    /* let mut current_list: &Vec<T: ListMove> = Vec::new(); */

    for event in stdin.events() {
        match event.unwrap() {
            Event::Key(Key::Char('q')) => { // --------- close ---------------
                match current_screen {
                    Channels => {
                        break;
                    },
                    Videos => {
                        app.current_title = String::from(TITLE);
                        channel_list.show(&mut app);
                        current_screen = Channels;
                    },
                }
            },
            Event::Key(Key::Char('j')) | Event::Key(Key::Down) => {
                match current_screen {
                    Channels => {
                        channel_list.next();
                        channel_list.show(&mut app);
                    },
                    Videos => {
                        channel_list.get_selected().unwrap().next();
                        channel_list.get_selected().unwrap().show(&mut app);
                    }
                }
            },
            Event::Key(Key::Char('k')) | Event::Key(Key::Up) => {
                match current_screen {
                    Channels => {
                        channel_list.prev();
                        channel_list.show(&mut app);
                    },
                    Videos => {
                        channel_list.get_selected().unwrap().prev();
                        channel_list.get_selected().unwrap().show(&mut app);
                    }
                }
            },
            Event::Key(Key::Char('o')) => {  // ----------- open ---------------
                let video_list = channel_list.get_selected().unwrap();
                match current_screen {
                    Channels => {
                        app.current_title = video_list.name.clone();
                        video_list.show(&mut app);
                        current_screen = Videos;
                    },
                    Videos => {
                        video_list.get_selected().unwrap().open();
                    }
                }
            },
            Event::Key(Key::Char('m')) => { // ----------- mark ---------------
                match current_screen {
                    Channels => (),
                    Videos => {
                        let list = channel_list.get_selected().unwrap();
                        let item = list.get_selected().unwrap();
                        item.mark(true);
                        list.next();
                        list.show(&mut app); // redraw screen
                        channel_list.save(); // write changes
                    },
                }
            },
            Event::Key(Key::Char('M')) => { // ----------- unmark -------------
                match current_screen {
                    Channels => (),
                    Videos => {
                        let list = channel_list.get_selected().unwrap();
                        let item = list.get_selected().unwrap();
                        item.mark(false);
                        list.next();
                        list.show(&mut app); // redraw screen
                        channel_list.save(); // write changes
                    },
                }
            },
            Event::Key(Key::Char('R')) => {
                let _ = app.terminal.resize(app.terminal.size().unwrap());
                channel_list.show(&mut app);
            },
            _ => {}
        }
    }
}
