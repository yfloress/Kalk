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

//! Shared rendering helpers and formatting functions for the UI layer.
//!
//! This module contains pure-ish utility functions used by both `mod.rs` (panels)
//! and `popups.rs` (overlay dialogs). Extracting them keeps both of those files
//! under the ~600-line guideline.

use crate::app::InputField;
use crate::i18n::Messages;
use crate::model::{Course, CourseGradeResult, MAX_GRADE, NeededGradeStatus, WeightValidation};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use super::icons::icons;
use super::theme::theme;

// =============================================================================
// Delete Confirmation Helper
// =============================================================================

/// Shared renderer for delete-confirmation popups.
///
/// Draws a fixed-size centred popup with a question, a separator, a warning
/// line, and keybinding hints at the bottom.
pub fn render_delete_confirmation(
    frame: &mut Frame,
    title: &str,
    question: &str,
    warning_text: &str,
    confirm_label: &str,
    cancel_label: &str,
    use_nerd_fonts: bool,
) {
    let t = theme();
    let ic = icons(use_nerd_fonts);
    let term = frame.size();

    // Build the formatted title with icon
    let title_str = format!(" {}{} ", ic.delete, title);

    // Calculate the minimum width needed to avoid truncation:
    // - title (with border chars)
    // - question line: 2 (indent) + icon + question
    // - warning line: 2 (indent) + icon + warning_text
    // - hints line: 2 (indent) + "Enter/y: " + confirm + "  " + "Esc/n: " + cancel
    let question_icon_w = if use_nerd_fonts { 2 } else { 0 };
    let warning_icon_w = ic.warning.chars().count();
    let content_widths = [
        title_str.chars().count() + 2, // +2 for border chars
        2 + question_icon_w + question.chars().count(),
        2 + warning_icon_w + warning_text.chars().count(),
        2 + 9 + confirm_label.chars().count() + 2 + 6 + cancel_label.chars().count(),
    ];
    let max_content = content_widths.iter().copied().max().unwrap_or(40);
    // Add 2 for left+right border, clamp to reasonable bounds
    let ideal_w = (max_content + 4) as u16;
    let popup_w = ideal_w.clamp(36, 60).min(term.width);

    // Height: title(1) + question(2) + sep(1) + warning(2) + spacer(1) + hints(1) + border(2)
    let popup_h = 11u16.min(term.height);
    let x = term.x + term.width.saturating_sub(popup_w) / 2;
    let y = term.y + term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w, popup_h);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(title_str)
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border_danger));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // question
            Constraint::Length(1), // separator
            Constraint::Length(2), // warning
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // keybindings
        ])
        .split(inner);

    // Question line with optional icon
    let mut q_spans = vec![Span::styled("  ", Style::default())];
    if use_nerd_fonts {
        q_spans.push(Span::styled(ic.info, Style::default().fg(t.text_primary)));
    }
    q_spans.push(Span::styled(
        question,
        Style::default()
            .fg(t.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
    let q = Paragraph::new(Line::from(q_spans)).wrap(Wrap { trim: true });
    frame.render_widget(q, chunks[0]);

    // Dynamic separator that fills available width
    let sep_width = inner.width as usize;
    let sep_line: String = "\u{2500}".repeat(sep_width);
    let sep = Paragraph::new(Line::from(Span::styled(
        sep_line,
        Style::default().fg(t.popup_separator),
    )));
    frame.render_widget(sep, chunks[1]);

    // Warning line with icon and wrapping
    let warn = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  {}", ic.warning),
            Style::default().fg(t.status_warn),
        ),
        Span::styled(warning_text, Style::default().fg(t.status_warn)),
    ]))
    .wrap(Wrap { trim: true });
    frame.render_widget(warn, chunks[2]);

    // Keybinding hints
    let hints = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  Enter/y: {}  ", confirm_label),
            Style::default().fg(t.status_fail),
        ),
        Span::styled(
            format!("Esc/n: {}", cancel_label),
            Style::default().fg(t.text_muted),
        ),
    ]));
    frame.render_widget(hints, chunks[4]);
}

// =============================================================================
// Rendering Helpers
// =============================================================================

/// Return a border style that highlights when the panel is focused.
pub fn focused_border_style(is_focused: bool) -> Style {
    let t = theme();
    if is_focused {
        Style::default().fg(t.border_focused)
    } else {
        Style::default().fg(t.border_unfocused)
    }
}

/// Create a centred rectangle inside `r` using percentage-based sizing.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render a single-line text input field with a label border.
/// When `is_active` the field is highlighted and a cursor pipe is appended.
pub fn render_input_field(
    frame: &mut Frame,
    label: &str,
    value: &str,
    is_active: bool,
    area: Rect,
) {
    let t = theme();
    let style = if is_active {
        Style::default().fg(t.input_active)
    } else {
        Style::default().fg(t.input_inactive)
    };

    // Show cursor indicator when active
    let display_value = if is_active {
        format!("{}|", value)
    } else {
        value.to_string()
    };

    let input = Paragraph::new(display_value).style(style).block(
        Block::default()
            .title(label)
            .borders(Borders::ALL)
            .border_type(t.border_type)
            .border_style(style),
    );

    frame.render_widget(input, area);
}

/// Render a toggle field that cycles values with Space/Enter instead of text
/// input.  Shows the current value in angle brackets when active (`< val >`)
/// or square brackets when inactive (`[val]`).
pub fn render_toggle_field(
    frame: &mut Frame,
    label: &str,
    current_value: &str,
    is_active: bool,
    area: Rect,
) {
    let t = theme();
    let style = if is_active {
        Style::default().fg(t.input_active)
    } else {
        Style::default().fg(t.input_inactive)
    };

    let display = if is_active {
        format!("< {} >", current_value)
    } else {
        format!("[{}]", current_value)
    };

    let widget = Paragraph::new(display).style(style).block(
        Block::default()
            .title(label)
            .borders(Borders::ALL)
            .border_type(t.border_type)
            .border_style(style),
    );

    frame.render_widget(widget, area);
}

// =============================================================================
// Category Help
// =============================================================================

/// Return a short contextual help string for the currently focused field.
pub fn contextual_field_help(field: InputField, m: &Messages) -> String {
    match field {
        InputField::Name => m.help_name.to_string(),
        InputField::Weight => m.help_weight.to_string(),
        InputField::DropLowest => m.help_drop_lowest.to_string(),
        InputField::AvgMethod => m.help_averaging_method.to_string(),
        InputField::MinimumAverage => m.help_minimum_average.to_string(),
        InputField::OnMinNotMet => m.help_on_min_not_met.to_string(),
        InputField::MinPerEval => m.help_min_per_eval.to_string(),
        InputField::RoundBeforeWeight => m.help_round_before_weighting.to_string(),
        _ => String::new(),
    }
}

/// Draw a full-screen help overlay explaining every category field.
pub fn draw_category_help_overlay(frame: &mut Frame, m: &Messages) {
    let t = theme();
    let area = centered_rect(70, 60, frame.size());
    frame.render_widget(Clear, area);

    let label_style = Style::default()
        .fg(t.input_active)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(t.text_primary);

    let help_lines = vec![
        Line::from(Span::styled(
            format!("  {} ", m.advanced_rules),
            Style::default()
                .fg(t.status_info)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.name), label_style),
            Span::styled(m.help_name, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.weight), label_style),
            Span::styled(m.help_weight, desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("── {} ──", m.advanced_rules),
            Style::default().fg(t.status_info),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.drop_lowest), label_style),
            Span::styled(m.help_drop_lowest, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.averaging_method), label_style),
            Span::styled(m.help_averaging_method, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.minimum_average), label_style),
            Span::styled(m.help_minimum_average, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.on_minimum_not_met), label_style),
            Span::styled(m.help_on_min_not_met, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.minimum_per_evaluation), label_style),
            Span::styled(m.help_min_per_eval, desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", m.round_before_weighting), label_style),
            Span::styled(m.help_round_before_weighting, desc_style),
        ]),
    ];

    let block = Block::default()
        .title(format!(" {} ", m.help_toggle))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.status_info));

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

// =============================================================================
// Formatting Functions
// =============================================================================

/// Compute the display text and colour for the course average widget.
///
/// Accounts for rule overrides (`overridden_by`), `needs_global`, and normal
/// pass/fail colouring.  Returns `(text, Color)`.
pub fn format_course_average(
    course: &Course,
    grade_result: &CourseGradeResult,
    m: &Messages,
) -> (String, Color) {
    let t = theme();
    if let Some(ref cat_name) = grade_result.overridden_by {
        // FailCourse overrides produce grade 0 which is never passing —
        // show as FAILED in red instead of "capped by" in magenta.
        if !course.is_passing_grade(grade_result.grade) {
            let text = format!(
                "{}: {:.1} -> {:.0} ({}) [{}]",
                m.current,
                grade_result.grade,
                Course::round_grade(grade_result.grade),
                m.failed,
                cat_name
            );
            (text, t.status_fail)
        } else {
            let text = format!(
                "{}: {:.1} ({} {})",
                m.current, grade_result.grade, m.grade_capped_by, cat_name
            );
            (text, t.status_override)
        }
    } else if grade_result.needs_global {
        let text = format!(
            "{}: {:.1} | {}",
            m.current, grade_result.grade, m.needs_global
        );
        (text, t.status_warn)
    } else {
        match course.current_grade() {
            Some(grade) => {
                let rounded = Course::round_grade(grade);
                let status = if course.is_passing_grade(grade) {
                    m.passed
                } else {
                    m.failed
                };
                let text = format!("{}: {:.1} -> {:.0} ({})", m.current, grade, rounded, status);
                let color = if course.is_passing_grade(grade) {
                    t.status_pass
                } else {
                    t.status_fail
                };
                (text, color)
            }
            None => (m.no_grades_yet.to_string(), t.text_muted),
        }
    }
}

/// Format course status message (used in the courses panel list items).
pub fn format_course_status(course: &Course, m: &Messages) -> String {
    if !course.has_evaluations() {
        return m.no_evaluations.to_string();
    }

    match course.current_grade() {
        Some(grade) => {
            let rounded = Course::round_grade(grade);
            let status = if course.is_passing_grade(grade) {
                m.passed
            } else {
                m.failed
            };
            format!("{}: {:.1} -> {:.0} ({})", m.current, grade, rounded, status)
        }
        None => m.no_evaluations.to_string(),
    }
}

/// Format weight-validation status into a short human-readable string.
pub fn format_weight_validation(validation: &WeightValidation, m: &Messages) -> String {
    match validation {
        WeightValidation::Valid => m.weights_ok.to_string(),
        WeightValidation::Under(total) => format!("{} {:.1}%", m.weights_warning, total),
        WeightValidation::Over(total) => format!("{} {:.1}%", m.weights_error, total),
        WeightValidation::Empty => m.no_categories.to_string(),
    }
}

/// Format the result of a needed-grade calculation into a user-friendly string.
pub fn format_needed_grade(needed: &crate::model::NeededGrade, m: &Messages) -> String {
    match needed.status {
        NeededGradeStatus::Success => {
            if let Some(value) = needed.value {
                if value == 0.0 {
                    format!("{}: 0+ ({})", m.need, m.need_grade_any)
                } else {
                    // Already graded and passing
                    format!("{:.0} ({})", value, m.passing)
                }
            } else {
                format!("{}: 0+ ({})", m.need, m.need_grade_any)
            }
        }
        NeededGradeStatus::Failure => {
            if let Some(value) = needed.value {
                if value > MAX_GRADE {
                    let rounded_up = value.ceil() as i32;
                    format!(
                        "{}: {:.2} -> {} ({})",
                        m.need, value, rounded_up, m.need_grade_impossible
                    )
                } else {
                    // Already graded and failing
                    format!("{:.0} ({})", value, m.below_passing)
                }
            } else {
                m.cannot_pass.to_string()
            }
        }
        NeededGradeStatus::Warning => {
            if let Some(value) = needed.value {
                let rounded_up = value.ceil() as i32;
                if (value - value.floor()).abs() < 0.01 {
                    format!("{} {} {}", m.need, rounded_up, m.need_grade_in_eval)
                } else {
                    format!(
                        "{} {:.2} -> {} {}",
                        m.need, value, rounded_up, m.need_grade_in_eval
                    )
                }
            } else {
                m.cannot_pass.to_string()
            }
        }
        NeededGradeStatus::Info => m.no_evaluations.to_string(),
    }
}
