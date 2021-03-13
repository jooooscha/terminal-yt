use clipboard::{ClipboardContext, ClipboardProvider};
use data::{fetch_data::fetch_new_videos, history::read_history, url_file::*};
use data_types::internal::{Channel, ChannelList, Filter, ToSpans};
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use termion::event::Key;
use tui::widgets::{Block, Borders, List, ListItem};
use Screen::*;
mod app;
mod draw;
mod events;

use app::{Action::*, App, Screen};
use draw::draw;
use events::*;
use notification::notify::notify_user;

fn update_channel_list(
    status_update_sender: Sender<String>,
    channel_update_sender: Sender<Channel>,
) {
    thread::spawn(move || {
        fetch_new_videos(status_update_sender, channel_update_sender);
    });
}

fn main() {
    let result = std::panic::catch_unwind(|| {
        run();
    });

    if let Err(error_text) = result {
        panic!(error_text);
    }
}

fn run() {
    let mut history = match read_history() {
        Some(h) => h,
        None => ChannelList::new(),
    };

    let url_file_content = read_urls_file();

    history.channels = history
        .channels
        .into_iter()
        .filter(|channel| {
            url_file_content
                .channels
                .iter()
                .any(|url_channel| url_channel.id() == channel.id)
                || url_file_content
                    .channels
                    .iter()
                    .any(|url_channel| url_channel.id() == channel.id)
        })
        .collect();

    let mut app = App::new_from_channel_list(history);

    let events = Events::new();
    let tick_counter_limit = 10;
    let mut tick_counter = 0;

    let (status_update_sender, status_update_reveiver) = channel();
    let (channel_update_sender, channel_update_receiver) = channel();

    if app.config.update_at_start {
        update_channel_list(status_update_sender.clone(), channel_update_sender.clone());
    }

    loop {
        let event = events.next();

        for c in channel_update_receiver.try_iter() {
            app.update_channel_list(c);
            app.save();

            app.action(Update);
        }

        match event.unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    // ----------------- close -----------------------
                    match app.current_screen {
                        Channels => break,
                        Videos => app.action(Leave),
                    }
                }
                Key::Esc | Key::Char('h') | Key::Left => {
                    // ---------------------- back --------------
                    match app.current_screen {
                        Channels => {}
                        Videos => app.action(Leave),
                    }
                }
                Key::Char('j') | Key::Down => {
                    // ---------------------- Down ---------------------
                    app.action(Down);
                }
                Key::Char('k') | Key::Up => {
                    app.action(Up);
                }
                Key::Char('n') => {
                    app.action(NextChannel);
                }
                Key::Char('p') => {
                    app.action(PrevChannel);
                }
                Key::Char('\n') | Key::Char('l') | Key::Right | Key::Char('o') => {
                    match app.current_screen {
                        Channels => app.action(Enter),
                        Videos => {
                            app.action(Open);
                            if app.config.mark_on_open {
                                app.action(Mark);
                            }
                        }
                    }
                }
                Key::Char('m') => {
                    // ----------- mark ---------------
                    app.action(Mark);
                }
                Key::Char('M') => {
                    // ----------- unmark -------------
                    app.action(Unmark);
                }
                Key::Char('r') => {
                    update_channel_list(
                        status_update_sender.clone(),
                        channel_update_sender.clone(),
                    );
                    app.action(Leave);
                }
                Key::Char('t') => {
                    app.config.show_empty_channels = !app.config.show_empty_channels;
                    let new_filter = match app.current_filter {
                        Filter::NoFilter => Filter::OnlyNew,
                        Filter::OnlyNew => Filter::NoFilter,
                    };
                    app.set_filter(new_filter);
                }
                Key::Char('c') => match app.current_screen {
                    Channels => (),
                    Videos => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let link = app.get_selected_video_link();
                        notify_user(&link);
                        ctx.set_contents(link).unwrap();
                    }
                },
                _ => {}
            },
            Event::Tick => {
                tick_counter += 1;
                for v in status_update_reveiver.try_iter() {
                    app.update_line = v;
                    app.action(Update);
                }
                if tick_counter == tick_counter_limit {
                    tick_counter = 0;
                    app.update_line = String::new();
                }
                app.action(Update);
            }
        }
        app.update();
    }
}
