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

static mut MSG_LINE_TIMER: u8 = 1;
static mut MSG_LINE: String = String::new();

pub fn draw(app: &mut App) {

    // -------------- Visuals/Data ---------------
    let title = app.config.app_title.clone();

    let update_line = app.update_line.clone();

    unsafe {
        MSG_LINE_TIMER -= 1;
        if MSG_LINE_TIMER == 0 {
            MSG_LINE = if let Some(rec) = app.msg_array.pop() { 
                MSG_LINE_TIMER = app.config.message_timeout;
                rec
            } else {
                MSG_LINE_TIMER = 1;
                String::from(INFO_LINE)
            };
        }
    }

    let current_selected = app.get_current_selected();

    let (show_second_block, channel_name) = match app.current_screen {
        Channels => (false, String::new()),
        Videos => {
            let right_title = app.get_channel_list().channels[current_selected].name.clone();
            (true, right_title)
        }
    };

    let mut block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let symbol = match show_second_block {
        true => "-",
        false => ">>",
    };

    let constraints = match app.current_screen {
        Channels =>  [ Constraint::Percentage(100) ].as_ref(),
        Videos => [ Constraint::Percentage(35), Constraint::Percentage(65) ].as_ref(),
    };
    // -------------------------------------------

    // all channels - left
    let mut all_chan = app.get_channel_list().clone();
    let mut chan = Vec::new();
    let chan_str: Vec<Spans> = all_chan.channels.iter_mut().map(|e| e.to_spans()).collect();
    for e in chan_str.into_iter() {
        chan.push(ListItem::new(e));
    }
    let chan_state = &mut all_chan.list_state;


    // all videos - right
    let mut all_vids = match app.get_channel_list().channels.get(current_selected) {
        Some(e) => e.clone(),
        None => Channel::new(),
    };
    let mut vid = Vec::new();
    let vid_str: Vec<Spans> = all_vids.videos.iter_mut().map(|e| e.to_spans()).collect();
    for e in vid_str.into_iter() {
        vid.push(ListItem::new(e));
    }
    let vid_state = &mut all_vids.list_state;

    // playback history - far right
    let mut playback_history = Vec::new();
    let playback_history_spans: Vec<Spans> = app.playback_history.iter_mut().rev().map(|v| v.to_spans()).collect();
    for e in playback_history_spans.into_iter() {
        playback_history.push(ListItem::new(e));
    }

    let _res = app.terminal.draw(|f| {

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
        let new_and_playback = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(main_structure[0]);

        // --------------------------
        let channel_and_video = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(constraints)
            .split(new_and_playback[0]);

        let list = List::new(chan.clone())
            .block(block.clone())
            .highlight_style(Style::default())
            .highlight_symbol(symbol);
        f.render_stateful_widget(list, channel_and_video[0], chan_state);

        if show_second_block {
            block = block.title(format!(" {} ", channel_name));

            let list = List::new(vid.clone())
                .block(block.clone())
                .highlight_style(Style::default())
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, channel_and_video[1], vid_state);
        }

        block = block.title(" Playback History ");
        let playback_history = List::new(playback_history)
            .block(block.clone())
            .highlight_style(Style::default())
            .highlight_symbol(symbol);
        f.render_widget(playback_history, new_and_playback[1]);

        let par_1 = Paragraph::new(Span::from(update_line.clone()))
            .style(Style::default())
            .alignment(Alignment::Left);
        f.render_widget(par_1, main_structure[1]);

        unsafe {
            let par_2 = Paragraph::new(Span::from(MSG_LINE.clone()))
                .style(Style::default())
                .alignment(Alignment::Left);
            f.render_widget(par_2, main_structure[2]);
        }
    });
}
