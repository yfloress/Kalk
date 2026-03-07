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

//! User interface rendering using Ratatui.
//!
//! This module handles the main layout, panels, and footer.
//! Popup dialogs live in `popups`, rendering helpers and formatting in `helpers`.
//! All calculation logic lives in `model/` — this module only formats and renders.

pub(crate) mod helpers;
mod popups;

use crate::app::{App, Focus, Screen};
use crate::model::{MinimumNotMetAction, WeightValidation};
use helpers::{
    focused_border_style, format_course_average, format_course_status, format_weight_validation,
};
use popups::{
    draw_category_popup, draw_course_popup, draw_delete_popup, draw_delete_template_popup,
    draw_evaluation_popup, draw_language_popup, draw_save_template_popup, draw_template_popup,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
};

// =============================================================================
// Main Draw
// =============================================================================

/// Main UI rendering function.
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.size());

    // Main area: 3-column layout for Course -> Category -> Evaluation hierarchy
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ])
        .split(chunks[0]);

    draw_courses_panel(frame, app, main_chunks[0]);
    draw_categories_panel(frame, app, main_chunks[1]);
    draw_evaluations_panel(frame, app, main_chunks[2]);
    draw_footer(frame, app, chunks[1]);

    // Draw popups on top
    match &app.screen {
        Screen::SelectingTemplate => draw_template_popup(frame, app),
        Screen::EditingCourse { is_new } => draw_course_popup(frame, app, *is_new),
        Screen::EditingCategory { is_new } => draw_category_popup(frame, app, *is_new),
        Screen::EditingEvaluation { is_new } => {
            draw_evaluation_popup(frame, app, *is_new, !*is_new);
        }
        Screen::ConfirmDelete => draw_delete_popup(frame, app),
        Screen::ConfirmDeleteTemplate => draw_delete_template_popup(frame, app),
        Screen::SavingTemplate => draw_save_template_popup(frame, app),
        Screen::SelectingLanguage => draw_language_popup(frame, app),
        Screen::Main => {}
    }
}

// =============================================================================
// Panel Drawing
// =============================================================================

fn draw_courses_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let is_focused = app.focus == Focus::Courses;
    let border_style = focused_border_style(is_focused);

    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| {
            let status = format_course_status(c, m);

            // Weight validation indicator
            let weight_status = match c.validate_weights() {
                WeightValidation::Valid => Span::styled(" [OK]", Style::default().fg(Color::Green)),
                WeightValidation::Under(w) => {
                    Span::styled(format!(" [{:.0}%]", w), Style::default().fg(Color::Yellow))
                }
                WeightValidation::Over(w) => {
                    Span::styled(format!(" [{:.0}%!]", w), Style::default().fg(Color::Red))
                }
                WeightValidation::Empty => Span::styled(
                    format!(" [{}]", m.no_categories),
                    Style::default().fg(Color::DarkGray),
                ),
            };

            // Current grade color
            let grade_color = match c.current_grade() {
                Some(g) if c.is_passing_grade(g) => Color::Green,
                Some(_) => Color::Red,
                None => Color::DarkGray,
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                    weight_status,
                ]),
                Line::from(Span::styled(
                    format!("  {}", status),
                    Style::default().fg(grade_color),
                )),
            ])
        })
        .collect();

    let title = format!(" {} ({}) ", m.courses, app.courses.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(app.selected_course);

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_categories_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let is_focused = app.focus == Focus::Categories;
    let border_style = focused_border_style(is_focused);

    let Some(course) = app.current_course() else {
        let block = Block::default()
            .title(format!(" {} ", m.categories))
            .borders(Borders::ALL)
            .border_style(border_style);
        let paragraph = Paragraph::new(m.select_course_to_view)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    };

    // Split area for header info, course average, and category list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with weight info
            Constraint::Length(3), // Course average
            Constraint::Min(0),    // Category list
        ])
        .split(area);

    // Course info header with weight validation
    let validation = course.validate_weights();
    let validation_color = match &validation {
        WeightValidation::Valid => Color::Green,
        WeightValidation::Under(_) => Color::Yellow,
        WeightValidation::Over(_) => Color::Red,
        WeightValidation::Empty => Color::DarkGray,
    };

    let validation_msg = format_weight_validation(&validation, m);
    let header_text = format!(
        "{}: {:.0} | Total: {:.0}% | {}",
        m.passing_grade.split(' ').next().unwrap_or("Pass"),
        course.passing_grade,
        course.total_weight(),
        validation_msg
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(validation_color))
        .block(
            Block::default()
                .title(format!(" {} ", course.name))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
    frame.render_widget(header, chunks[0]);

    // Course total average with pass/fail status, rule overrides, and needs_global
    let grade_result = course.compute_grade();
    let (avg_text, avg_color) = format_course_average(course, &grade_result, m);
    let avg_block = Block::default()
        .title(format!(" {} ", m.course_average))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(avg_color));
    let avg_widget = Paragraph::new(avg_text)
        .style(Style::default().fg(avg_color))
        .block(avg_block);
    frame.render_widget(avg_widget, chunks[1]);

    // Category list — use grade_result to show failed minimums and eval violations
    let passing_grade = course.passing_grade;
    let items: Vec<ListItem> = course
        .categories
        .iter()
        .enumerate()
        .map(|(cat_idx, cat)| {
            let avg = cat
                .average_grade()
                .map(|g| format!("{:.1}", g))
                .unwrap_or_else(|| "-".to_string());

            let mut progress = format!(
                "{}/{} {}",
                cat.graded_count(),
                cat.evaluations.len(),
                m.graded
            );

            // Show drop count in progress
            if cat.rules.drop_lowest > 0 {
                progress.push_str(&format!(" (-{})", cat.rules.drop_lowest));
            }

            // Check if this category has a failed minimum in the grade result
            let failed_min = grade_result
                .failed_minimums
                .iter()
                .find(|fm| fm.category_idx == cat_idx);

            // Check if this category has eval violations in the grade result
            let has_eval_violations = grade_result
                .eval_violations
                .iter()
                .any(|ev| ev.category_idx == cat_idx && !ev.failing_indices.is_empty());

            // Color based on passing status, but override with Magenta if minimum not met
            let avg_color = if failed_min.is_some() {
                Color::Magenta
            } else {
                match cat.is_passing(passing_grade) {
                    Some(true) => Color::Green,
                    Some(false) => Color::Red,
                    None => Color::DarkGray,
                }
            };

            // Build name line with optional rules indicator
            let mut name_spans = vec![
                Span::styled(&cat.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" ({:.0}%)", cat.weight),
                    Style::default().fg(Color::Yellow),
                ),
            ];
            if !cat.rules.is_default() {
                name_spans.push(Span::styled(
                    format!(" [{}]", m.rules_active),
                    Style::default().fg(Color::Cyan),
                ));
            }
            if has_eval_violations {
                name_spans.push(Span::styled(
                    format!(" ({})", m.eval_below_min),
                    Style::default().fg(Color::Magenta),
                ));
            }

            // Build avg line with optional minimum warning
            let mut avg_spans = vec![
                Span::raw(format!("  {}: ", m.avg)),
                Span::styled(avg, Style::default().fg(avg_color)),
                Span::styled(
                    format!(" | {}", progress),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            if let Some(fm) = failed_min {
                let action_hint = match fm.action {
                    MinimumNotMetAction::FinalEqualsAverage => "",
                    MinimumNotMetAction::RequiresGlobal => " !G",
                };
                avg_spans.push(Span::styled(
                    format!(
                        " {:.1} < {:.0} {}{}",
                        fm.average, fm.required, m.minimum_not_met, action_hint
                    ),
                    Style::default().fg(Color::Magenta),
                ));
            }

            ListItem::new(vec![Line::from(name_spans), Line::from(avg_spans)])
        })
        .collect();

    let title = format!(" {} ({}) ", m.categories, course.categories.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(app.selected_category);

    frame.render_stateful_widget(list, chunks[2], &mut state);
}

fn draw_evaluations_panel(frame: &mut Frame, app: &App, area: Rect) {
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

    let passing_grade = app
        .current_course()
        .map(|c| c.passing_grade)
        .unwrap_or(55.0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Header with category info
    let avg = category
        .average_grade()
        .map(|g| format!("{:.1}", g))
        .unwrap_or_else(|| "-".to_string());

    // Compute grade result to access eval_violations for this category
    let grade_result = app.current_course().map(|c| c.compute_grade());
    let eval_violation = grade_result.as_ref().and_then(|gr| {
        selected_cat_idx.and_then(|ci| {
            gr.eval_violations
                .iter()
                .find(|ev| ev.category_idx == ci && !ev.failing_indices.is_empty())
        })
    });

    let avg_color = if category.meets_minimum() == Some(false) {
        Color::Magenta
    } else {
        match category.is_passing(passing_grade) {
            Some(true) => Color::Green,
            Some(false) => Color::Red,
            None => Color::DarkGray,
        }
    };

    let mut header_spans = vec![
        Span::raw(format!(
            "{} ({:.0}%) | {}: ",
            category.name, category.weight, m.avg
        )),
        Span::styled(avg, Style::default().fg(avg_color)),
        Span::raw(format!(
            " | {}/{} {}",
            category.graded_count(),
            category.evaluations.len(),
            m.graded
        )),
    ];

    if category.rules.drop_lowest > 0 {
        header_spans.push(Span::styled(
            format!(" (-{})", category.rules.drop_lowest),
            Style::default().fg(Color::Cyan),
        ));
    }

    // Show per-eval minimum violation info if present
    if let Some(ev) = &eval_violation {
        header_spans.push(Span::styled(
            format!(
                " | {} {}: {:.0}",
                ev.category_name, m.eval_below_min, ev.required
            ),
            Style::default().fg(Color::Magenta),
        ));
    }

    let header_text = Line::from(header_spans);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Precompute dropped indices and below-minimum indices
    let dropped_indices = category.dropped_indices();
    let below_min_indices = category.evals_below_minimum().unwrap_or_default();

    // Evaluations table — add a Status column for indicators
    let header_labels = ["#", m.name, m.grade, ""];
    let header_cells = header_labels
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
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

            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(e.name.as_str()),
                Cell::from(grade_str).style(grade_style),
                Cell::from(status).style(status_style),
            ])
            .style(style)
        })
        .collect();

    let title = format!(" {} ({}) ", m.evaluations, category.evaluations.len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(7),
            Constraint::Length(12),
        ],
    )
    .header(header_row)
    .block(
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

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
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
        Screen::EditingCourse { .. }
        | Screen::EditingCategory { .. }
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
