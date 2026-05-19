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

//! Three-step AI import wizard popups.
//!
//! The wizard is invoked when the synthetic "Create with AI" entry at index 0
//! of the template selector is confirmed:
//!
//! 1. [`draw_import_prompt`] — shows the prompt and copies it on `c`.
//! 2. [`draw_import_paste`]  — waits for a bracketed-paste event from the user.
//! 3. [`draw_import_preview`] — preview of the parsed course; `Enter` commits.

use super::helpers::centered_rect;
use super::theme::theme;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

// =============================================================================
// Step 1 — Prompt
// =============================================================================

pub fn draw_import_prompt(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step1_title),
            Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // intro hint
            Constraint::Min(0),    // prompt body
            Constraint::Length(2), // feedback + footer
        ])
        .split(inner);

    let hint = Paragraph::new(m.import_step1_hint)
        .style(Style::default().fg(t.text_secondary))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, chunks[0]);

    let prompt = Paragraph::new(m.import_prompt)
        .style(Style::default().fg(t.text_primary))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(Style::default().fg(t.border_unfocused)),
        );
    frame.render_widget(prompt, chunks[1]);

    let mut spans: Vec<Span<'_>> = Vec::new();
    if app.import_copied {
        spans.push(Span::styled(
            format!("✓ {}  ", m.import_step1_copied),
            Style::default().fg(t.status_pass).add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::styled(
        m.import_step1_copy_key,
        Style::default().fg(t.footer_key),
    ));
    spans.push(Span::raw("   "));
    spans.push(Span::styled(
        m.import_step1_next,
        Style::default().fg(t.footer_key),
    ));
    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, chunks[2]);
}

// =============================================================================
// Step 2 — Paste
// =============================================================================

pub fn draw_import_paste(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step2_title),
            Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // intro hint
            Constraint::Min(0),    // error area
            Constraint::Length(1), // footer
        ])
        .split(inner);

    let hint = Paragraph::new(m.import_step2_hint)
        .style(Style::default().fg(t.text_secondary))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, chunks[0]);

    let body = if let Some(err) = &app.import_paste_error {
        Paragraph::new(err.as_str())
            .style(Style::default().fg(t.status_fail))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("…").style(Style::default().fg(t.text_muted))
    };
    frame.render_widget(body, chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(m.import_step2_back, Style::default().fg(t.footer_key)),
    ]));
    frame.render_widget(footer, chunks[2]);
}

// =============================================================================
// Step 3 — Preview
// =============================================================================

pub fn draw_import_preview(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(70, 75, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step3_title),
            Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(course) = &app.import_parsed else {
        let p = Paragraph::new("…").style(Style::default().fg(t.text_muted));
        frame.render_widget(p, inner);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    // Course header — name (+ rename note) and passing grade.
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", m.name),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            course.name.clone(),
            Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(from) = &app.import_renamed_from {
        lines.push(Line::from(Span::styled(
            format!("  ({} {} → {})", m.import_renamed_to, from, course.name),
            Style::default().fg(t.status_warn),
        )));
    }
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", m.passing_grade),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            format!("{:.0}", course.passing_grade),
            Style::default().fg(t.text_primary),
        ),
    ]));

    // Weight summary line.
    let weight_color = if (app.import_total_weight - 100.0).abs() < 0.01 {
        t.status_pass
    } else {
        t.status_warn
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", m.import_step3_total_weight),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            format!("{:.0}%", app.import_total_weight),
            Style::default().fg(weight_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   ·   {} {}", app.import_total_evals, m.import_step3_evaluations),
            Style::default().fg(t.text_muted),
        ),
    ]));

    if (app.import_total_weight - 100.0).abs() >= 0.01 {
        lines.push(Line::from(Span::styled(
            format!("⚠ {}", m.import_warning_weight_not_100),
            Style::default().fg(t.status_warn),
        )));
    }

    lines.push(Line::from(""));

    // Categories listing.
    for cat in &course.categories {
        let mut header_spans = vec![
            Span::styled(
                "▎ ",
                Style::default().fg(t.popup_border),
            ),
            Span::styled(
                cat.name.clone(),
                Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({:.0}%)", cat.weight),
                Style::default().fg(t.weight_label),
            ),
        ];
        if cat.rules.drop_lowest > 0 {
            header_spans.push(Span::styled(
                format!("  -{}", cat.rules.drop_lowest),
                Style::default().fg(t.status_override),
            ));
        }
        lines.push(Line::from(header_spans));
        lines.push(Line::from(Span::styled(
            format!(
                "    {} {}",
                cat.evaluations.len(),
                m.import_step3_evaluations
            ),
            Style::default().fg(t.text_muted),
        )));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let body = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(body, chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(m.import_step2_back, Style::default().fg(t.footer_key)),
        Span::styled("   ", Style::default()),
        Span::styled(
            m.import_step3_save_as_template,
            Style::default().fg(t.footer_key),
        ),
        Span::styled("   ", Style::default()),
        Span::styled(
            m.import_step3_confirm,
            Style::default().fg(t.status_pass).add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(footer, chunks[1]);
}

