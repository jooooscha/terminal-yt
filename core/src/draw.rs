use crate::{
    config::Config,
    core::Core,
    data_types::{channel::channel::Channel, channel_list::ChannelList},
    history::MinimalVideo,
    Screen::*,
    ToTuiListItem,
};
use std::sync::{Arc, Mutex};
use std::thread;
use tui::{backend::TermionBackend, widgets::ListItem, Terminal};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, List, Paragraph},
};

const INFO_LINE: &str =
    "q close; o open video/select; Enter/l select; Esc/h go back; m mark; M unmark";

pub struct View {
    #[cfg(not(test))]
    terminal: Arc<
        Mutex<
            Terminal<
                TermionBackend<
                    termion::screen::AlternateScreen<
                        termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
                    >,
                >,
            >,
        >,
    >,
    #[cfg(test)]
    terminal: Arc<
        Mutex<
            Terminal<
                TermionBackend<
                    termion::screen::AlternateScreen<
                        termion::input::MouseTerminal<std::io::Stdout>,
                    >,
                >,
            >,
        >,
    >,
    config: Config,
    update_line: String,
    show_videos: bool,
    channel_list: ChannelList,
    current_selected: Channel,
    selected_channel_name: String,
    playback_history: Vec<MinimalVideo>,
}

impl From<&Core> for View {
    fn from(core: &Core) -> Self {
        let terminal = core.terminal.clone();
        let config = core.config.clone();
        let update_line = core.update_line.clone();
        let show_channel_block = core.current_screen == Videos;
        let channel_list = core.get_filtered_channel_list().clone();
        let current_selected = core.get_selected_channel().clone();
        let selected_channel_name = current_selected.name().clone();
        let playback_history = core.playback_history.clone();

        View {
            terminal,
            config,
            update_line,
            show_videos: show_channel_block,
            channel_list,
            current_selected,
            selected_channel_name,
            playback_history,
        }
    }
}

pub fn draw(app: View) {
    thread::spawn(move || {
        let mut block = Block::default()
            .title(app.config.app_title.clone())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let symbol = match app.show_videos {
            true => "-",
            false => ">>",
        };

        let constraints = if app.show_videos {
            [Constraint::Percentage(35), Constraint::Percentage(65)].as_ref()
        } else {
            [Constraint::Percentage(100)].as_ref()
        };

        // -------------------------------------------

        // all channels - left view
        let channels = app.channel_list.clone();

        let chan: Vec<ListItem> = channels.get_spans_list();

        // all videos - right view
        let current_channel = app.current_selected.clone();

        let mut vid_state = current_channel.state();

        let vid = current_channel.get_spans_list();

        // playback history - bottom view
        let playback_history: Vec<ListItem> = app
            .playback_history
            .iter()
            .map(|v| v.to_list_item())
            .rev()
            .collect();

        let _res = app.terminal.clone().lock().unwrap().draw(|f| {
            // --------------------------
            let main_structure = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Percentage(97),
                    Constraint::Percentage(2),
                    Constraint::Percentage(1),
                ])
                .split(f.size());

            // --------------------------
            // LAYOUT (Blocks)
            // --------------------------

            // main two parts split
            let channel_list_and_history = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(main_structure[0]);

            // split of channels and their videos
            let channel_and_video = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(constraints)
                .split(channel_list_and_history[0]);

            // --------------------------

            let list = List::new(chan)
                .block(block.clone())
                .highlight_style(Style::default())
                .highlight_symbol(symbol);
            f.render_stateful_widget(list, channel_and_video[0], &mut channels.state());

            if app.show_videos {
                block = block.title(format!(" {} ", app.selected_channel_name));

                let list = List::new(vid.clone())
                    .block(block.clone())
                    .highlight_style(Style::default())
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, channel_and_video[1], &mut vid_state);
            }

            block = block.title(" Playback History ");
            let playback_history = List::new(playback_history)
                .block(block.clone())
                .highlight_style(Style::default())
                .highlight_symbol(symbol);
            f.render_widget(playback_history, channel_list_and_history[1]);

            let par_1 = Paragraph::new(Span::from(app.update_line.clone()))
                .style(Style::default())
                .alignment(Alignment::Left);
            f.render_widget(par_1, main_structure[1]);

            let par_2 = Paragraph::new(Span::from(INFO_LINE.clone()))
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(par_2, main_structure[2]);
        });
    });
}
