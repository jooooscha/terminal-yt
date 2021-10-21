mod events;

use clipboard::{ClipboardContext, ClipboardProvider};
use core::{
    core::Core, data_types::channel::channel::Channel, fetch_data::fetch_new_videos, Action::*,
    Screen::*,
};
use events::*;
use notification::notify::{notify_open, notify_link};
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use termion::event::Key;

fn update_channel_list(
    channel_update_sender: Sender<Channel>,
) {
    thread::spawn(move || {
        fetch_new_videos(channel_update_sender);
    });
}

fn main() {
    let mut core = Core::new_with_history();

    let events = Events::new();

    let mut tick_counter = 0;
    let mut size = core.terminal.clone().lock().unwrap().size().unwrap();

    let (channel_update_sender, channel_update_receiver) = channel();

    if core.update_at_start() {
        update_channel_list(channel_update_sender.clone());
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
                    match core.get_current_screen() {
                        Channels => break,
                        Videos => {
                            core.action(Leave);
                            core.draw();
                        }
                    }
                }
                Key::Esc | Key::Char('h') | Key::Left => {
                    // ---------------------- back --------------
                    match core.get_current_screen() {
                        Channels => {}
                        Videos => { let _ = core.action(Leave); }
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
                Key::Char('f') => {
                    match core.get_current_screen() {
                        Channels => {}
                        Videos => {
                            core.action(SetVideoFav);
                        }
                    }
                    core.draw();
                }
                Key::Char('\n') | Key::Char('l') | Key::Right | Key::Char('o') => {
                    match core.get_current_screen() {
                        Channels => { let _ = core.action(Enter); }
                        Videos => {
                            let video_details = core.action(Open);
                            match video_details {
                                Some(vd) => { let _ = notify_open(&vd); }
                                None => ()
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
                    update_channel_list(channel_update_sender.clone());
                    core.action(Leave);
                }
                Key::Char('t') => {
                    core.set_show_empty(!core.get_show_empty());
                    core.draw();
                }
                Key::Char('c') => match core.get_current_screen() {
                    Channels => (),
                    Videos => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let link = core.get_selected_video_link();
                        notify_link(&link);
                        ctx.set_contents(link).unwrap();
                    }
                },
                _ => {}
            },
            Event::Tick => {
                if tick_counter == 0 {
                    let actually_updated = core.update_status_line();
                    if actually_updated {
                        core.draw();
                        tick_counter = 4;
                    }
                } else {
                    tick_counter -= 1
                }

                if core.terminal.clone().lock().unwrap().size().unwrap() != size.clone() {
                    core.draw();
                    size = core.terminal.clone().lock().unwrap().size().unwrap();
                }
            }
        }
    }
}

