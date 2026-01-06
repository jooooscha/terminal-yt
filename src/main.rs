mod backend;
mod events;

use std::fs::File;
use std::sync::mpsc::channel;
use std::sync::{RwLock, Arc};

use crate::backend::{core::Core, draw::draw, data::Data, Action::*, Error, Screen::*};
use crate::notification::*;
use arboard::Clipboard;
use backend::data::downloader::Downloader;
use events::*;
use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};
use termion::event::Key;
use log::*;

mod notification;

fn main() -> Result<(), Error> {

    // init loggin
    let loggin_config = ConfigBuilder::new()
        .add_filter_ignore("reqwest".to_string())
        .set_target_level(LevelFilter::Debug)
        .build();

    WriteLogger::init(
        LevelFilter::Debug,
        loggin_config,
        File::create("debug.log").unwrap(),
    ).unwrap();

    let core = match Core::load() {
        Ok(core) => core,
        Err(error) => {
            return Err(error);
        }
    };

    let core = Arc::new(RwLock::new(core));

    let events = Events::new(); // event queue
    let mut tick_counter = 0;

    let (status_sender, status_receiver) = channel();
    let data = Data::init(status_sender.clone());

    let downloader = Downloader::new(status_sender);

    if core.read().unwrap().update_at_start() {
        data.update(&core.read().unwrap().config);
    }


    loop {
        let event = events.next();

        if let Ok(c) = data.try_recv() {
            let core_write_lock = core.try_write();
            if let Ok(mut core) = core_write_lock {
                downloader.sync_channel(c.clone());
                core.update_channel(c);
                core.save();
            }
        }

        let core_pointer = Arc::clone(&core);
        let core_write_lock = core.write();

        if let Ok(mut core) = core_write_lock {
            match event.unwrap() {
                Event::Input(input) => match input {
                    Key::Char('q') => {
                        // ----------------- close -----------------------
                        match core.get_current_screen() {
                            Channels => break,
                            Videos => {
                                core.action(Leave);
                                draw(core_pointer);
                            }
                        }
                    }
                    Key::Esc | Key::Char('h') | Key::Left => {
                        // ---------------------- back --------------
                        match core.get_current_screen() {
                            Channels => {}
                            Videos => {
                                core.action(Leave);
                            }
                        }
                        draw(core_pointer);
                    }
                    Key::Char('j') | Key::Down => {
                        // ---------------------- Down ---------------------
                        core.action(Down);
                        draw(core_pointer);
                    }
                    Key::Char('k') | Key::Up => {
                        core.action(Up);
                        draw(core_pointer);
                    }
                    Key::Char('n') => {
                        core.action(NextChannel);
                        draw(core_pointer);
                    }
                    Key::Char('p') => {
                        core.action(PrevChannel);
                        draw(core_pointer);
                    }
                    Key::Char('f') => {
                        match core.get_current_screen() {
                            Channels => {}
                            Videos => {
                                core.action(SetVideoFav);
                            }
                        }
                        draw(core_pointer);
                    }
                    Key::Char('\n') | Key::Char('l') | Key::Right | Key::Char('o') => {
                        match core.get_current_screen() {
                            Channels => {
                                core.action(Enter);
                            }
                            Videos => {
                                core.action(Open);
                            }
                        }
                        draw(core_pointer);
                    }
                    Key::Char('m') => {
                        core.action(Mark(true));
                        draw(core_pointer);
                    }
                    Key::Char('M') => {
                        core.action(Mark(false));
                        draw(core_pointer);
                    }
                    Key::Char('r') => {
                        /* update_channel_list(channel_update_sender.clone()); */
                        data.update(&core.config);
                        core.action(Leave);
                    }
                    Key::Char('t') => {
                        // core.set_show_empty(!core.get_show_empty());
                        core.toggle_filter();
                        draw(core_pointer);
                    }
                    Key::Char('c') => match core.get_current_screen() {
                        Channels => (),
                        Videos => {
                            let link = core.get_selected_video_link();
                            notify_link(&link);

                            let mut clipboard = Clipboard::new().unwrap();
                            if let Err(err) = clipboard.set_text(link) {
                                notify_error(&format!("{:?}", err));
                            }
                        }
                    },
                    _ => {}
                },
                Event::Tick => {

                    let mut changed = false;

                    if tick_counter == 0 {

                        for item in status_receiver.try_iter() {
                            changed = true;
                            core.update_status_line(item)
                        }

                        if changed {
                            tick_counter = 4;
                        }

                    } else {
                        tick_counter -= 1
                    }

                    if core.terminal.update_size() || changed {
                        draw(core_pointer);
                    }
                }
            }
        }
    }

    Ok(())
}
