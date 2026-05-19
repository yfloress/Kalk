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

mod eval_popups;
pub(crate) mod helpers;
pub(crate) mod icons;
mod panels;
mod popups;
mod settings_popup;
pub(crate) mod theme;

use crate::app::{App, Focus, Screen};
use crate::model::{Course, GlobalExamPolicy, MAX_GRADE, NeededGradeStatus, WeightValidation};
use eval_popups::{draw_bulk_add_popup, draw_evaluation_popup, draw_global_grade_popup};
use helpers::{focused_border_style, format_course_average, format_weight_validation};
use icons::icons;
use panels::{draw_evaluations_panel, draw_footer};
use popups::{
    draw_category_popup, draw_course_popup, draw_delete_popup, draw_delete_template_popup,
    draw_help_popup, draw_language_popup, draw_save_template_popup, draw_template_popup,
};
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.size());

    // Main area: 3-column layout for Course -> Category -> Evaluation hierarchy
    // Compact mode triggers either by user toggle OR when the terminal is too
    // narrow for the normal two-line courses layout to display without clipping.
    let total_width = chunks[0].width;
    let ic = icons(app.use_nerd_fonts);
    let m = app.messages();

    let highlight_len = ic.highlight.chars().count() as u16;
    let max_name_len = app
        .courses
        .iter()
        .map(|c| c.name.chars().count() as u16)
        .max()
        .unwrap_or(4);

    // Calculate the exact width normal mode needs per course, then take the max.
    // Line 1: highlight + "NAME [OK]"  (name + weight_status)
    // Line 2: highlight + "  Actual: 100 (REPROBADO) [CatName]"
    let normal_needed = {
        let weight_status_max: u16 = app
            .courses
            .iter()
            .map(|c| match c.validate_weights() {
                WeightValidation::Valid => {
                    // " [✓]"
                    2 + ic.weight_ok.chars().count() as u16 + 1
                }
                WeightValidation::Under(w) | WeightValidation::Over(w) => {
                    // " [!80%]"
                    let digits = format!("{:.0}", w).len() as u16;
                    2 + 1 + digits + 1 + 1 // " [" + icon + digits + "%" + "]"
                }
                WeightValidation::Empty => {
                    // " [Sin categorias]"
                    2 + m.no_categories.chars().count() as u16 + 1
                }
            })
            .max()
            .unwrap_or(4);

        // Longest second line: "  Actual: 100 (REPROBADO) · 12/15"
        // or "  Actual: 0 (REPROBADO) [CatName] · 0/5"
        // The trailing " · X/Y" chip is appended when the course has any
        // evaluations, so factor its width into the layout estimate.
        let status_line_max: u16 = app
            .courses
            .iter()
            .map(|c| {
                let grade_result = c.compute_grade();
                let has_rule_issues = grade_result.overridden_by.is_some()
                    || grade_result.needs_global
                    || !grade_result.failed_minimums.is_empty();
                let graded: usize = c.categories.iter().map(|cat| cat.graded_count()).sum();
                let total: usize = c.categories.iter().map(|cat| cat.evaluations.len()).sum();
                let digits = |n: usize| -> u16 {
                    if n == 0 {
                        1
                    } else {
                        (n as f64).log10().floor() as u16 + 1
                    }
                };
                // " · {graded}/{total}" = 3 + digits(graded) + 1 + digits(total)
                let chip_w = if total > 0 {
                    3 + digits(graded) + 1 + digits(total)
                } else {
                    0
                };
                if !c.has_evaluations() {
                    // "  Sin evaluaciones"
                    2 + m.no_evaluations.chars().count() as u16
                } else {
                    let rounded = Course::round_grade(grade_result.grade);
                    let is_truly_passing =
                        c.is_passing_grade(grade_result.grade) && !has_rule_issues;
                    let label = if is_truly_passing { m.passed } else { m.failed };
                    let base = if let Some(ref cat_name) = grade_result.overridden_by {
                        // "  Actual: 0 (REPROBADO) [CatName]"
                        format!("{}: {:.0} ({}) [{}]", m.current, rounded, label, cat_name)
                    } else {
                        format!("{}: {:.0} ({})", m.current, rounded, label)
                    };
                    2 + base.chars().count() as u16 + chip_w // "  " prefix + chip
                }
            })
            .max()
            .unwrap_or(10);

        // Accent column (the coloured "\u{258E} " bar shown before every
        // course) costs 2 extra columns on both lines.
        let accent_w = 2u16;
        let line1_w = highlight_len + accent_w + max_name_len + weight_status_max;
        let line2_w = highlight_len + accent_w + status_line_max;
        line1_w.max(line2_w) + 2 // +2 for borders
    };

    // Auto-compact when the normal layout would need more width than
    // available, or when the terminal is very narrow overall.
    let auto_compact = normal_needed > (total_width * 2 / 5) || total_width < 100;
    let effective_compact = app.compact_courses || auto_compact;

    // Minimum width = title text so it never gets clipped
    // Title: " {icon}Courses (N) " + 2 border columns
    let title_width = {
        let count_digits = if app.courses.is_empty() {
            1
        } else {
            (app.courses.len() as f64).log10().floor() as u16 + 1
        };
        // " " + icon + label + " (" + digits + ") " + borders
        1 + ic.course.chars().count() as u16
            + m.courses.chars().count() as u16
            + 2
            + count_digits
            + 2
            + 2
    };

    let courses_constraint = if effective_compact {
        // Dynamic width: accent(2) + name + space + grade(3) + chip(6 max
        // for " 99/99") + borders(2) + highlight + pad(1)
        let needed = 2 + max_name_len + 1 + 3 + 6 + 2 + highlight_len + 1;
        Constraint::Length(needed.max(title_width).clamp(12, 38))
    } else {
        // Dynamic width based on actual content, clamped to reasonable bounds
        Constraint::Length(normal_needed.clamp(15, total_width / 2))
    };

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if effective_compact {
            [
                courses_constraint,
                Constraint::Percentage(45),
                Constraint::Min(0),
            ]
        } else {
            [
                courses_constraint,
                Constraint::Min(0),
                Constraint::Percentage(40),
            ]
        })
        .split(chunks[0]);

    draw_courses_panel(frame, app, main_chunks[0], effective_compact);
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
        Screen::Main => {}
    }
}

// =============================================================================
// Panel Drawing
// =============================================================================

fn draw_courses_panel(frame: &mut Frame, app: &App, area: Rect, effective_compact: bool) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let is_focused = app.focus == Focus::Courses;
    let border_style = focused_border_style(is_focused);

    let compact = effective_compact;

    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| {
            // Compute the true course result (accounting for rule overrides)
            let grade_result = c.compute_grade();
            let has_evals = c.has_evaluations();

            // True pass/fail color: accounts for rule overrides and needs_global.
            // When needs_global is set, check whether it's actually possible
            // to pass via the global exam.  If impossible, treat as failed.
            let global_impossible = grade_result.needs_global
                && c.global_policy != GlobalExamPolicy::None
                && matches!(c.needed_global_grade().status, NeededGradeStatus::Failure);

            // Check if the global was already taken and resolved the outcome
            let global_taken_passing = grade_result.needs_global
                && grade_result
                    .grade_after_global
                    .is_some_and(|g| c.is_passing_grade(g));
            let global_taken_failing = grade_result.needs_global
                && grade_result
                    .grade_after_global
                    .is_some_and(|g| !c.is_passing_grade(g));

            // True pass/fail color: accounts for rule overrides and needs_global.
            // A course is only truly passing when:
            // - grade >= passing_grade
            // - no rule overrides (overridden_by is None)
            // - no unresolved needs_global flag
            // - no failed_minimums at all
            let has_rule_issues = grade_result.overridden_by.is_some()
                || (grade_result.needs_global && !global_taken_passing)
                || !grade_result.failed_minimums.is_empty();

            let true_color = if !has_evals {
                t.text_muted
            } else if global_taken_passing {
                t.status_pass
            } else if global_taken_failing {
                t.status_fail
            } else if grade_result.needs_global && !global_impossible {
                t.status_override
            } else if has_rule_issues || !c.is_passing_grade(grade_result.grade) {
                t.status_fail
            } else {
                t.status_pass
            };

            // Coloured accent bar shown before each course so the user can
            // scan the list and see pass/fail/no-data status at a glance.
            let accent = Span::styled(
                "\u{258E} ",
                Style::default().fg(true_color),
            );

            // Progress chip — "graded/total" across every category of the
            // course.  Lets the user see how far through the semester each
            // course is without having to open it.
            let graded_count: usize =
                c.categories.iter().map(|cat| cat.graded_count()).sum();
            let total_count: usize =
                c.categories.iter().map(|cat| cat.evaluations.len()).sum();

            if compact {
                // Compact: single line — "NAME GRADE X/Y" with colour-coded
                // grade and a muted progress chip.
                let short_grade = if has_evals {
                    // When global was taken, show the after-global grade
                    let display_grade = if let Some(after) = grade_result.grade_after_global {
                        after
                    } else {
                        grade_result.grade
                    };
                    let rounded = Course::round_grade(display_grade);
                    format!(" {:.0}", rounded)
                } else {
                    " -".to_string()
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
            } else {
                // Weight validation indicator (only in normal mode)
                // When weights are valid but the course has academic issues
                // (needs_global, rule overrides, failed minimums), show the
                // academic status icon instead so the user is not misled.
                let recoverable_global = grade_result.needs_global
                    && !global_impossible
                    && !global_taken_passing
                    && !global_taken_failing;
                let simply_failing = has_evals
                    && !c.is_passing_grade(grade_result.grade)
                    && !grade_result.needs_global;
                let no_evals_yet = !has_evals;
                let effectively_failed = global_taken_failing
                    || global_impossible
                    || simply_failing
                    || (has_rule_issues && !recoverable_global && !global_taken_passing);

                let weight_status = if global_taken_passing
                    && matches!(c.validate_weights(), WeightValidation::Valid)
                {
                    Span::styled(
                        format!(" [{}]", ic.weight_ok),
                        Style::default().fg(t.status_pass),
                    )
                } else if recoverable_global
                    && matches!(c.validate_weights(), WeightValidation::Valid)
                {
                    Span::styled(
                        format!(" [{}]", ic.warning),
                        Style::default().fg(t.status_override),
                    )
                } else if effectively_failed
                    && matches!(c.validate_weights(), WeightValidation::Valid)
                {
                    Span::styled(
                        format!(" [{}]", ic.failed.trim()),
                        Style::default().fg(t.status_fail),
                    )
                } else if no_evals_yet && matches!(c.validate_weights(), WeightValidation::Valid) {
                    Span::styled(
                        format!(" [{}]", ic.weight_ok),
                        Style::default().fg(t.text_muted),
                    )
                } else {
                    match c.validate_weights() {
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
                    }
                };

                // Normal: two lines with full status info (rule-aware)
                let status = if has_evals {
                    if let Some(after) = grade_result.grade_after_global {
                        // Global was taken — show the final (after-global) grade
                        let rounded = Course::round_grade(after);
                        let label = if c.is_passing_grade(after) {
                            m.passed
                        } else {
                            m.failed
                        };
                        format!("{}: {:.0} ({})", m.current, rounded, label)
                    } else {
                        let rounded = Course::round_grade(grade_result.grade);
                        let is_truly_passing =
                            c.is_passing_grade(grade_result.grade) && !has_rule_issues;
                        let label = if is_truly_passing { m.passed } else { m.failed };
                        if let Some(ref cat_name) = grade_result.overridden_by {
                            format!("{}: {:.0} ({}) [{}]", m.current, rounded, label, cat_name)
                        } else {
                            format!("{}: {:.0} ({})", m.current, rounded, label)
                        }
                    }
                } else {
                    m.no_evaluations.to_string()
                };
                let mut status_spans = vec![Span::styled(
                    format!("    {}", status),
                    Style::default().fg(true_color),
                )];
                if total_count > 0 {
                    status_spans.push(Span::styled(
                        format!(" \u{00b7} {}/{}", graded_count, total_count),
                        Style::default().fg(t.text_muted),
                    ));
                }
                ListItem::new(vec![
                    Line::from(vec![
                        accent,
                        Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                        weight_status,
                    ]),
                    Line::from(status_spans),
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
            let accent = Span::styled(
                "\u{258E} ",
                Style::default().fg(avg_color),
            );

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
                    format!(" | {}", progress),
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
                    .map(|v| format!(" | {}: {:.0}", m.global_needed, v.ceil()))
                    .unwrap_or_default(),
                NeededGradeStatus::Failure => needed
                    .value
                    .filter(|&v| v > MAX_GRADE)
                    .map(|v| {
                        format!(
                            " | {}: {} ({})",
                            m.global_needed,
                            v.ceil() as i32,
                            m.need_grade_impossible
                        )
                    })
                    .unwrap_or_else(|| format!(" | {}: {}", m.global_needed, m.cannot_pass)),
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
        let accent = Span::styled(
            "\u{258E} ",
            Style::default().fg(global_color),
        );
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
                format!(" | {}{}", progress, needed_hint),
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
