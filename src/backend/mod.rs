pub(crate) mod core;
pub(crate) mod data;

mod draw;
pub(super) mod io;

use serde::{Deserialize, Serialize};
use tui::widgets::ListItem;

use std::{
    io::{stdin, stdout, Stdout},
    sync::{Arc, Mutex},
};
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, layout::Rect, Terminal as TuiTerminal};

pub trait ToTuiListItem {
    fn to_list_item(&self) -> ListItem;
}

#[cfg(not(test))]
type TermScreen = MouseTerminal<RawTerminal<Stdout>>;
#[cfg(test)]
type TermScreen = MouseTerminal<Stdout>;

type Backend = TermionBackend<AlternateScreen<TermScreen>>;

type Term = Arc<Mutex<TuiTerminal<Backend>>>;

#[derive(Clone)]
pub(crate) struct Terminal {
    term: Term,
    last_size: Rect,
}

impl Default for Terminal {
    fn default() -> Self {
        #[cfg(not(test))]
        let stdout = stdout().into_raw_mode().unwrap();
        #[cfg(test)]
        let stdout = stdout();
        let mouse_terminal = MouseTerminal::from(stdout);
        /* let screen = mouse_terminal; */
        let screen = AlternateScreen::from(mouse_terminal);
        let _stdin = stdin();
        let backend = TermionBackend::new(screen);
        let terminal = TuiTerminal::new(backend).unwrap();
        let size = terminal.size().unwrap();
        let term = Arc::new(Mutex::new(terminal));

        Terminal {
            term,
            last_size: size,
        }
    }
}

impl Terminal {
    pub(crate) fn update_size(&mut self) -> bool {
        let changed = self.current_size() != self.last_size;
        self.last_size = self.current_size();
        changed
    }

    fn current_size(&self) -> Rect {
        self.term.clone().lock().unwrap().size().unwrap()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Filter {
    NoFilter,
    OnlyNew,
}

#[derive(PartialEq)]
pub enum Action {
    Mark(bool),
    Up,
    Down,
    Enter,
    Leave,
    NextChannel,
    PrevChannel,
    Open,
    SetVideoFav,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Screen {
    Channels,
    Videos,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortingMethod {
    Date,
    Text,
    UnseenDate,
    UnseenText,
}

impl Default for SortingMethod {
    fn default() -> Self {
        Self::UnseenDate
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    ParseConfig(serde_yaml::Error),
    ParseDB(serde_json::Error),
    ParseSubscription(serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
