use std::io::Write;
use tui::{
    widgets::{
        Paragraph,
        BorderType,
    },
    style::Style,
    text::{
        Spans,
        Span,
    },
    layout::{
        Alignment,
        Layout,
        Constraint,
        Direction
    },
};

use crate::*;
use fetch_data::structs::ToString;

const INFO_LINE: &str = "q close, o open video/select, Enter select, m mark, M unmark, j down, k up, R redraw screen (buggy)";

pub fn draw<W: Write>(app: &mut App<W>) {

    let mut all_chan = app.all_channels.clone();
    let mut chan = Vec::new();
    let chan_str: Vec<Spans> = all_chan.channels.iter_mut().map(|e| e.to_string()).collect();
    for e in chan_str.into_iter() {
        chan.push(ListItem::new(e));
    }
    let chan_state = &mut all_chan.list_state;

    let i = app.current_selected;

    let mut all_vids = app.all_channels.channels[i].clone();
    let mut vid = Vec::new();
    let vid_str: Vec<Spans> = all_vids.videos.iter_mut().map(|e| e.to_string()).collect();
    for e in vid_str.into_iter() {
        vid.push(ListItem::new(e));
    }
    let vid_state = &mut all_vids.list_state;

    let constraints = match app.current_screen {
        Channels =>  [ Constraint::Percentage(100) ].as_ref(),
        Videos => [ Constraint::Percentage(35), Constraint::Percentage(65) ].as_ref(),
    };

    let (show_second_block, channel_name) = match app.current_screen {
        Channels => (false, String::new()),
        Videos => {
            let right_title = app.all_channels.channels[i].name.clone();
            (true, right_title)
        }
    };

    let title = String::from("TYT");

    let _res = app.terminal.draw(|f| {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Percentage(99),
                Constraint::Percentage(1),
            ])
            .split(f.size());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(constraints)
            .split(vert[0]);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let list = List::new(chan.clone())
            .block(block)
            .highlight_style(Style::default())
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[0], chan_state);

        if show_second_block {
            let block = Block::default()
                .title(channel_name)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            let list = List::new(vid.clone())
                .block(block)
                .highlight_style(Style::default())
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[1], vid_state);
        }

        let paragraph = Paragraph::new(Span::from(INFO_LINE))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(paragraph, vert[1]);

    });
}
