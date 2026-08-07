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

//! Attendance popup: classes held, classes missed, the requirement, and what
//! missing it does. Shows the resulting percentage live, so the user never has
//! to work it out themselves.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, InputField};
use crate::model::AttendanceAction;

use super::helpers::{centered_rect, render_input_field, render_toggle_field};
use super::keyhints::render_hint;
use super::theme::theme;

pub fn draw_attendance_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();

    let area = centered_rect(55, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.attendance_title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([
            Constraint::Length(3), // classes held
            Constraint::Length(3), // classes missed
            Constraint::Length(3), // required percentage
            Constraint::Length(3), // consequence
            Constraint::Length(1), // spacing
            Constraint::Length(1), // derived percentage
            Constraint::Min(0),
            Constraint::Length(1), // key hints
        ])
        .split(inner);

    render_input_field(
        frame,
        m.attendance_total,
        &app.edit_classes_total,
        app.input_field == InputField::ClassesTotal,
        rows[0],
    );
    render_input_field(
        frame,
        m.attendance_missed,
        &app.edit_classes_missed,
        app.input_field == InputField::ClassesMissed,
        rows[1],
    );
    render_input_field(
        frame,
        m.attendance_required,
        &app.edit_attendance_required,
        app.input_field == InputField::AttendanceRequired,
        rows[2],
    );

    let action_label = match app.edit_attendance_action {
        AttendanceAction::WarnOnly => m.attendance_warn_only,
        AttendanceAction::FailCourse => m.attendance_fails_course,
    };
    render_toggle_field(
        frame,
        m.attendance_if_not_met,
        action_label,
        app.input_field == InputField::AttendanceAction,
        rows[3],
    );

    frame.render_widget(
        Paragraph::new(summary(app, m)).alignment(Alignment::Center),
        rows[5],
    );

    render_hint(
        frame,
        &[
            ("Tab", m.help_cycle_focus),
            ("Enter", m.confirm),
            ("Esc", m.cancel),
        ],
        rows[7],
    );
}

/// The percentage the typed numbers produce, and how many classes are left to
/// spare — the figure the student actually wants.
fn summary(app: &App, m: &crate::i18n::Messages) -> Line<'static> {
    let t = theme();
    let attendance = app.previewed_attendance();

    let Some(percent) = attendance.percent() else {
        return Line::from(Span::styled(
            m.attendance_untracked,
            Style::default().fg(t.text_muted),
        ));
    };

    let color = if attendance.is_below_requirement() {
        t.status_fail
    } else {
        t.status_pass
    };

    let mut spans = vec![Span::styled(
        format!("{:.0}%", percent),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )];

    if let Some(left) = attendance.misses_remaining() {
        spans.push(Span::styled(
            format!("  ·  {} {}", left, m.attendance_can_still_miss),
            Style::default().fg(t.text_muted),
        ));
    }

    Line::from(spans)
}
