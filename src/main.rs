use std::{
    thread,
    sync::mpsc::{
        channel,
        Sender,
    }
};
use tui::widgets::{Block, Borders, List, ListItem};
use termion::event::Key;
use Screen::*;
use fetch_data::{
    fetch_data::{
        fetch_new_videos,
        read_history,
        write_history,
    },
};
use data_types::{
    internal::{
        ChannelList,
        Channel,
        ToSpans,
    },
};
mod draw;
mod events;
mod app;
mod config;

use draw::draw;
use events::*;
use app::{
    Action::*,
    App,
    Screen,
};

fn update_channel_list(result_sender: Sender<ChannelList>, url_sender: Sender<String>) {
    thread::spawn(move|| {
        let new_chan = fetch_new_videos(url_sender);
        result_sender.send(new_chan.clone()).unwrap();
        write_history(&new_chan);
    });

}

fn main() {
    let mut app = App::new_with_channel_list(read_history());

    let events = Events::new();

    let (result_sender, result_receiver) = channel();
    let (url_sender, url_receiver) = channel();

    /* update_channel_list(result_sender.clone(), url_sender.clone()); */
    // let mut update = true;
    let mut update = false;

    loop {
        let event = events.next();

        if update {
            match result_receiver.try_recv() {
                Ok(v) => {
                    app.channel_list = v;
                    update = false;
                    app.action(Update);
                },
                Err(_) => {}
            }
        }

        match event.unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') => { // ----------------- close -----------------------
                    match app.current_screen {
                        Channels => break,
                        Videos => app.action(Back),
                    }
                },
                Key::Esc | Key::Char('h') | Key::Left => { // ---------------------- back --------------
                    match app.current_screen {
                        Channels => {},
                        Videos => app.action(Back),
                    }
                }
                Key::Char('j') | Key::Down => { // ---------------------- Down ---------------------
                    app.action(Down);
                },
                Key::Char('k') | Key::Up => {
                    app.action(Up);
                },
                Key::Char('\n') | Key::Char('l') | Key::Right => {  // ----------- open ---------------
                    match app.current_screen {
                        Channels => app.action(Enter),
                        Videos => {}
                    }
                },
                Key::Char('o') => {
                    match app.current_screen {
                        Channels => app.action(Enter),
                        Videos => {
                            app.action(Open);
                            if app.config.mark_on_open {
                                app.action(Mark);
                            }
                        },
                    }
                }
                Key::Char('m') => { // ----------- mark ---------------
                    app.action(Mark);
                },
                Key::Char('M') => { // ----------- unmark -------------
                    app.action(Unmark);
                },
                Key::Char('r') => {
                    update_channel_list(result_sender.clone(), url_sender.clone());
                    app.action(Back);
                    update = true;
                }
                /* Key::Char('t') => {
                 *     app.config.show_empty_channels = !app.config.show_empty_channels;
                 *     if app.config.show_empty_channels {
                 *         app.backup_list = app.channel_list.clone();
                 *         app.channel_list = app.channel_list.clone();
                 *     } else {
                 *         app.channel_list = app.backup_list.clone();
                 *     }
                 *     app.update();
                 * } */
                _ => {}
            }
            Event::Tick => {
                app.update_line = match url_receiver.try_recv() {
                    Ok(v) => v,
                    Err(_) => String::new(),
                };
                app.action(Update);
            }

        }
    }
}
