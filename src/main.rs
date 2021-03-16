mod events;

use clipboard::{ClipboardContext, ClipboardProvider};
use core::{
    data_types::channel::Channel, fetch_data::fetch_new_videos, Action::*, core::Core, Filter, Screen::*,
};
use events::*;
use notification::notify::notify_user;
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use termion::event::Key;

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
    let mut core = Core::new_from_history();

    let events = Events::new();

    let mut tick_counter = 0;
    let mut size = core.terminal.clone().lock().unwrap().size().unwrap();

    let (channel_update_sender, channel_update_receiver) = channel();

    if core.config.update_at_start {
        update_channel_list(core.status_sender.clone(), channel_update_sender.clone());
    }

    loop {
        let event = events.next();

        if let Ok(c) = channel_update_receiver.try_recv() {
            core.update_channel(c);
            core.save();
        }

        match event.unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    // ----------------- close -----------------------
                    match core.current_screen {
                        Channels => break,
                        Videos => {
                            core.action(Leave);
                            core.draw();
                        },
                    }
                }
                Key::Esc | Key::Char('h') | Key::Left => {
                    // ---------------------- back --------------
                    match core.current_screen {
                        Channels => {}
                        Videos => core.action(Leave),
                    }
                    core.draw();
                }
                Key::Char('j') | Key::Down => {
                    // ---------------------- Down ---------------------
                    core.action(Down);
                    core.draw();
                }
                Key::Char('k') | Key::Up => {
                    core.action(Up);
                    core.draw();
                }
                Key::Char('n') => {
                    core.action(NextChannel);
                    core.draw();
                }
                Key::Char('p') => {
                    core.action(PrevChannel);
                    core.draw();
                }
                Key::Char('\n') | Key::Char('l') | Key::Right | Key::Char('o') => {
                    match core.current_screen {
                        Channels => core.action(Enter),
                        Videos => {
                            core.action(Open);
                            if core.config.mark_on_open {
                                core.action(Mark);
                            }
                        }
                    }
                    core.draw();
                }
                Key::Char('m') => {
                    // ----------- mark ---------------
                    core.action(Mark);
                    core.draw();
                }
                Key::Char('M') => {
                    // ----------- unmark -------------
                    core.action(Unmark);
                    core.draw();
                }
                Key::Char('r') => {
                    update_channel_list(core.status_sender.clone(), channel_update_sender.clone());
                    core.action(Leave);
                }
                Key::Char('t') => {
                    core.config.show_empty_channels = !core.config.show_empty_channels;
                    let new_filter = match core.current_filter {
                        Filter::NoFilter => Filter::OnlyNew,
                        Filter::OnlyNew => Filter::NoFilter,
                    };
                    core.set_filter(new_filter);
                    core.draw();
                }
                Key::Char('c') => match core.current_screen {
                    Channels => (),
                    Videos => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let link = core.get_selected_video_link();
                        notify_user(&link);
                        ctx.set_contents(link).unwrap();
                    }
                },
                _ => {}
            },
            Event::Tick => {
                if tick_counter == 2 {
                    let actually_updated = core.update_status_line();
                    if actually_updated {
                        core.draw();
                    }
                    tick_counter = 0;
                } else {
                    tick_counter += 1;
                }

                if core.terminal.clone().lock().unwrap().size().unwrap() != size.clone() {
                    core.draw();
                    size = core.terminal.clone().lock().unwrap().size().unwrap();
                }
            }
        }
    }
}
