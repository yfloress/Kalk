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

//! The `?` cheat-sheet overlay: every shortcut, grouped by purpose.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;

use super::keyhints::render_hint;
use super::theme::theme;

/// Render the global keyboard cheat-sheet overlay.
/// Groups shortcuts by purpose so the user can scan instead of read top-down.
pub fn draw_help_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let term = frame.area();

    // Centred popup with breathing room around the content.
    let popup_w = 62u16.min(term.width.saturating_sub(2));
    let popup_h = 38u16.min(term.height.saturating_sub(2));
    let x = term.width.saturating_sub(popup_w) / 2;
    let y = term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w, popup_h);

    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {} ", m.help_title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let key_style = Style::default()
        .fg(t.footer_key)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(t.text_primary);
    let group_style = Style::default()
        .fg(t.status_info)
        .add_modifier(Modifier::BOLD);

    let row = |key: &str, desc: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {:<12} ", key), key_style),
            Span::styled(desc.to_string(), desc_style),
        ])
    };
    let group =
        |title: &str| -> Line<'static> { Line::from(Span::styled(title.to_string(), group_style)) };
    let blank = || Line::from("");

    let undo_redo = format!("{} / {}", m.undo, m.redo);
    let copy_paste = format!("{} / {}", m.yank, m.paste);

    let lines: Vec<Line> = vec![
        group(m.help_group_global),
        row("q", m.quit),
        row("?", m.help_open),
        row("Esc", m.help_close_popup),
        blank(),
        group(m.semesters),
        row("h", m.home_open),
        row("l / Enter", m.semester_open),
        row("n / r / d", m.semester_manage),
        row("J / K", m.semester_move),
        blank(),
        group(m.help_group_navigation),
        row("k / j  ↑/↓", m.help_move_updown),
        row("h / l  ←/→", m.help_focus_lr),
        row("h", m.home_open),
        row("Tab", m.help_cycle_focus),
        row("Home / End", m.jump_first_last),
        blank(),
        group(m.help_group_editing),
        row("n", m.help_new_generic),
        row("Enter", m.help_edit_selected),
        row("d", m.help_delete_selected),
        row("Ctrl+N", m.bulk_add),
        row("Ctrl+Z / Y", &undo_redo),
        blank(),
        group(m.help_group_actions),
        row("g", m.enter_global),
        row("t", m.save_as_template),
        row("b", m.balance),
        row("y / p", &copy_paste),
        blank(),
        group(m.help_group_view),
        row("S", m.settings),
        row("L", m.change_language),
    ];

    // The body scrolls under its own hint line, so reserve a row for it.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, rows[0]);
    render_hint(frame, &[("Esc/Enter/?", m.help_close_hint)], rows[1]);
}
