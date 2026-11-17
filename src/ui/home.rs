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

//! Home screen: the semester list and the dashboard metrics for the selected
//! one.  All figures come from `model::SemesterMetrics` — nothing is computed
//! here.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::app::App;

use super::helpers::{
    centered_rect, draw_scrollbar, focused_border_style, render_delete_confirmation,
    render_input_field,
};
use super::home_metrics::draw_metrics;
use super::icons::icons;
use super::keyhints::{render_hint, styled_keybindings};
use super::surfaces::popup_block;
use super::theme::theme;

/// Wordmark shown when there is nothing to report yet.
const WORDMARK: &str = r#" ___  __    ________  ___       ___  __
|\  \|\  \ |\   __  \|\  \     |\  \|\  \
\ \  \/  /|\ \  \|\  \ \  \    \ \  \/  /|_
 \ \   ___  \ \   __  \ \  \    \ \   ___  \
  \ \  \\ \  \ \  \ \  \ \  \____\ \  \\ \  \
   \ \__\\ \__\ \__\ \__\ \_______\ \__\\ \__\
    \|__| \|__|\|__|\|__|\|_______|\|__| \|__|"#;

/// The wordmark as styled lines, ready to render left-aligned.
pub(super) fn wordmark_lines(color: ratatui::style::Color) -> Vec<Line<'static>> {
    WORDMARK
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(color))))
        .collect()
}

/// Rows the wordmark needs, plus breathing room above and below.
const WORDMARK_ROWS: u16 = 11;

pub fn draw_home(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(super::panels::footer_height(app, frame.area())),
        ])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(chunks[0]);

    draw_semester_list(frame, app, columns[0]);
    draw_metrics(frame, app, columns[1]);
    draw_home_footer(frame, app, chunks[1]);
}

fn draw_semester_list(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);

    let items: Vec<ListItem> = app
        .semesters
        .iter()
        .map(|s| {
            let count = s.courses.len();
            Line::from(vec![
                Span::styled(&s.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}", count), Style::default().fg(t.text_muted)),
            ])
        })
        .map(ListItem::new)
        .collect();

    let title = format!(" {}{} ({}) ", ic.course, m.semesters, app.semesters.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(focused_border_style(true)),
        )
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let mut state = ListState::default();
    state.select(Some(app.selected_semester));
    frame.render_stateful_widget(list, area, &mut state);
    // Marks how far into the semester list the window sits.
    draw_scrollbar(
        frame,
        area,
        app.semesters.len(),
        area.height.saturating_sub(2) as usize,
        state.offset(),
    );
}

pub(super) fn draw_wordmark(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();

    let art_w = WORDMARK
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;

    // Too cramped for the art — the hint alone is more useful than a mangled
    // wordmark.
    if area.height < WORDMARK_ROWS || area.width < art_w {
        let hint = Paragraph::new(m.home_no_courses)
            .style(Style::default().fg(t.text_muted))
            .alignment(Alignment::Center);
        frame.render_widget(hint, area);
        return;
    }

    let art_h = WORDMARK.lines().count() as u16;
    let top = area.y + (area.height.saturating_sub(WORDMARK_ROWS)) / 2;

    // Centred as one block and drawn left-aligned inside it. Centring each
    // line individually would shift them apart and break the diagonals,
    // because the lines are not all the same width.
    let art_area = Rect {
        x: area.x + (area.width - art_w) / 2,
        y: top,
        width: art_w,
        height: art_h,
    };
    frame.render_widget(
        Paragraph::new(wordmark_lines(t.status_info)).alignment(Alignment::Left),
        art_area,
    );

    let hint_area = Rect {
        x: area.x,
        y: top + art_h + 1,
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(m.home_no_courses)
            .style(Style::default().fg(t.text_muted))
            .alignment(Alignment::Center),
        hint_area,
    );
}

fn draw_home_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Home is where a session starts, so a load error must surface here.
    // The banner gets a strip of its own so the keymap below it stays
    // visible; too short for both, the banner keeps the whole area.
    let mut target = area;
    if app.status_message.is_some() {
        match super::panels::status_and_footer_rows(area) {
            Some((banner_area, footer_area)) => {
                super::panels::draw_status_banner(frame, app, banner_area);
                target = footer_area;
            }
            None => {
                super::panels::draw_status_banner(frame, app, area);
                return;
            }
        }
    }

    let m = app.messages();
    let t = theme();

    // Only the essentials; renaming and reordering live in the `?` overlay.
    // `?` stays last so the width ladder never drops it.
    let keys = [
        ("j/k", m.help_move_updown),
        ("Enter", m.semester_open),
        ("n", m.semester_new),
        ("d", m.semester_delete),
        ("e", m.backup_export),
        ("i", m.backup_import),
        ("Esc", m.help_close_popup),
        ("?", m.help_open),
    ];

    let badge = super::panels::simulation_badge(app);
    let width = area
        .width
        .saturating_sub(2)
        .saturating_sub(badge.as_ref().map_or(0, |b| b.width() as u16 + 1));
    let line = super::panels::with_badge(styled_keybindings(&keys, t, width), badge);

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(t.border_type)
            .border_style(Style::default().fg(t.footer_border)),
    );
    frame.render_widget(footer, target);
}

// =============================================================================
// Semester popups
// =============================================================================

/// Single-field form for creating or renaming a semester.
pub fn draw_semester_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();

    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let title = if is_new {
        m.semester_new
    } else {
        m.semester_rename
    };
    let block = popup_block(format!(" {} ", title));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    render_input_field(frame, m.semester_name_label, &app.edit_name, true, rows[0]);

    render_hint(frame, &[("Enter", m.confirm), ("Esc", m.cancel)], rows[1]);
}

pub fn draw_delete_semester_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let semester = app.current_semester();
    let count = semester.courses.len();
    let label = if count == 1 {
        m.course_singular
    } else {
        m.course_plural
    };

    let warning = format!("{} ({} {})", m.semester_delete_warning, count, label);

    render_delete_confirmation(
        frame,
        m.confirm_delete,
        m.semester_delete,
        &warning,
        m.confirm,
        m.cancel,
        app.use_nerd_fonts,
    );
}
