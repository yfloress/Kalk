// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

//! Fuzzy search uses the existing popup, input and selectable list widgets.

use super::{keyhints::render_hint, surfaces::popup_block, theme::theme};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};

pub(super) fn draw_search_popup(frame: &mut Frame, app: &App) {
    let Some(search) = &app.search else {
        return;
    };
    let m = app.messages();
    let t = theme();
    let area = frame
        .area()
        .centered(Constraint::Percentage(85), Constraint::Percentage(75));
    frame.render_widget(Clear, area);
    let block = popup_block(format!(" {} ({}) ", m.search_title, search.entries.len()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);
    let query = rows[0];
    let width = query.width.saturating_sub(2).max(1);
    let scroll = search.input.visual_scroll(width as usize);
    frame.render_widget(
        Paragraph::new(search.input.value())
            .scroll((0, scroll.min(u16::MAX as usize) as u16))
            .style(Style::default().fg(t.text_primary))
            .block(
                Block::bordered()
                    .title(m.search_query)
                    .border_type(t.border_type)
                    .border_style(Style::default().fg(t.input_active)),
            ),
        query,
    );
    if query.height >= 3 && query.width > 2 {
        let x = search
            .input
            .visual_cursor()
            .saturating_sub(scroll)
            .min(width as usize - 1) as u16;
        frame.set_cursor_position((query.x + x + 1, query.y + 1));
    }
    if search.entries.is_empty() {
        frame.render_widget(
            Paragraph::new(m.search_empty).style(Style::default().fg(t.text_secondary)),
            rows[1],
        );
    } else {
        let list = List::new(
            search
                .entries
                .iter()
                .map(|entry| ListItem::new(Line::from(entry.label.as_str()))),
        )
        .style(Style::default().fg(t.text_primary))
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        );
        let mut state = ListState::default().with_selected(Some(search.selected));
        frame.render_stateful_widget(list, rows[1], &mut state);
    }
    render_hint(
        frame,
        &[
            ("↑↓", m.help_move_updown),
            ("Enter", m.confirm),
            ("Esc", m.cancel),
        ],
        rows[2],
    );
}
