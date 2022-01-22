use crate::backend::{
    core::{Core, StatusUpdate},
    data::{channel::Channel, channel_list::ChannelList},
    io::history::History,
    Backend, Screen,
    Screen::*,
    Terminal,
};
use std::thread;
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

pub struct AppState {
    channel: Option<Channel>,
    channel_list: ChannelList,
    history: History,
    screen: Screen,
    status: Vec<StatusUpdate>,
    terminal: Terminal,
}

impl From<&Core> for AppState {
    fn from(core: &Core) -> Self {
        let channel = core.get_selected_channel().cloned();
        let channel_list = core.get_filtered_channel_list().clone();
        let history = core.playback_history.clone();
        let screen = core.current_screen.clone();
        let status = core.status.clone();
        let terminal = core.terminal.clone();

        AppState {
            channel,
            channel_list,
            history,
            screen,
            status,
            terminal,
        }
    }
}

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
    main: Vec<Rect>,
    content: Vec<Rect>,
    other: Vec<Rect>,
}

impl AppLayout {
    fn load(f: &mut Frame<'_, Backend>, screen: &Screen) -> Self {
        let video_size = match screen {
            Channels => 0,
            Videos => 75,
        };

        let main_split = vec![Percentage(80), Percentage(19), Percentage(1)];
        let content_split = vec![Percentage(100 - video_size), Percentage(video_size)];
        let other_split = vec![Percentage(50), Percentage(50)];

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

        let other = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(other_split)
            .split(main[1]);

        Self {
            main,
            content,
            other,
        }
    }

    fn status(&self) -> Rect {
        self.other[1]
    }

    fn info(&self) -> Rect {
        self.main[2]
    }

    fn history(&self) -> Rect {
        self.other[0]
    }

    fn channels(&self) -> Rect {
        self.content[0]
    }

    fn videos(&self) -> Rect {
        self.content[1]
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn draw(app: AppState) {
    thread::spawn(move || {
        // all channels - left view
        let channels = app.channel_list.clone();

        // all videos - right view
        /* let current_channel = app.current_selected.clone(); */

        let channel_symbol = match app.screen {
            Channels => ">> ",
            Videos => "-",
        };
        let chan_widget = Widget::builder()
            .with_title("Test name")
            .with_symbol(channel_symbol)
            .with_list(channels.get_spans_list());

        let _ = app.terminal.term.clone().lock().unwrap().draw(|f| {
            let layout = AppLayout::load(f, &app.screen);

            f.render_stateful_widget(
                chan_widget.render(),
                layout.channels(),
                &mut channels.state(),
            );

            if let Some(channel) = app.channel {
                let video_widget = Widget::builder()
                    .with_title(&format!(" {} ", channel.name()))
                    .with_symbol(">> ")
                    .with_list(channel.get_spans_list());

                f.render_stateful_widget(
                    video_widget.render(),
                    layout.videos(),
                    &mut channel.state(),
                );
            }

            let mut list = Vec::new();
            for line in app.status {
                list.push(line.to_list_item());
            }
            let widget_height = layout.status().height as usize - 2; // minus two for the borders
            list.reverse();
            list.truncate(widget_height);
            list.reverse();
            let status = Widget::builder()
                .with_title("Download Status")
                .with_list(list);

            f.render_widget(status.render(), layout.status());

            let history_widget = Widget::builder()
                .with_title(" Playback History ")
                .with_list(app.history.to_list_items());

            f.render_widget(history_widget.render(), layout.history());

            //////////////////////////////

            // let par_1 = Paragraph::new(Span::from(app.status.clone()))
            //     .style(Style::default())
            //     .alignment(Alignment::Left);
            // f.render_widget(par_1, layout.status());

            let par_2 = Paragraph::new(Span::from(INFO_LINE))
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(par_2, layout.info());
        });
    });
}
