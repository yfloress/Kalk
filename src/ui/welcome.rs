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

//! First-run wizard rendering: language, then a live Nerd Font check.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::i18n::Language;

use super::home::wordmark_lines;
use super::icons::icons;
use super::theme::theme;

pub fn draw_welcome(frame: &mut Frame, app: &App, fonts_step: bool) {
    let t = theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    let inner = block.inner(frame.area());
    frame.render_widget(block, frame.area());

    let art = wordmark_lines(t.status_info);
    let art_h = art.len() as u16;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(art_h + 2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    draw_art(frame, &art, rows[0]);

    if fonts_step {
        draw_font_step(frame, app, rows[1]);
    } else {
        draw_language_step(frame, app, rows[1]);
    }

    draw_hint(frame, app, fonts_step, rows[2]);
}

/// Centred as one block and left-aligned inside it, so the diagonals hold.
fn draw_art(frame: &mut Frame, art: &[Line<'static>], area: Rect) {
    let width = art
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.chars().count()).sum())
        .max()
        .unwrap_or(0) as u16;

    if area.width < width || area.height < art.len() as u16 {
        return;
    }

    let target = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + 1,
        width,
        height: art.len() as u16,
    };
    frame.render_widget(
        Paragraph::new(art.to_vec()).alignment(Alignment::Left),
        target,
    );
}

fn draw_language_step(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);

    let mut lines = vec![
        Line::from(Span::styled(
            m.welcome_language_title,
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, lang) in Language::all().iter().enumerate() {
        let selected = i == app.selected_language;
        let prefix = if selected { ic.selected } else { ic.unselected };
        let style = if selected {
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_secondary)
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, lang.display_name()),
            style,
        )));
    }

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_font_step(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();

    // Rendered with icons forced on: the whole point is to let the user see
    // whether their terminal font can draw them.
    let nerd = icons(true);
    let sample = format!(
        "{}  {}  {}  {}",
        nerd.course, nerd.passed, nerd.failed, nerd.needs_global
    );

    let mut lines = vec![
        Line::from(Span::styled(
            m.welcome_fonts_title,
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            sample,
            Style::default()
                .fg(t.status_info)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            m.welcome_fonts_question,
            Style::default().fg(t.text_secondary),
        )),
        Line::from(""),
    ];

    let choice = |label: &str, active: bool| {
        if active {
            Span::styled(
                format!("[ {} ]", label),
                Style::default()
                    .fg(t.footer_key)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!("  {}  ", label), Style::default().fg(t.text_muted))
        }
    };
    lines.push(Line::from(vec![
        choice(m.welcome_yes, app.use_nerd_fonts),
        Span::raw("    "),
        choice(m.welcome_no, !app.use_nerd_fonts),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        m.welcome_fonts_hint,
        Style::default().fg(t.text_muted),
    )));

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_hint(frame: &mut Frame, app: &App, fonts_step: bool, area: Rect) {
    let m = app.messages();
    let t = theme();

    let yes_no = format!("{} / {}", m.welcome_yes, m.welcome_no);
    let keys: Vec<(&str, &str)> = if fonts_step {
        vec![
            ("h/l", yes_no.as_str()),
            ("Enter", m.welcome_start),
            ("Esc", m.welcome_back),
        ]
    } else {
        vec![("j/k", m.help_move_updown), ("Enter", m.welcome_next)]
    };

    let mut spans = Vec::new();
    for (i, (key, label)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(Span::styled("[", Style::default().fg(t.footer_border)));
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("] ", Style::default().fg(t.footer_border)));
        spans.push(Span::styled(*label, Style::default().fg(t.footer_desc)));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}
