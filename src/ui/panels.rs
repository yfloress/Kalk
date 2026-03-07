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

//! Extracted panel rendering functions.
//!
//! Contains `draw_evaluations_panel` and `draw_footer`, split out from
//! `ui/mod.rs` to keep file sizes under the ~600-line guideline.

use super::helpers::focused_border_style;
use crate::app::{App, Focus, Screen};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

// =============================================================================
// Evaluations Panel
// =============================================================================

pub fn draw_evaluations_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let is_focused = app.focus == Focus::Evaluations;
    let border_style = focused_border_style(is_focused);
    let selected_cat_idx = app.selected_category;

    let Some(category) = app.current_category() else {
        let block = Block::default()
            .title(format!(" {} ", m.evaluations))
            .borders(Borders::ALL)
            .border_style(border_style);

        let message = if app.current_course().is_some() {
            m.select_category_to_view
        } else {
            m.select_course_first
        };

        let paragraph = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    };

    // Safety: current_category() returned Some above, which guarantees current_course() is Some.
    let Some(course) = app.current_course() else {
        return;
    };
    let passing_grade = course.passing_grade;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Header with category info
    let avg = category
        .average_grade()
        .map(|g| format!("{:.1}", g))
        .unwrap_or_else(|| "-".to_string());

    let avg_color = if category.meets_minimum() == Some(false) {
        Color::Magenta
    } else {
        match category.is_passing(passing_grade) {
            Some(true) => Color::Green,
            Some(false) => Color::Red,
            None => Color::DarkGray,
        }
    };

    // Compute grade result to check for eval violations in this category
    let grade_result = course.compute_grade();
    let eval_violation = selected_cat_idx.and_then(|ci| {
        grade_result
            .eval_violations
            .iter()
            .find(|ev| ev.category_idx == ci && !ev.failing_indices.is_empty())
    });

    let drop_hint = if category.rules.drop_lowest > 0 {
        format!("(-{})", category.rules.drop_lowest)
    } else {
        String::new()
    };

    let header_spans = vec![
        Span::styled(format!("{}: ", m.avg), Style::default().fg(Color::DarkGray)),
        Span::styled(avg, Style::default().fg(avg_color)),
        Span::styled(
            format!(
                " | {}/{}{}",
                category.graded_count(),
                category.evaluations.len(),
                if drop_hint.is_empty() {
                    String::new()
                } else {
                    format!(" {}", drop_hint)
                }
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let header_title = if let Some(ev) = &eval_violation {
        format!(
            " {} ({:.0}%) | {}>={:.0} ",
            ev.category_name, category.weight, m.eval_below_min, ev.required
        )
    } else {
        format!(" {} ({:.0}%) ", category.name, category.weight)
    };
    let header_text = Line::from(header_spans);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .title(header_title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Precompute dropped indices and below-minimum indices
    let dropped_indices = category.dropped_indices();
    let below_min_indices = category.evals_below_minimum().unwrap_or_default();

    // Check if any evaluation has a status indicator (dropped or below min)
    let has_status = !dropped_indices.is_empty() || !below_min_indices.is_empty();

    // Evaluations table — add a Status column only when needed
    let header_cells: Vec<Cell> = if has_status {
        vec!["#", m.name, m.grade, ""]
    } else {
        vec!["#", m.name, m.grade]
    }
    .into_iter()
    .map(|h| Cell::from(h).style(Style::default().add_modifier(Modifier::BOLD)))
    .collect();
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = category
        .evaluations
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let is_selected = app.selected_evaluation == Some(i) && is_focused;
            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let is_dropped = dropped_indices.contains(&i);
            let is_below_min = below_min_indices.contains(&i);

            let grade_str = e
                .grade
                .map(|g| format!("{:.0}", g))
                .unwrap_or_else(|| "-".to_string());

            let grade_style = if is_dropped {
                Style::default().fg(Color::DarkGray)
            } else if is_below_min {
                Style::default().fg(Color::Magenta)
            } else {
                match e.grade {
                    Some(g) if g >= passing_grade => Style::default().fg(Color::Green),
                    Some(_) => Style::default().fg(Color::Red),
                    None => Style::default().fg(Color::DarkGray),
                }
            };

            // Status indicator
            let status = if is_dropped {
                format!("({})", m.dropped)
            } else if is_below_min {
                format!("({})", m.eval_below_min)
            } else {
                String::new()
            };
            let status_style = if is_dropped {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Magenta)
            };

            let mut cells = vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(e.name.as_str()),
                Cell::from(grade_str).style(grade_style),
            ];
            if has_status {
                cells.push(Cell::from(status).style(status_style));
            }
            Row::new(cells).style(style)
        })
        .collect();

    let title = format!(" {} ({}) ", m.evaluations, category.evaluations.len());
    let col_widths: Vec<Constraint> = if has_status {
        vec![
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
            Constraint::Length(10),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
        ]
    };
    let table = Table::new(rows, col_widths).header(header_row).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(table, chunks[1]);
}

// =============================================================================
// Footer
// =============================================================================

pub fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();

    // If there's a status message (error/info), show it prominently
    if let Some(ref status) = app.status_message {
        let is_error = status.starts_with("Error");
        let color = if is_error { Color::Red } else { Color::Yellow };

        let footer = Paragraph::new(status.as_str())
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            );
        frame.render_widget(footer, area);
        return;
    }

    let keys = match &app.screen {
        Screen::Main => match app.focus {
            Focus::Courses => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | t: {} | b: {} | Ctrl+L: {}",
                    m.quit,
                    m.new,
                    m.edit,
                    m.delete,
                    m.save_as_template,
                    m.balance,
                    m.change_language
                )
            }
            Focus::Categories => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | Ctrl+L: {}",
                    m.quit, m.new_category, m.edit, m.delete, m.change_language
                )
            }
            Focus::Evaluations => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | Ctrl+L: {}",
                    m.quit, m.new_eval, m.edit, m.delete, m.change_language
                )
            }
        },
        Screen::SelectingTemplate => {
            if app.is_user_template_selected() {
                format!(
                    "↑/↓/Tab: {} | Enter: {} | d: {} | Esc: {}",
                    m.select, m.confirm, m.delete, m.cancel
                )
            } else {
                format!(
                    "↑/↓/Tab: {} | Enter: {} | Esc: {}",
                    m.select, m.confirm, m.cancel
                )
            }
        }
        Screen::SelectingLanguage => {
            format!(
                "↑/↓/Tab: {} | Enter: {} | Esc: {}",
                m.select, m.confirm, m.cancel
            )
        }
        Screen::EditingCategory { .. } => {
            let rules_hint = if app.show_advanced_rules {
                m.advanced_rules_hide
            } else {
                m.advanced_rules_show
            };
            format!(
                "Tab: {} | Enter: {} | Esc: {} | {} | {}",
                m.next_field, m.confirm, m.cancel, rules_hint, m.help_toggle
            )
        }
        Screen::EditingCourse { .. }
        | Screen::EditingEvaluation { .. }
        | Screen::SavingTemplate => {
            format!(
                "Tab: {} | Enter: {} | Esc: {}",
                m.next_field, m.confirm, m.cancel
            )
        }
        Screen::ConfirmDelete | Screen::ConfirmDeleteTemplate => {
            format!("Enter/y: {} | Esc/n: {}", m.confirm, m.cancel)
        }
    };

    let footer = Paragraph::new(keys)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(footer, area);
}
