use std::io::Write;
use tui::{
    widgets::{
        ListState,
        BorderType,
    },
    style::Style,
    text::Spans,
};

use crate::*;
use crate::fetch_data::ToString;


pub fn draw<E: ToString, W: Write>(mut elements: Vec<E>, app: &mut App<W>, state: &mut ListState) {

    let strinified: Vec<Spans> = elements.iter_mut().map(|e| e.to_string()).collect();

    let mut items = Vec::new();
    for e in strinified.into_iter() {
        items.push(
            ListItem::new(e)
        );
    }

    let title = app.current_title.clone();
    let _res = app.terminal.draw(|f| {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        /* f.render_widget(block, f.size()); */
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default())
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, f.size(), state);
    });
}
