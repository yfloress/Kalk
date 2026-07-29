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

//! Popup dialogs for evaluation editing, global grade entry, and bulk-add.

use super::helpers::{
    centered_rect, format_course_status, format_needed_grade, render_input_field,
};
use super::icons::icons;
use super::theme::theme;
use crate::app::{App, InputField};
use crate::model::{GlobalExamPolicy, NeededGradeStatus, WeightValidation};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

// =============================================================================
// Evaluation Popup
// =============================================================================

pub fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let is_editing = !is_new;
    let m = app.messages();
    let t = theme();
    let has_weighted = app.category_has_weighted_evals();
    let popup_height = if has_weighted { 60 } else { 50 };
    let area = centered_rect(60, popup_height, frame.area());
    frame.render_widget(Clear, area);
    let title = if is_new {
        format!(" {} ", m.new_evaluation)
    } else {
        format!(" {} ", m.edit_evaluation)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    frame.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(3), // Grade field
        Constraint::Length(1), // Spacing
        Constraint::Length(3), // Name field
        Constraint::Length(1), // Spacing
    ];
    if has_weighted {
        constraints.push(Constraint::Length(3)); // Eval Weight field
        constraints.push(Constraint::Length(1)); // Weight hint
    }
    constraints.push(Constraint::Length(3)); // Required grade info

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(constraints)
        .split(area);

    let mut slot = 0;

    // Grade field FIRST
    render_input_field(
        frame,
        m.grade,
        &app.edit_grade,
        app.input_field == InputField::Grade,
        inner[slot],
    );
    slot += 1;
    slot += 1; // Spacing

    // Name field second
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[slot],
    );
    slot += 1;
    slot += 1; // Spacing

    // Eval Weight field (only for weighted-evaluations categories)
    if has_weighted {
        render_input_field(
            frame,
            m.eval_weight,
            &app.edit_eval_weight,
            app.input_field == InputField::EvalWeight,
            inner[slot],
        );
        slot += 1;

        // Show eval weight total hint
        if let Some(category) = app.current_category() {
            let validation = category.validate_eval_weights();
            let (hint, hint_color) = match validation {
                WeightValidation::Valid => (format!("  {}", m.eval_weights_ok), t.status_pass),
                WeightValidation::Under(total) => (
                    format!("  {} {:.0}%", m.eval_weights_warning, total),
                    t.status_warn,
                ),
                WeightValidation::Over(total) => (
                    format!("  {} {:.0}%", m.eval_weights_error, total),
                    t.status_fail,
                ),
                WeightValidation::Empty => {
                    (format!("  {} 0%", m.eval_weights_warning), t.status_warn)
                }
            };
            let hint_widget = Paragraph::new(hint).style(Style::default().fg(hint_color));
            frame.render_widget(hint_widget, inner[slot]);
        }
        slot += 1;
    }
    if let Some(course) = app.current_course() {
        let (info, info_color) = if let (Some(cat_idx), Some(eval_idx)) =
            (app.selected_category, app.selected_evaluation)
        {
            let needed = course.needed_grade_for_evaluation(cat_idx, eval_idx, is_editing);
            let color = match needed.status {
                NeededGradeStatus::Success => t.status_pass,
                NeededGradeStatus::Failure => t.status_fail,
                NeededGradeStatus::Warning => t.status_warn,
                NeededGradeStatus::Info => t.text_muted,
            };
            (format_needed_grade(&needed, m), color)
        } else {
            let text = format_course_status(course, m, app.use_nerd_fonts);
            let color = match course.current_grade() {
                Some(g) if course.is_passing_grade(g) => t.status_pass,
                Some(_) => t.status_fail,
                None => t.text_muted,
            };
            (text, color)
        };

        let info_block = Block::default()
            .title(format!(" {} ", m.need_to_pass))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(info_color));
        let info_widget = Paragraph::new(info)
            .style(Style::default().fg(info_color))
            .block(info_block);
        frame.render_widget(info_widget, inner[slot]);
    }
}

// =============================================================================
// Global Grade Popup
// =============================================================================

pub fn draw_global_grade_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {}{} ", ic.grade, m.global_exam))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show current course info and policy
    let course = app.current_course();
    let policy_label = course
        .map(|c| match &c.global_policy {
            GlobalExamPolicy::None => m.global_policy_none,
            GlobalExamPolicy::Weighted { .. } => m.global_policy_weighted,
            GlobalExamPolicy::ReplacesWorstGrade => m.global_policy_replaces,
        })
        .unwrap_or(m.global_policy_none);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // policy info
            Constraint::Length(1), // needed grade info
            Constraint::Length(1), // spacing
            Constraint::Length(3), // grade input
            Constraint::Min(0),    // spacer
        ])
        .split(inner);

    // Policy info line
    let policy_info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  {}: ", m.global_policy),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(policy_label, Style::default().fg(t.text_primary)),
    ]));
    frame.render_widget(policy_info, layout[0]);

    // Needed grade info line
    if let Some(course) = course {
        let needed = course.needed_global_grade();
        let needed_text = format_needed_grade(&needed, m);
        let needed_color = match needed.status {
            NeededGradeStatus::Success => t.status_pass,
            NeededGradeStatus::Failure => t.status_fail,
            NeededGradeStatus::Warning => t.status_warn,
            NeededGradeStatus::Info => t.text_muted,
        };
        let needed_line = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("  {}: ", m.global_needed),
                Style::default().fg(t.text_muted),
            ),
            Span::styled(needed_text, Style::default().fg(needed_color)),
        ]));
        frame.render_widget(needed_line, layout[1]);
    }

    // Grade input field
    let field_area = {
        let pad = 2u16.min(layout[3].width / 2);
        Rect::new(
            layout[3].x + pad,
            layout[3].y,
            layout[3].width.saturating_sub(pad * 2),
            layout[3].height,
        )
    };

    render_input_field(
        frame,
        m.global_grade,
        &app.edit_global_grade,
        true,
        field_area,
    );
}

// =============================================================================
// Bulk-Add Evaluations Popup
// =============================================================================

/// Draw the bulk-add evaluations popup (name base + count).
pub fn draw_bulk_add_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    // Fixed size: 3+1+3+1+1 = 9 content + 2 v-margin + 2 borders = 13
    let popup_w = 50u16;
    let popup_h = 13u16;
    let term = frame.area();
    let x = term.x + term.width.saturating_sub(popup_w) / 2;
    let y = term.y + term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w.min(term.width), popup_h.min(term.height));
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {} ", m.bulk_add_title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    frame.render_widget(block, area);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .horizontal_margin(2)
        .constraints([
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Count field
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Hint
        ])
        .split(area);
    let on_name = app.input_field == InputField::Name;
    render_input_field(frame, m.name, &app.edit_name, on_name, inner[0]);
    render_input_field(
        frame,
        m.bulk_add_count,
        &app.edit_bulk_count,
        !on_name,
        inner[2],
    );
    let hint = Paragraph::new(m.bulk_add_hint).style(Style::default().fg(t.text_muted));
    frame.render_widget(hint, inner[4]);
}
