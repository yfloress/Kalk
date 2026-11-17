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

//! Settings rendered with Ratatui's selectable Table and shared popup surface.

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use super::icons::icons;
use super::keyhints::render_hint;
use super::surfaces::popup_block;
use super::theme::theme;

pub fn draw_settings_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = frame
        .area()
        .centered(Constraint::Length(62), Constraint::Length(16));
    frame.render_widget(Clear, area);
    let block = popup_block(format!(" {}{} ", ic.settings, m.settings));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Max(3),
        Constraint::Length(1),
    ])
    .split(inner);

    let boolean = |enabled: bool| {
        let (marker, label, color) = if enabled {
            ("[x]", m.enabled, t.status_pass)
        } else {
            ("[ ]", m.disabled, t.text_secondary)
        };
        Line::from(Span::styled(
            format!("{marker} {label}"),
            Style::default().fg(color),
        ))
    };
    let values = [
        boolean(app.use_nerd_fonts),
        Line::from(Span::styled(
            format!("< {} >", app.language.display_name()),
            Style::default().fg(t.status_info),
        )),
        boolean(app.mouse_scroll),
    ];
    let value_width = values.iter().map(Line::width).max().unwrap_or(0) as u16;
    let labels = [
        m.settings_nerd_fonts,
        m.settings_language,
        m.settings_mouse_scroll,
    ];
    let rows = labels.into_iter().zip(values).map(|(label, value)| {
        Row::new([
            Cell::from(label).style(Style::default().fg(t.text_primary)),
            Cell::from(value.alignment(Alignment::Right)),
        ])
        .bottom_margin(1)
    });
    let table = Table::new(rows, [Constraint::Min(0), Constraint::Length(value_width)])
        .column_spacing(2)
        .row_highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);
    let mut state = TableState::default().with_selected(app.selected_setting);
    frame.render_stateful_widget(table, chunks[0], &mut state);

    frame.render_widget(
        Paragraph::new("─".repeat(chunks[1].width as usize))
            .style(Style::default().fg(t.popup_separator)),
        chunks[1],
    );
    let description = match app.selected_setting {
        0 => m.settings_nerd_fonts_desc,
        1 => m.settings_language_desc,
        2 => m.settings_mouse_scroll_desc,
        _ => "",
    };
    frame.render_widget(
        Paragraph::new(description)
            .style(Style::default().fg(t.text_secondary))
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
    render_hint(
        frame,
        &[
            ("↑↓", m.help_move_updown),
            ("Space/←→", m.toggle),
            ("Enter", m.confirm),
            ("Esc", m.cancel),
        ],
        chunks[3],
    );
}
