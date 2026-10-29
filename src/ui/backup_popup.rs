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

//! Backup import popup: a path, then what the file holds and a warning that
//! it replaces everything.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;

use super::helpers::render_input_field;
use super::keyhints::render_hint;
use super::theme::theme;

pub fn draw_import_backup_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let term = frame.area();

    let width = 72u16.min(term.width.saturating_sub(4));
    let height = 11u16.min(term.height.saturating_sub(2));
    let area = Rect::new(
        term.width.saturating_sub(width) / 2,
        term.height.saturating_sub(height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);

    let border = if app.backup_preview.is_some() {
        t.popup_border_danger
    } else {
        t.popup_border
    };
    let block = Block::default()
        .title(format!(" {} ", m.backup_import))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // path
            Constraint::Min(0),    // preview, error or explanation
            Constraint::Length(1), // key hints
        ])
        .split(inner);

    render_input_field(
        frame,
        m.backup_path_label,
        &app.backup_path,
        app.backup_preview.is_none(),
        rows[0],
    );

    let body: Vec<Line> = if let Some(data) = &app.backup_preview {
        vec![
            Line::from(Span::styled(
                format!(
                    "{} {} \u{b7} {} {} \u{b7} {} {}",
                    data.semesters.len(),
                    m.backup_semesters,
                    data.course_count(),
                    m.course_plural,
                    data.user_templates.len(),
                    m.backup_templates
                ),
                Style::default()
                    .fg(t.text_primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                m.backup_replace_warning,
                Style::default().fg(t.status_warn),
            )),
        ]
    } else if let Some(err) = &app.backup_error {
        vec![Line::from(Span::styled(
            err.as_str(),
            Style::default().fg(t.status_fail),
        ))]
    } else {
        vec![Line::from(Span::styled(
            m.backup_import_explain,
            Style::default().fg(t.text_muted),
        ))]
    };
    frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: true }), rows[1]);

    let confirm = if app.backup_preview.is_some() {
        m.backup_replace
    } else {
        m.backup_read
    };
    render_hint(frame, &[("Enter", confirm), ("Esc", m.cancel)], rows[2]);
}
