mod backend;
mod events;

use crate::backend::{core::Core, data::Data, Action::*, Error, Screen::*};
use crate::notification::notify_link;
use copypasta::{ClipboardContext, ClipboardProvider};
use events::*;
use termion::event::Key;

mod notification;

fn main() -> Result<(), Error> {
    let mut core = match Core::load() {
        Ok(core) => core,
        Err(error) => {
            return Err(error);
        }
    };

    let events = Events::new();

    let mut tick_counter = 0;

    let data = Data::init(core.status_sender.clone());

    if core.update_at_start() {
        data.update();
    }

    loop {
        let event = events.next();

        if let Ok(c) = data.try_recv() {
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
                        Videos => {
                            let _ = core.action(Leave);
                        }
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
                        Channels => {
                            let _ = core.action(Enter);
                        }
                        Videos => {
                            core.action(Open);
                        }
                    }
                    core.draw();
                }
                Key::Char('m') => {
                    core.action(Mark(true));
                    core.draw();
                }
                Key::Char('M') => {
                    core.action(Mark(false));
                    core.draw();
                }
                Key::Char('r') => {
                    /* update_channel_list(channel_update_sender.clone()); */
                    data.update();
                    core.action(Leave);
                }
                Key::Char('t') => {
                    core.set_show_empty(!core.get_show_empty());
                    core.draw();
                }
                Key::Char('c') => match core.get_current_screen() {
                    Channels => (),
                    Videos => {
                        let mut ctx = ClipboardContext::new().unwrap();
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

                if core.terminal.update_size() {
                    core.draw();
                }
            }
        }
    }

    Ok(())
}
