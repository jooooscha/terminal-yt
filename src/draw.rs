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

use crate::{
    *,
    app::{
        App,
    },
};

const INFO_LINE: &str = "q close; o open video/select; Enter/l select; Esc/h go back; m mark; M unmark";

pub fn draw(app: &mut App) {
    let mut all_chan = app.get_channel_list().clone();
    let mut chan = Vec::new();
    let chan_str: Vec<Spans> = all_chan.channels.iter_mut().map(|e| e.to_spans()).collect();
    for e in chan_str.into_iter() {
        chan.push(ListItem::new(e));
    }
    let chan_state = &mut all_chan.list_state;

    let i = app.get_current_selected();

    let mut all_vids = match app.get_channel_list().channels.get(i) {
        Some(e) => e.clone(),
        None => Channel::new(),
    };
    let mut vid = Vec::new();
    let vid_str: Vec<Spans> = all_vids.videos.iter_mut().map(|e| e.to_spans()).collect();
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
            let right_title = app.get_channel_list().channels[i].name.clone();
            (true, right_title)
        }
    };

    let title = app.config.app_title.clone();

    let update_line = app.update_line.clone();

    let _res = app.terminal.draw(|f| {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Percentage(97),
                Constraint::Percentage(2),
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

        let symbol = match show_second_block {
            true => "-",
            false => ">>",
        };

        let list = List::new(chan.clone())
            .block(block)
            .highlight_style(Style::default())
            .highlight_symbol(symbol);
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

        let par_1 = Paragraph::new(Span::from(update_line.clone()))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(par_1, vert[1]);

        let par_2 = Paragraph::new(Span::from(INFO_LINE))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(par_2, vert[2]);
    });
}
