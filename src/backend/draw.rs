use crate::backend::{
    io::config::Config,
    core::Core,
    data::{
        channel::Channel,
        channel_list::ChannelList,
    },
    io::history::History,
    Screen,
    Screen::*,
    Terminal,
    Backend,
};
use std::thread;
use tui::widgets::ListItem;
use tui::{
    Frame,
    layout::{Alignment, Constraint::*, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, List, Paragraph},
};

const INFO_LINE: &str =
    "q close; o open video/select; Enter/l select; Esc/h go back; m mark; M unmark";

pub struct AppDraw {
    terminal: Terminal,
    config: Config,
    update_line: String,
    screen: Screen,
    channel_list: ChannelList,
    current_selected: Option<Channel>,
    /* selected_channel_name: String, */
    playback_history: History,
}

impl From<&Core> for AppDraw {
    fn from(core: &Core) -> Self {
        let terminal = core.terminal.clone();
        let config = core.config.clone();
        let update_line = core.update_line.clone();
        let screen = core.current_screen.clone();
        let channel_list = core.get_filtered_channel_list().clone();
        let current_selected = core.get_selected_channel().cloned();
        let playback_history = core.playback_history.clone();

        AppDraw {
            terminal,
            config,
            update_line,
            screen,
            channel_list,
            current_selected,
            playback_history,
        }
    }
}

struct Widget<'a> {
    title: String,
    list: Vec<ListItem<'a>>,
}

impl<'a> Widget<'a> {
    fn render(self) -> List<'a> {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        List::new(self.list)
            .block(block)
            .highlight_style(Style::default())
            .highlight_symbol(">>")
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
            Channels => {
                0
            },
            Videos => {
                75 
            },
        };

        let main_split = vec![ Percentage(80), Percentage(17), Percentage(2), Percentage(1) ];
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
        self.main[2]
    }

    fn info(&self) -> Rect {
        self.main[3]
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
pub fn draw(app: AppDraw) {
    thread::spawn(move || {
        let mut block = Block::default()
            .title(app.config.app_title.clone())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let symbol = match app.screen == Videos {
            true => "-",
            false => ">>",
        };

        // all channels - left view
        let channels = app.channel_list.clone();

        let chan: Vec<ListItem> = channels.get_spans_list();

        // all videos - right view
        let current_channel = app.current_selected.clone();

        // playback history - bottom view
        let playback_history = app.playback_history.to_list_items();

        let chan_part = Widget {
            title: "test name".to_string(),
            list: channels.get_spans_list(),
        };

        let _ = app.terminal.term.clone().lock().unwrap().draw(|f| {

            let layout = AppLayout::load(f, &app.screen);

/*             let list = List::new(chan)
 *                 .block(block.clone())
 *                 .highlight_style(Style::default())
 *                 .highlight_symbol(symbol);
 *
 *             f.render_stateful_widget(list, layout.channels(), &mut channels.state()); */

            f.render_stateful_widget(chan_part.render(), layout.channels(), &mut channels.state());

            if current_channel.is_some() {
                let channel = current_channel.unwrap();
                let name = channel.name();
                let state = &mut channel.state();
                let videos = channel.get_spans_list();

                block = block.title(format!(" {} ", name));

                let list = List::new(videos)
                    .block(block.clone())
                    .highlight_style(Style::default())
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, layout.videos(), state);
            }

            block = block.title(" Playback History ");
            let playback_history = List::new(playback_history)
                .block(block.clone())
                .highlight_style(Style::default())
                .highlight_symbol(symbol);
            f.render_widget(playback_history, layout.history());

            let par_1 = Paragraph::new(Span::from(app.update_line.clone()))
                .style(Style::default())
                .alignment(Alignment::Left);
            f.render_widget(par_1, layout.status());

            let par_2 = Paragraph::new(Span::from(INFO_LINE))
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(par_2, layout.info());
        });
    });
}
