mod events;

use clipboard::{ClipboardContext, ClipboardProvider};
/* use data::{fetch_data::fetch_new_videos, history::read_history}; */
use app::{
    fetch_data::fetch_new_videos,
    Action::*,
    App,
    Screen::*,
    data_types::internal::{
        Channel,
        Filter,
    },
};
/* use data::internal::{Channel, ChannelList, Filter}; */
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use termion::event::Key;
/* use app::Screen::*;
 * use app::{Action::*, App, Screen}; */
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

    let mut app = App::new_from_history();

    let events = Events::new();

    let (channel_update_sender, channel_update_receiver) = channel();

    if app.config.update_at_start {
        update_channel_list(app.status_sender.clone(), channel_update_sender.clone());
    }

    loop {
        let event = events.next();

        if let Ok(c) = channel_update_receiver.try_recv() {
            app.update_channel(c);
            app.save();

            app.draw();
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
                        app.status_sender.clone(),
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
                app.update_status_line();

                app.draw();
            }
        }
    }
}
