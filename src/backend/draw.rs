use crate::backend::{
    core::Core,
    Screen,
    Screen::*,
};
use std::{thread, rc::Rc, sync::{RwLock, Arc}};
use tui::widgets::ListItem;
use tui::{
    layout::{Alignment, Constraint::*, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, List, Paragraph},
    Frame,
};

const INFO_LINE: &str =
    "q close; o open video/select; Enter/l select; Esc/h go back; m mark; M unmark";

#[derive(Default)]
struct Widget<'a> {
    title: String,
    symbol: &'a str,
    list: Vec<ListItem<'a>>,
}

impl<'a> Widget<'a> {
    fn builder() -> Self {
        Self::default()
    }

    fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    fn with_list(mut self, list: Vec<ListItem<'a>>) -> Self {
        self.list = list;
        self
    }

    fn with_symbol(mut self, symbol: &'a str) -> Self {
        self.symbol = symbol;
        self
    }

    fn render(self) -> List<'a> {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        List::new(self.list)
            .block(block)
            .highlight_style(Style::default())
            .highlight_symbol(self.symbol)
    }
}

pub struct AppLayout {
    main: Rc<[Rect]>,
    content: Rc<[Rect]>,
}

impl AppLayout {
    fn load(f: &mut Frame<'_>, screen: &Screen) -> Self {
        let video_size = match screen {
            Channels => 0,
            Videos => 75,
        };

        let main_split = vec![Percentage(80), Percentage(19), Percentage(1)];
        let content_split = vec![Percentage(100 - video_size), Percentage(video_size)];

        let main = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(main_split)
            .split(f.size());

        let content = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(content_split)
            .split(main[0]);

        Self {
            main,
            content,
        }
    }

    fn info(&self) -> Rect {
        self.main[2]
    }

    fn history(&self) -> Rect {
        self.main[1]
    }

    fn channels(&self) -> Rect {
        self.content[0]
    }

    fn videos(&self) -> Rect {
        self.content[1]
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn draw(core: Arc<RwLock<Core>>) {
    thread::spawn(move || {
        let (
            channels,
            current_screen,
            app_title,
            terminal,
            history,
        ) = {
            let core_read_lock = core.read().unwrap();
            (
                core_read_lock.channel_list().clone(),
                core_read_lock.current_screen.clone(),
                core_read_lock.config.app_title.clone(),
                core_read_lock.terminal.term.clone(),
                core_read_lock.playback_history.clone(),
            )
        };

        let channel_symbol = match current_screen {
            Channels => ">> ",
            Videos => "-",
        };
        let chan_widget = Widget::builder()
            .with_title(&format!(" {} ", app_title))
            .with_symbol(channel_symbol)
            .with_list(channels.get_spans_list());

        let _ = terminal.lock().unwrap().draw(|f| {
            let layout = AppLayout::load(f, &current_screen);

            if let Ok(mut core_write_lock) = core.try_write() {

                f.render_stateful_widget(
                    chan_widget.render(),
                    layout.channels(),
                    core_write_lock.channel_list_mut().state_mut(),
                );

                if let Some(channel) = core_write_lock.get_selected_channel_mut() {
                    let c = channel.clone();
                    let video_widget = Widget::builder()
                        .with_title(&format!(" {} ", channel.name()))
                        .with_symbol(">> ")
                        .with_list(c.get_spans_list());

                    f.render_stateful_widget(
                        video_widget.render(),
                        layout.videos(),
                        channel.state_mut(),
                    );
                }
            }

            let history_widget = Widget::builder()
                .with_title(" Playback History ")
                .with_list(history.to_list_items());

            f.render_widget(history_widget.render(), layout.history());

            //////////////////////////////

            let info = Paragraph::new(Span::from(INFO_LINE))
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(info, layout.info());
        });
    });
}
