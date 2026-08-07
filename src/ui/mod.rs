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

//! User interface rendering using Ratatui.
//!
//! This module handles the main layout and the courses/categories panels.
//! The evaluations panel and footer live in `panels`, popup dialogs in `popups`,
//! rendering helpers and formatting in `helpers`.
//! Icon sets live in `icons`, colour themes in `theme`.
//! All calculation logic lives in `model/` — this module only formats and renders.

mod eval_popups;
pub(crate) mod helpers;
mod home;
pub(crate) mod icons;
mod panels;
mod popups;
mod popups_import;
mod settings_popup;
pub(crate) mod theme;

use crate::app::{App, Focus, Screen};
use crate::model::{
    Course, CourseOutcome, GlobalExamPolicy, MAX_GRADE, NeededGradeStatus, WeightValidation,
};
use eval_popups::{draw_bulk_add_popup, draw_evaluation_popup, draw_global_grade_popup};
use helpers::{focused_border_style, format_course_average, format_weight_validation};
use home::{draw_delete_semester_popup, draw_semester_popup};
use icons::icons;
use panels::{draw_evaluations_panel, draw_footer};
use popups::{
    draw_category_popup, draw_course_popup, draw_delete_popup, draw_delete_template_popup,
    draw_help_popup, draw_language_popup, draw_save_template_popup, draw_template_popup,
};
use popups_import::{draw_import_paste, draw_import_preview, draw_import_prompt};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use settings_popup::draw_settings_popup;
use theme::theme;

// =============================================================================
// Main Draw
// =============================================================================

/// Main UI rendering function.
pub fn draw(frame: &mut Frame, app: &App) {
    // Home replaces the whole layout rather than overlaying it, and its own
    // popups belong on top of Home — not on top of the three-panel view.
    if matches!(
        app.screen,
        Screen::Home | Screen::EditingSemester { .. } | Screen::ConfirmDeleteSemester
    ) {
        home::draw_home(frame, app);
        match &app.screen {
            Screen::EditingSemester { is_new } => draw_semester_popup(frame, app, *is_new),
            Screen::ConfirmDeleteSemester => draw_delete_semester_popup(frame, app),
            _ => {}
        }
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    // Main area: 3-column layout for Course -> Category -> Evaluation hierarchy.
    // The courses panel always uses the compact single-line layout.
    let ic = icons(app.use_nerd_fonts);
    let m = app.messages();

    let highlight_len = ic.highlight.chars().count() as u16;
    let max_name_len = app
        .courses()
        .iter()
        .map(|c| c.name.chars().count() as u16)
        .max()
        .unwrap_or(4);

    // Minimum width = title text so it never gets clipped.
    // Title: " {icon}Courses (N) " + 2 border columns.
    let title_width = {
        let count_digits = if app.courses().is_empty() {
            1
        } else {
            (app.courses().len() as f64).log10().floor() as u16 + 1
        };
        // " " + icon + label + " (" + digits + ") " + borders
        1 + ic.course.chars().count() as u16
            + m.courses.chars().count() as u16
            + 2
            + count_digits
            + 2
            + 2
    };

    // Compact courses column: accent(2) + name + space + grade(3) + chip(6 max
    // for " 99/99") + borders(2) + highlight + pad(1).
    let needed = 2 + max_name_len + 1 + 3 + 6 + 2 + highlight_len + 1;
    let courses_constraint = Constraint::Length(needed.max(title_width).clamp(12, 38));

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            courses_constraint,
            Constraint::Percentage(45),
            Constraint::Min(0),
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
        Screen::EditingEvaluation { is_new } => draw_evaluation_popup(frame, app, *is_new),
        Screen::ConfirmDelete => draw_delete_popup(frame, app),
        Screen::ConfirmDeleteTemplate => draw_delete_template_popup(frame, app),
        Screen::SavingTemplate => draw_save_template_popup(frame, app),
        Screen::SelectingLanguage => draw_language_popup(frame, app),
        Screen::Settings => draw_settings_popup(frame, app),
        Screen::BulkAddEvaluations => draw_bulk_add_popup(frame, app),
        Screen::EnteringGlobalGrade => draw_global_grade_popup(frame, app),
        Screen::Help => draw_help_popup(frame, app),
        Screen::ImportPrompt => draw_import_prompt(frame, app),
        Screen::ImportPaste => draw_import_paste(frame, app),
        Screen::ImportPreview => draw_import_preview(frame, app),
        Screen::Home
        | Screen::EditingSemester { .. }
        | Screen::ConfirmDeleteSemester
        | Screen::Main => {}
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

    let items: Vec<ListItem> = app
        .courses()
        .iter()
        .map(|c| {
            let true_color = match c.outcome() {
                CourseOutcome::NoData => t.text_muted,
                CourseOutcome::Passing => t.status_pass,
                CourseOutcome::Failing => t.status_fail,
                CourseOutcome::PendingGlobal => t.status_override,
            };

            // Coloured accent bar shown before each course so the user can
            // scan the list and see pass/fail/no-data status at a glance.
            let accent = Span::styled("\u{258E} ", Style::default().fg(true_color));

            // Progress chip — "graded/total" across every category of the
            // course.  Lets the user see how far through the semester each
            // course is without having to open it.
            let graded_count: usize = c.categories.iter().map(|cat| cat.graded_count()).sum();
            let total_count: usize = c.categories.iter().map(|cat| cat.evaluations.len()).sum();

            // Single line — "NAME GRADE X/Y" with colour-coded grade and a
            // muted progress chip.
            let short_grade = match c.final_grade() {
                Some(grade) => format!(" {:.0}", Course::round_grade(grade)),
                None => " -".to_string(),
            };
            let mut spans = vec![
                accent,
                Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(short_grade, Style::default().fg(true_color)),
            ];
            if total_count > 0 {
                spans.push(Span::styled(
                    format!(" {}/{}", graded_count, total_count),
                    Style::default().fg(t.text_muted),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" {}{} ({}) ", ic.course, m.courses, app.courses().len());
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
        "{:.0} · {:.0}% · {}",
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

    // Course total average with pass/fail status, rule overrides, and needs_global.
    // The line carries per-span colours so the descriptive label stays neutral
    // and only the value portion turns red/green.  The title is forced to the
    // primary text colour so it doesn't inherit the accent from the paragraph.
    let grade_result = course.compute_grade();
    let (avg_line, avg_color) = format_course_average(course, &grade_result, m, app.use_nerd_fonts);
    let avg_title = Line::from(Span::styled(
        format!(" {} ", m.course_average),
        Style::default().fg(t.text_primary),
    ));
    let avg_block = Block::default()
        .title(avg_title)
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(avg_color));
    let avg_widget = Paragraph::new(avg_line).block(avg_block);
    frame.render_widget(avg_widget, chunks[1]);

    // Category list — use grade_result to show failed minimums and eval violations
    let passing_grade = course.passing_grade;
    // Inner width of the categories panel (excluding borders)
    let panel_inner_w = area.width.saturating_sub(2) as usize;
    let narrow = panel_inner_w < 50;
    let mut items: Vec<ListItem> = course
        .categories
        .iter()
        .enumerate()
        .map(|(cat_idx, cat)| {
            let avg = cat
                .average_grade()
                .map(|g| format!("{:.1}", g))
                .unwrap_or_else(|| "-".to_string());

            let mut progress = if narrow {
                format!("{}/{}", cat.graded_count(), cat.evaluations.len())
            } else {
                format!(
                    "{}/{} {}",
                    cat.graded_count(),
                    cat.evaluations.len(),
                    m.graded
                )
            };

            // Show drop count in progress
            if cat.rules.drop_lowest > 0 {
                progress.push_str(&format!(" (-{})", cat.rules.drop_lowest));
            }

            // Check if this category has a failed minimum in the grade result.
            // Skip per-eval violations here — they are already shown via the
            // "(! bajo min)" indicator on the name line.  Showing them again
            // with "avg < required" produces confusing text (e.g. "78.0 < 50").
            let failed_min = grade_result
                .failed_minimums
                .iter()
                .find(|fm| fm.category_idx == cat_idx && !fm.from_per_eval);

            // Check if this category has eval violations in the grade result
            let eval_violation = grade_result
                .eval_violations
                .iter()
                .find(|ev| ev.category_idx == cat_idx && !ev.failing_indices.is_empty());
            let has_eval_violations = eval_violation.is_some();

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

            // Coloured accent bar mirrors the one on the courses panel — it
            // reflects this category's pass/fail/override status at a glance.
            let accent = Span::styled("\u{258E} ", Style::default().fg(avg_color));

            // Build name line with optional rules indicator
            let mut name_spans = vec![
                accent,
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
                // In narrow mode, just show the icon; in wide mode, show icon + label
                let violation_text = if narrow {
                    format!(" {}", ic.below_min)
                } else {
                    format!(" ({}{})", ic.below_min, m.eval_below_min)
                };
                name_spans.push(Span::styled(
                    violation_text,
                    Style::default().fg(t.status_override),
                ));
            }

            // Build avg line with optional minimum warning. The extra two
            // spaces of indent align this line with the start of the name
            // after the accent bar above.
            let mut avg_spans = vec![
                Span::raw(format!("    {}: ", m.avg)),
                Span::styled(avg, Style::default().fg(avg_color)),
                Span::styled(
                    format!(" · {}", progress),
                    Style::default().fg(t.text_muted),
                ),
            ];
            if let Some(fm) = failed_min {
                // In narrow mode, show a short hint; in wide mode, show full text
                let min_text = if narrow {
                    format!(" <{:.0}", fm.required)
                } else {
                    format!(
                        " {:.1} < {:.0} {}",
                        fm.average, fm.required, m.minimum_not_met
                    )
                };
                avg_spans.push(Span::styled(
                    min_text,
                    Style::default().fg(t.status_override),
                ));
            } else if let Some(ev) = eval_violation {
                // Per-eval violation: show "<REQUIRED"
                avg_spans.push(Span::styled(
                    format!(" <{:.0}", ev.required),
                    Style::default().fg(t.status_override),
                ));
            }

            ListItem::new(vec![Line::from(name_spans), Line::from(avg_spans)])
        })
        .collect();

    // Virtual "Global" category row — shown when the course needs a global exam
    if app.shows_virtual_global() {
        let global_grade_text = match course.global_exam_grade {
            Some(g) => format!("{:.1}", g),
            None => "-".to_string(),
        };
        let progress = if course.global_exam_grade.is_some() {
            "1/1".to_string()
        } else {
            "0/1".to_string()
        };

        // Determine colour: pass/fail after global, or override if not yet taken
        let global_color = if let Some(after) = grade_result.grade_after_global {
            if course.is_passing_grade(after) {
                t.status_pass
            } else {
                t.status_fail
            }
        } else {
            // Not taken yet — check if achievable
            let needed = course.needed_global_grade();
            if matches!(needed.status, NeededGradeStatus::Failure) {
                t.status_fail
            } else {
                t.status_override
            }
        };

        // Build needed-grade hint
        let needed_hint = if course.global_exam_grade.is_none() {
            let needed = course.needed_global_grade();
            match needed.status {
                NeededGradeStatus::Warning => needed
                    .value
                    .map(|v| format!(" · {}: {:.0}", m.global_needed, v.ceil()))
                    .unwrap_or_default(),
                NeededGradeStatus::Failure => needed
                    .value
                    .filter(|&v| v > MAX_GRADE)
                    .map(|v| {
                        format!(
                            " · {}: {} ({})",
                            m.global_needed,
                            v.ceil() as i32,
                            m.need_grade_impossible
                        )
                    })
                    .unwrap_or_else(|| format!(" · {}: {}", m.global_needed, m.cannot_pass)),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        let global_policy_hint = match &course.global_policy {
            GlobalExamPolicy::Weighted {
                semester_weight,
                global_weight,
            } => format!(
                " ({:.0}/{:.0})",
                semester_weight * 100.0,
                global_weight * 100.0
            ),
            GlobalExamPolicy::ReplacesWorstGrade => format!(" ({})", m.global_policy_replaces),
            GlobalExamPolicy::None => String::new(),
        };

        // Same accent treatment as regular categories — keeps the visual
        // language consistent for the virtual global row.
        let accent = Span::styled("\u{258E} ", Style::default().fg(global_color));
        let name_spans = vec![
            accent,
            Span::styled(
                m.global_exam,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(global_color),
            ),
            Span::styled(global_policy_hint, Style::default().fg(t.text_muted)),
        ];
        let avg_spans = vec![
            Span::raw(format!("    {}: ", m.grade)),
            Span::styled(global_grade_text, Style::default().fg(global_color)),
            Span::styled(
                format!(" · {}{}", progress, needed_hint),
                Style::default().fg(t.text_muted),
            ),
        ];
        items.push(ListItem::new(vec![
            Line::from(name_spans),
            Line::from(avg_spans),
        ]));
    }

    let visible_count = items.len();
    let title = format!(" {}{} ({}) ", ic.category, m.categories, visible_count);
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
