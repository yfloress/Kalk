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
//! This module handles the main layout and the courses/categories panels.
//! The evaluations panel and footer live in `panels`, popup dialogs in `popups`,
//! rendering helpers and formatting in `helpers`.
//! Icon sets live in `icons`, colour themes in `theme`.
//! All calculation logic lives in `model/` — this module only formats and renders.

pub(crate) mod helpers;
pub(crate) mod icons;
mod panels;
mod popups;
pub(crate) mod theme;

use crate::app::{App, Focus, Screen};
use crate::model::{Course, MinimumNotMetAction, WeightValidation};
use helpers::{
    focused_border_style, format_course_average, format_course_status, format_weight_validation,
};
use icons::icons;
use panels::{draw_evaluations_panel, draw_footer};
use popups::{
    draw_category_popup, draw_course_popup, draw_delete_popup, draw_delete_template_popup,
    draw_evaluation_popup, draw_language_popup, draw_save_template_popup, draw_settings_popup,
    draw_template_popup,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use theme::theme;

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
    // When compact_courses is active, the courses panel is narrower.
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.compact_courses {
            [
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(45),
            ]
        } else {
            [
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ]
        })
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
        Screen::EditingEvaluation { is_new } => draw_evaluation_popup(frame, app, *is_new),
        Screen::ConfirmDelete => draw_delete_popup(frame, app),
        Screen::ConfirmDeleteTemplate => draw_delete_template_popup(frame, app),
        Screen::SavingTemplate => draw_save_template_popup(frame, app),
        Screen::SelectingLanguage => draw_language_popup(frame, app),
        Screen::Settings => draw_settings_popup(frame, app),
        Screen::Main => {}
    }
}

// =============================================================================
// Panel Drawing
// =============================================================================

fn draw_courses_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let is_focused = app.focus == Focus::Courses;
    let border_style = focused_border_style(is_focused);

    let compact = app.compact_courses;

    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| {
            // Weight validation indicator
            let weight_status = match c.validate_weights() {
                WeightValidation::Valid => Span::styled(
                    format!(" [{}]", ic.weight_ok),
                    Style::default().fg(t.status_pass),
                ),
                WeightValidation::Under(w) => Span::styled(
                    format!(" [{}{:.0}%]", ic.weight_warn, w),
                    Style::default().fg(t.status_warn),
                ),
                WeightValidation::Over(w) => Span::styled(
                    format!(" [{}{:.0}%]", ic.weight_error, w),
                    Style::default().fg(t.status_fail),
                ),
                WeightValidation::Empty => Span::styled(
                    format!(" [{}]", m.no_categories),
                    Style::default().fg(t.text_muted),
                ),
            };

            // Current grade color
            let grade_color = match c.current_grade() {
                Some(g) if c.is_passing_grade(g) => t.status_pass,
                Some(_) => t.status_fail,
                None => t.text_muted,
            };

            if compact {
                // Compact: single line — "NAME [OK] 54.5→55"
                let short_grade = match c.current_grade() {
                    Some(g) => {
                        let r = Course::round_grade(g);
                        format!(" {:.0}\u{2192}{:.0}", g, r)
                    }
                    None => String::new(),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                    weight_status,
                    Span::styled(short_grade, Style::default().fg(grade_color)),
                ]))
            } else {
                // Normal: two lines
                let status = format_course_status(c, m);
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
            }
        })
        .collect();

    let title = format!(" {}{} ({}) ", ic.course, m.courses, app.courses.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let mut state = ListState::default();
    state.select(app.selected_course);

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_categories_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let is_focused = app.focus == Focus::Categories;
    let border_style = focused_border_style(is_focused);

    let Some(course) = app.current_course() else {
        let block = Block::default()
            .title(format!(" {}{} ", ic.category, m.categories))
            .borders(Borders::ALL)
            .border_type(t.border_type)
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
        WeightValidation::Valid => t.status_pass,
        WeightValidation::Under(_) => t.status_warn,
        WeightValidation::Over(_) => t.status_fail,
        WeightValidation::Empty => t.text_muted,
    };

    let validation_msg = format_weight_validation(&validation, m);
    let total_w = course.total_weight().abs();
    let header_text = format!(
        "{:.0} | {:.0}% | {}",
        course.passing_grade, total_w, validation_msg
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(validation_color))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(format!(" {} ", course.name))
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(border_style),
        );
    frame.render_widget(header, chunks[0]);

    // Course total average with pass/fail status, rule overrides, and needs_global
    let grade_result = course.compute_grade();
    let (avg_text, avg_color) = format_course_average(course, &grade_result, m);
    let avg_block = Block::default()
        .title(format!(" {} ", m.course_average))
        .borders(Borders::ALL)
        .border_type(t.border_type)
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

            // Color based on passing status, but override with override colour if minimum not met
            let avg_color = if failed_min.is_some() {
                t.status_override
            } else {
                match cat.is_passing(passing_grade) {
                    Some(true) => t.status_pass,
                    Some(false) => t.status_fail,
                    None => t.text_muted,
                }
            };

            // Build name line with optional rules indicator
            let mut name_spans = vec![
                Span::styled(&cat.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" ({:.0}%)", cat.weight),
                    Style::default().fg(t.weight_label),
                ),
            ];
            if !cat.rules.is_default() {
                name_spans.push(Span::styled(
                    format!(" [{}{}]", ic.rules_active, m.rules_active),
                    Style::default().fg(t.status_info),
                ));
            }
            if has_eval_violations {
                name_spans.push(Span::styled(
                    format!(" ({}{})", ic.below_min, m.eval_below_min),
                    Style::default().fg(t.status_override),
                ));
            }

            // Build avg line with optional minimum warning
            let mut avg_spans = vec![
                Span::raw(format!("  {}: ", m.avg)),
                Span::styled(avg, Style::default().fg(avg_color)),
                Span::styled(
                    format!(" | {}", progress),
                    Style::default().fg(t.text_muted),
                ),
            ];
            if let Some(fm) = failed_min {
                let action_hint = match fm.action {
                    MinimumNotMetAction::FinalEqualsAverage => "",
                    MinimumNotMetAction::RequiresGlobal => " !G",
                    MinimumNotMetAction::FailCourse => " !F",
                };
                avg_spans.push(Span::styled(
                    format!(
                        " {:.1} < {:.0} {}{}",
                        fm.average, fm.required, m.minimum_not_met, action_hint
                    ),
                    Style::default().fg(t.status_override),
                ));
            }

            ListItem::new(vec![Line::from(name_spans), Line::from(avg_spans)])
        })
        .collect();

    let title = format!(
        " {}{} ({}) ",
        ic.category,
        m.categories,
        course.categories.len()
    );
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let mut state = ListState::default();
    state.select(app.selected_category);

    frame.render_stateful_widget(list, chunks[2], &mut state);
}
