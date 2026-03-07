// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  Kyronix
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

//! Settings popup for the UI layer.
//!
//! Extracted from `popups.rs` to keep file sizes under the ~900-line limit.

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use super::icons::icons;
use super::theme::theme;

// =============================================================================
// Settings Popup
// =============================================================================

pub fn draw_settings_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);

    let popup_w = 56u16;
    let popup_h = 18u16;
    let term = frame.size();
    let x = term.x + term.width.saturating_sub(popup_w) / 2;
    let y = term.y + term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w.min(term.width), popup_h.min(term.height));
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {}{} ", ic.settings, m.settings))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // padding
            Constraint::Length(1), // setting 0: nerd fonts
            Constraint::Length(1), // setting 1: language
            Constraint::Length(1), // setting 2: compact courses
            Constraint::Length(1), // separator
            Constraint::Length(2), // description of selected setting
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // hint
        ])
        .split(inner);

    // --- Setting rows ---
    struct SettingRow<'a> {
        label: &'a str,
        value: String,
        value_color: ratatui::style::Color,
    }

    let rows = [
        SettingRow {
            label: m.settings_nerd_fonts,
            value: if app.use_nerd_fonts {
                m.enabled
            } else {
                m.disabled
            }
            .to_string(),
            value_color: if app.use_nerd_fonts {
                t.status_pass
            } else {
                t.text_muted
            },
        },
        SettingRow {
            label: m.settings_language,
            value: app.language.display_name().to_string(),
            value_color: t.status_info,
        },
        SettingRow {
            label: m.settings_compact_courses,
            value: if app.compact_courses {
                m.enabled
            } else {
                m.disabled
            }
            .to_string(),
            value_color: if app.compact_courses {
                t.status_pass
            } else {
                t.text_muted
            },
        },
    ];

    for (i, row) in rows.iter().enumerate() {
        let is_selected = app.selected_setting == i;
        let row_style = if is_selected {
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_selected { ic.highlight } else { "  " };

        let line = Line::from(vec![
            Span::styled(prefix, row_style),
            Span::styled(format!("{}: ", row.label), row_style.fg(t.text_primary)),
            Span::styled(
                format!("< {} >", row.value),
                Style::default().fg(row.value_color),
            ),
        ]);
        // chunks[1], chunks[2], chunks[3] for the 3 settings
        frame.render_widget(Paragraph::new(line), chunks[1 + i]);
    }

    // --- Separator ---
    let sep = Paragraph::new(Line::from(Span::styled(
        "  ──────────────────────────────────────────────",
        Style::default().fg(t.popup_separator),
    )));
    frame.render_widget(sep, chunks[4]);

    // --- Description of selected setting ---
    let desc_text = match app.selected_setting {
        0 => m.settings_nerd_fonts_desc,
        1 => app.language.display_name(),
        2 => m.settings_compact_courses_desc,
        _ => "",
    };
    let desc = Paragraph::new(Line::from(Span::styled(
        format!("  {}", desc_text),
        Style::default().fg(t.text_secondary),
    )))
    .wrap(Wrap { trim: true });
    frame.render_widget(desc, chunks[5]);

    // --- Bottom hint ---
    let hint = Line::from(vec![
        Span::styled(
            "  Space/\u{2190}\u{2192}",
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(": {}  ", m.toggle),
            Style::default().fg(t.footer_desc),
        ),
        Span::styled(
            "Enter",
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(": {}  ", m.confirm),
            Style::default().fg(t.footer_desc),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(": {}", m.cancel),
            Style::default().fg(t.footer_desc),
        ),
    ]);
    frame.render_widget(Paragraph::new(hint), chunks[7]);
}
