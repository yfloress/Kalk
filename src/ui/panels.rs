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
use super::icons::icons;
use super::theme::theme;
use crate::app::{App, Focus, Screen};
use crate::model::{GlobalExamPolicy, MAX_GRADE, NeededGradeStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

// =============================================================================
// Evaluations Panel
// =============================================================================

pub fn draw_evaluations_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let is_focused = app.focus == Focus::Evaluations;
    let border_style = focused_border_style(is_focused);
    let selected_cat_idx = app.selected_category;

    // Virtual global category — render a dedicated panel instead of the
    // normal evaluations table.
    if app.is_on_virtual_global() {
        draw_virtual_global_panel(frame, app, area);
        return;
    }

    let Some(category) = app.current_category() else {
        let block = Block::default()
            .title(format!(" {}{} ", ic.evaluation, m.evaluations))
            .borders(Borders::ALL)
            .border_type(t.border_type)
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
        t.status_override
    } else {
        match category.is_passing(passing_grade) {
            Some(true) => t.status_pass,
            Some(false) => t.status_fail,
            None => t.text_muted,
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
        Span::styled(format!("{}: ", m.avg), Style::default().fg(t.text_muted)),
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
            Style::default().fg(t.text_muted),
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
            .border_type(t.border_type)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Precompute dropped indices and below-minimum indices
    let dropped_indices = category.dropped_indices();
    let below_min_indices = category.evals_below_minimum().unwrap_or_default();

    // Check if any evaluation has a status indicator (dropped or below min)
    let has_status = !dropped_indices.is_empty() || !below_min_indices.is_empty();

    // Auto-compact status text when the panel is narrow
    let panel_inner_w = area.width.saturating_sub(2) as usize;
    let narrow_evals = panel_inner_w < 40;

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
                    .bg(t.highlight_bg)
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
                Style::default().fg(t.text_muted)
            } else if is_below_min {
                Style::default().fg(t.status_override)
            } else {
                match e.grade {
                    Some(g) if g >= passing_grade => Style::default().fg(t.status_pass),
                    Some(_) => Style::default().fg(t.status_fail),
                    None => Style::default().fg(t.text_muted),
                }
            };

            // Status indicator — icon-only when panel is narrow
            let status = if is_dropped {
                if narrow_evals {
                    ic.dropped.to_string()
                } else {
                    format!("({}{})", ic.dropped, m.dropped)
                }
            } else if is_below_min {
                if narrow_evals {
                    ic.below_min.to_string()
                } else {
                    format!("({}{})", ic.below_min, m.eval_below_min)
                }
            } else {
                String::new()
            };
            let status_style = if is_dropped {
                Style::default().fg(t.text_muted)
            } else {
                Style::default().fg(t.status_override)
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

    let title = format!(
        " {}{} ({}) ",
        ic.evaluation,
        m.evaluations,
        category.evaluations.len()
    );
    let col_widths: Vec<Constraint> = if has_status {
        let status_w = if narrow_evals { 3 } else { 10 };
        vec![
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
            Constraint::Min(status_w),
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
            .border_type(t.border_type)
            .border_style(border_style),
    );

    frame.render_widget(table, chunks[1]);
}

// =============================================================================
// Virtual Global Category Panel
// =============================================================================

/// Render the evaluations panel when the virtual "Global" category is selected.
/// Shows the global exam grade (or a prompt to enter it), the needed grade,
/// and the result after the global exam if a grade has been entered.
fn draw_virtual_global_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let is_focused = app.focus == Focus::Evaluations;
    let border_style = focused_border_style(is_focused);

    let Some(course) = app.current_course() else {
        return;
    };

    let grade_result = course.compute_grade();

    // Build content lines
    let mut lines: Vec<Line> = Vec::new();

    // 1. Global exam grade (or hint to enter it)
    match course.global_exam_grade {
        Some(g) => {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", m.global_grade),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:.1}", g), Style::default().fg(t.status_info)),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", m.global_grade),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled("-", Style::default().fg(t.text_muted)),
            ]));
        }
    }

    lines.push(Line::from(""));

    // 2. Needed grade in global
    let needed = course.needed_global_grade();
    match needed.status {
        NeededGradeStatus::Warning => {
            if let Some(v) = needed.value {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", m.global_needed),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.0}", v.ceil()),
                        Style::default().fg(t.status_override),
                    ),
                ]));
            }
        }
        NeededGradeStatus::Failure => {
            let hint = if let Some(v) = needed.value {
                if v > MAX_GRADE {
                    format!("{} ({})", v.ceil() as i32, m.need_grade_impossible)
                } else {
                    m.cannot_pass.to_string()
                }
            } else {
                m.cannot_pass.to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", m.global_needed),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(hint, Style::default().fg(t.status_fail)),
            ]));
        }
        NeededGradeStatus::Success => {
            if course.global_exam_grade.is_some() {
                // Already took global and passed
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", m.global_result),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(m.passed, Style::default().fg(t.status_pass)),
                ]));
            }
        }
        NeededGradeStatus::Info => {}
    }

    // 3. Result after global (when grade has been entered)
    if let Some(after) = grade_result.grade_after_global {
        let rounded = crate::model::Course::round_grade(after);
        let (status_text, status_color) = if course.is_passing_grade(after) {
            (m.passed, t.status_pass)
        } else {
            (m.failed, t.status_fail)
        };
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", m.global_result),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.1} -> {:.0} ({})", after, rounded, status_text),
                Style::default().fg(status_color),
            ),
        ]));
    }

    // 4. Policy info
    lines.push(Line::from(""));
    let policy_desc = match &course.global_policy {
        GlobalExamPolicy::Weighted {
            semester_weight,
            global_weight,
        } => format!(
            "NF = NP * {:.0}% + CG * {:.0}%",
            semester_weight * 100.0,
            global_weight * 100.0
        ),
        GlobalExamPolicy::ReplacesWorstGrade => m.global_policy_replaces.to_string(),
        GlobalExamPolicy::None => String::new(),
    };
    if !policy_desc.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            policy_desc,
            Style::default().fg(t.text_muted),
        )]));
    }

    // 5. Enter hint
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        m.global_enter_hint,
        Style::default().fg(t.text_muted),
    )]));

    let title = format!(" {}{} ", ic.evaluation, m.global_exam);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(border_style);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

// =============================================================================
// Footer Helpers
// =============================================================================

/// Build a styled `Line` from a slice of `(key, description)` pairs.
///
/// Adapts to available width:
/// - **Full mode**: `key: description | key: description | ...`
/// - **Compact mode** (when full doesn't fit): `key | key | key | ...`
///
/// The `available_width` is the inner width (excluding borders) of the footer.
fn styled_keybindings<'a>(
    pairs: &[(&'a str, &'a str)],
    t: &super::theme::Theme,
    ic: &super::icons::IconSet,
    available_width: u16,
) -> Line<'a> {
    let sep_len = ic.key_hint_sep.chars().count();

    // Calculate full-mode width: key + ": " + desc, separated by key_hint_sep
    let full_width: usize = pairs
        .iter()
        .enumerate()
        .map(|(i, (key, desc))| {
            let entry = key.chars().count() + 2 + desc.chars().count(); // "key: desc"
            if i > 0 { entry + sep_len } else { entry }
        })
        .sum();

    let use_compact = full_width > available_width as usize;

    let mut spans: Vec<Span<'a>> = Vec::with_capacity(pairs.len() * 4);
    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                ic.key_hint_sep,
                Style::default().fg(t.footer_border),
            ));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ));
        if !use_compact {
            spans.push(Span::styled(
                format!(": {}", desc),
                Style::default().fg(t.footer_desc),
            ));
        }
    }
    Line::from(spans)
}

// =============================================================================
// Footer
// =============================================================================

pub fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    // Inner width = total width minus 2 border columns
    let available_width = area.width.saturating_sub(2);

    // If there's a status message (error/info), show it prominently
    if let Some(ref status) = app.status_message {
        let is_error = status.starts_with("Error");
        let color = if is_error {
            t.status_fail
        } else {
            t.status_warn
        };

        let footer = Paragraph::new(status.as_str())
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(t.border_type)
                    .border_style(Style::default().fg(color)),
            );
        frame.render_widget(footer, area);
        return;
    }

    let line: Line = match &app.screen {
        Screen::Main => match app.focus {
            Focus::Courses => styled_keybindings(
                &[
                    ("q", m.quit),
                    ("n", m.new),
                    ("Enter", m.edit),
                    ("d", m.delete),
                    ("g", m.global_exam),
                    ("t", m.save_as_template),
                    ("b", m.balance),
                    ("c", m.compact),
                    ("S", m.settings),
                    ("L", m.change_language),
                ],
                t,
                ic,
                available_width,
            ),
            Focus::Categories => styled_keybindings(
                &[
                    ("q", m.quit),
                    ("n", m.new_category),
                    ("Enter", m.edit),
                    ("d", m.delete),
                    ("S", m.settings),
                    ("L", m.change_language),
                ],
                t,
                ic,
                available_width,
            ),
            Focus::Evaluations => styled_keybindings(
                &[
                    ("q", m.quit),
                    ("n", m.new_eval),
                    ("Enter", m.edit),
                    ("d", m.delete),
                    ("S", m.settings),
                    ("L", m.change_language),
                ],
                t,
                ic,
                available_width,
            ),
        },
        Screen::SelectingTemplate => {
            if app.is_user_template_selected() {
                styled_keybindings(
                    &[
                        ("\u{2191}/\u{2193}/Tab", m.select),
                        ("Enter", m.confirm),
                        ("d", m.delete),
                        ("Esc", m.cancel),
                    ],
                    t,
                    ic,
                    available_width,
                )
            } else {
                styled_keybindings(
                    &[
                        ("\u{2191}/\u{2193}/Tab", m.select),
                        ("Enter", m.confirm),
                        ("Esc", m.cancel),
                    ],
                    t,
                    ic,
                    available_width,
                )
            }
        }
        Screen::SelectingLanguage => styled_keybindings(
            &[
                ("\u{2191}/\u{2193}/Tab", m.select),
                ("Enter", m.confirm),
                ("Esc", m.cancel),
            ],
            t,
            ic,
            available_width,
        ),
        Screen::EditingCategory { .. } => {
            let rules_hint = if app.show_advanced_rules {
                m.advanced_rules_hide
            } else {
                m.advanced_rules_show
            };
            styled_keybindings(
                &[
                    ("Tab", m.next_field),
                    ("Enter", m.confirm),
                    ("Esc", m.cancel),
                    ("Shift+A", rules_hint),
                    ("?", m.help_toggle),
                ],
                t,
                ic,
                available_width,
            )
        }
        Screen::EditingCourse { .. } => styled_keybindings(
            &[
                ("Tab", m.next_field),
                ("Enter", m.confirm),
                ("Esc", m.cancel),
                ("?", m.help_toggle),
            ],
            t,
            ic,
            available_width,
        ),
        Screen::EditingEvaluation { .. } | Screen::SavingTemplate => styled_keybindings(
            &[
                ("Tab", m.next_field),
                ("Enter", m.confirm),
                ("Esc", m.cancel),
            ],
            t,
            ic,
            available_width,
        ),
        Screen::ConfirmDelete | Screen::ConfirmDeleteTemplate => styled_keybindings(
            &[("Enter/y", m.confirm), ("Esc/n", m.cancel)],
            t,
            ic,
            available_width,
        ),
        Screen::Settings => styled_keybindings(
            &[("Space", m.toggle), ("Enter", m.confirm), ("Esc", m.cancel)],
            t,
            ic,
            available_width,
        ),
        Screen::EnteringGlobalGrade => styled_keybindings(
            &[("Enter", m.confirm), ("Esc", m.cancel)],
            t,
            ic,
            available_width,
        ),
    };

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(t.border_type)
            .border_style(Style::default().fg(t.footer_border)),
    );

    frame.render_widget(footer, area);
}
