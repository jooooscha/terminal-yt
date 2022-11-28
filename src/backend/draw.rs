use crate::backend::{
    core::Core,
    data::{channel::Channel, channel_list::ChannelList},
    io::{history::History, config::Config},
    layout::AppLayout,
    Screen,
    Screen::*,
    Terminal,
};
use std::thread;
use tui::widgets::ListItem;
use tui::{
    layout::Alignment,
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, List, Paragraph},
};

const INFO_LINE: &str =
    "q close; o open video/select; Enter/l select; Esc/h go back; m mark; M unmark";

pub struct AppState {
    channel: Option<Channel>,
    channel_list: ChannelList,
    history: History,
    screen: Screen,
    terminal: Terminal,
    config: Config,
}

impl From<&Core> for AppState {
    fn from(core: &Core) -> Self {
        let channel = core.get_selected_channel().cloned();
        let channel_list = core.channel_list().clone();
        let history = core.history.clone();
        let screen = core.current_screen.clone();
        let terminal = core.terminal.clone();
        let config = core.config.clone();

        AppState {
            channel,
            channel_list,
            history,
            screen,
            terminal,
            config,
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


#[allow(clippy::unnecessary_unwrap)]
pub fn draw(app: AppState) {
    thread::spawn(move || {
        let channels = app.channel_list.clone();

        let channel_symbol = match app.screen {
            Channels => ">> ",
            Videos => "-",
        };
        let chan_widget = Widget::builder()
            .with_title(&format!(" {} ", app.config.app_title))
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

            let history_widget = Widget::builder()
                .with_title(" Playback History ")
                .with_list(app.history.to_list_items());

            f.render_widget(history_widget.render(), layout.history());

            // let stats_widget = Widget::builder()
            //     .with_title(" Stats ")
            //     .with_list(app.history.to_list_items());

            // f.render_widget(history_widget.render(), layout.history());


            //////////////////////////////


            let stats = app.history.stats();
            let videos = match stats.today() {
                Some(stats) => stats.watched,
                None => 0
            };
            let info = Paragraph::new(Span::from(format!("{} - Videos Today: {} - Starts: {}", INFO_LINE, videos, stats.starts)))
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(info, layout.info());
        });
    });
}
