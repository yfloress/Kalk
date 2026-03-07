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

//! Popup dialogs for the UI layer.
//!
//! This module contains all overlay popups (template selection, course/category/
//! evaluation editing, delete confirmations, language selection, save-as-template).
//! Rendering helpers and formatting functions live in `helpers.rs`.

use crate::app::{App, Focus, InputField};
use crate::i18n::Language;
use crate::model::{AveragingMethod, MinimumNotMetAction, NeededGradeStatus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::helpers::{
    centered_rect, contextual_field_help, draw_category_help_overlay, format_course_status,
    format_needed_grade, render_delete_confirmation, render_input_field, render_toggle_field,
};

// =============================================================================
// Popup Drawing
// =============================================================================

pub fn draw_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(60, 70, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.select_course_template))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Combine built-in and user templates
    let all_templates = app.all_templates();
    let built_in_count = app.built_in_templates.len();

    let items: Vec<ListItem> = all_templates
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_user_template = i >= built_in_count;
            let prefix = if is_user_template { "★ " } else { "" };

            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("{}{}", prefix, t.name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {}", t.description),
                    Style::default().fg(if is_user_template {
                        Color::Yellow
                    } else {
                        Color::Gray
                    }),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_template));

    frame.render_stateful_widget(list, inner, &mut state);
}

pub fn draw_course_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    // Fixed height: border(1) + top_pad + field(3) + spacing(1) + field(3) + bot_pad + border(1)
    let popup_h = 13u16;
    let popup_w = 50u16;
    let term = frame.size();
    let x = term.x + term.width.saturating_sub(popup_w) / 2;
    let y = term.y + term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w.min(term.width), popup_h.min(term.height));
    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_course)
    } else {
        format!(" {} ", m.edit_course)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Centre the two inputs vertically inside `inner`
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // top spacer
            Constraint::Length(3), // Name field
            Constraint::Length(1), // spacing
            Constraint::Length(3), // Passing Grade field
            Constraint::Min(0),    // bottom spacer
        ])
        .split(inner);

    // Horizontal padding (inset fields by 2 columns on each side)
    let field_area = |rect: Rect| -> Rect {
        let pad = 2u16.min(rect.width / 2);
        Rect::new(
            rect.x + pad,
            rect.y,
            rect.width.saturating_sub(pad * 2),
            rect.height,
        )
    };

    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        field_area(layout[1]),
    );

    render_input_field(
        frame,
        m.passing_grade,
        &app.edit_passing_grade,
        app.input_field == InputField::PassingGrade,
        field_area(layout[3]),
    );
}

pub fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    let show_advanced = app.show_advanced_rules;
    let show_help = app.show_field_help;

    // Determine if OnMinNotMet field is visible (only when min average is set)
    let show_on_min_not_met = show_advanced && !app.edit_min_average.trim().is_empty();

    // --- Compute help text height dynamically ---
    let popup_width = 50u16;
    // Inner text width = popup - 2*margin - 2*border = 50 - 4 - 2 = 44
    let inner_text_width = popup_width.saturating_sub(6).max(1) as usize;
    let help_text = contextual_field_help(app.input_field, m);
    let help_lines_needed: u16 = if help_text.is_empty() {
        1
    } else {
        help_text.len().div_ceil(inner_text_width).max(1) as u16
    };

    // --- Compute popup height based on what's visible ---
    // Basic: Name(3) + spacing(1) + Weight(3) + hint(1) = 8
    // Separator: 2
    // Help hint area: dynamic
    // Outer margin: 4 (margin(2) top+bottom)
    let base_height: u16 = 8 + 2 + help_lines_needed + 4;
    let advanced_height: u16 = if show_advanced {
        let fields = if show_on_min_not_met { 6 } else { 5 };
        fields * 3 + 1 // fields * 3 lines + 1 spacing
    } else {
        0
    };
    let total_height = base_height + advanced_height;

    // Use absolute cell sizing for consistency
    let popup_height = total_height;
    let term = frame.size();
    let x = term.x + term.width.saturating_sub(popup_width) / 2;
    let y = term.y + term.height.saturating_sub(popup_height) / 2;
    let area = Rect::new(
        x,
        y,
        popup_width.min(term.width),
        popup_height.min(term.height),
    );

    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_category_title)
    } else {
        format!(" {} ", m.edit_category)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    // --- Build layout constraints ---
    let mut constraints = vec![
        Constraint::Length(3), // Name
        Constraint::Length(1), // Spacing
        Constraint::Length(3), // Weight
        Constraint::Length(1), // Weight hint
        Constraint::Length(2), // Advanced rules toggle line
    ];

    if show_advanced {
        constraints.push(Constraint::Length(1)); // Spacing after separator
        constraints.push(Constraint::Length(3)); // Drop Lowest
        constraints.push(Constraint::Length(3)); // Averaging Method (toggle)
        constraints.push(Constraint::Length(3)); // Minimum Average
        if show_on_min_not_met {
            constraints.push(Constraint::Length(3)); // On Min Not Met (toggle)
        }
        constraints.push(Constraint::Length(3)); // Min Per Eval
        constraints.push(Constraint::Length(3)); // Round Before Weighting (toggle)
    }

    // Contextual help hint at the bottom (dynamically sized)
    constraints.push(Constraint::Length(help_lines_needed));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(constraints)
        .split(area);

    let mut slot = 0;

    // =====================================================================
    // Basic Fields
    // =====================================================================

    // Name
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[slot],
    );
    slot += 1;

    // Spacing
    slot += 1;

    // Weight
    render_input_field(
        frame,
        m.weight,
        &app.edit_weight,
        app.input_field == InputField::Weight,
        inner[slot],
    );
    slot += 1;

    // Weight hint (inline, no border)
    if let Some(course) = app.current_course() {
        let current_total = course.total_weight();
        let hint = format!("  {}: {:.0}%", m.current_total, current_total);
        let hint_color = if current_total > 100.0 {
            Color::Red
        } else if current_total < 100.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        let hint_widget = Paragraph::new(hint).style(Style::default().fg(hint_color));
        frame.render_widget(hint_widget, inner[slot]);
    }
    slot += 1;

    // =====================================================================
    // Advanced Rules Toggle
    // =====================================================================

    let toggle_text = if show_advanced {
        format!("▼ {} ──────────────────", m.advanced_rules)
    } else {
        format!("▶ {} (Shift+A) ───────", m.advanced_rules)
    };
    let toggle_color = if show_advanced {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let toggle_widget = Paragraph::new(toggle_text).style(Style::default().fg(toggle_color));
    frame.render_widget(toggle_widget, inner[slot]);
    slot += 1;

    // =====================================================================
    // Advanced Fields (only when expanded)
    // =====================================================================

    if show_advanced {
        // Spacing after separator
        slot += 1;

        // Drop Lowest (toggle: 0..5)
        let drop_label = format!("{}", app.edit_drop_lowest);
        render_toggle_field(
            frame,
            m.drop_lowest,
            &drop_label,
            app.input_field == InputField::DropLowest,
            inner[slot],
        );
        slot += 1;

        // Averaging Method (toggle)
        let avg_method_label = match app.edit_averaging_method {
            AveragingMethod::Arithmetic => m.averaging_arithmetic,
            AveragingMethod::Geometric => m.averaging_geometric,
        };
        render_toggle_field(
            frame,
            m.averaging_method,
            avg_method_label,
            app.input_field == InputField::AvgMethod,
            inner[slot],
        );
        slot += 1;

        // Minimum Average
        render_input_field(
            frame,
            m.minimum_average,
            &app.edit_min_average,
            app.input_field == InputField::MinimumAverage,
            inner[slot],
        );
        slot += 1;

        // On Minimum Not Met (toggle, conditional)
        if show_on_min_not_met {
            let action_label = match app.edit_on_min_not_met {
                MinimumNotMetAction::FinalEqualsAverage => m.action_final_equals_avg,
                MinimumNotMetAction::RequiresGlobal => m.action_requires_global,
            };
            render_toggle_field(
                frame,
                m.on_minimum_not_met,
                action_label,
                app.input_field == InputField::OnMinNotMet,
                inner[slot],
            );
            slot += 1;
        }

        // Min Per Eval
        render_input_field(
            frame,
            m.minimum_per_evaluation,
            &app.edit_min_per_eval,
            app.input_field == InputField::MinPerEval,
            inner[slot],
        );
        slot += 1;

        // Round Before Weighting (toggle)
        let round_label = if app.edit_round_before_weighting {
            m.yes
        } else {
            m.no
        };
        render_toggle_field(
            frame,
            m.round_before_weighting,
            round_label,
            app.input_field == InputField::RoundBeforeWeight,
            inner[slot],
        );
        slot += 1;
    }

    // =====================================================================
    // Contextual help hint (always at bottom of popup)
    // =====================================================================

    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(help_widget, inner[slot]);

    // =====================================================================
    // Full help overlay (when ? is pressed)
    // =====================================================================

    if show_help {
        draw_category_help_overlay(frame, m);
    }
}

pub fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let is_editing = !is_new;
    let m = app.messages();
    let area = centered_rect(60, 50, frame.size());
    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_evaluation)
    } else {
        format!(" {} ", m.edit_evaluation)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Grade field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Required grade info
        ])
        .split(area);

    // Grade field FIRST
    render_input_field(
        frame,
        m.grade,
        &app.edit_grade,
        app.input_field == InputField::Grade,
        inner[0],
    );

    // Name field second
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[2],
    );

    // Show what grade is needed in this evaluation to pass
    if let Some(course) = app.current_course() {
        let (info, info_color) = if let (Some(cat_idx), Some(eval_idx)) =
            (app.selected_category, app.selected_evaluation)
        {
            let needed = course.needed_grade_for_evaluation(cat_idx, eval_idx, is_editing);
            let color = match needed.status {
                NeededGradeStatus::Success => Color::Green,
                NeededGradeStatus::Failure => Color::Red,
                NeededGradeStatus::Warning => Color::Yellow,
                NeededGradeStatus::Info => Color::DarkGray,
            };
            (format_needed_grade(&needed, m), color)
        } else {
            let text = format_course_status(course, m);
            let color = match course.current_grade() {
                Some(g) if course.is_passing_grade(g) => Color::Green,
                Some(_) => Color::Red,
                None => Color::DarkGray,
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
        frame.render_widget(info_widget, inner[4]);
    }
}

pub fn draw_save_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.save_as_template_title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Description field
            Constraint::Length(1), // Spacing
            Constraint::Length(2), // Info text
        ])
        .split(area);

    render_input_field(
        frame,
        m.template_name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        m.description,
        &app.edit_description,
        app.input_field == InputField::Description,
        inner[2],
    );

    // Show info about what will be saved
    if let Some(course) = app.current_course() {
        let info = format!(
            "{} {} '{}'",
            m.will_save_categories,
            course.categories.len(),
            course.name
        );
        let info_widget = Paragraph::new(info).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(info_widget, inner[4]);
    }
}

pub fn draw_delete_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let template_name = app
        .current_template()
        .map(|t| t.name.as_str())
        .unwrap_or(m.unknown);

    let question = format!("{} '{}'?", m.delete_template_question, template_name);
    render_delete_confirmation(
        frame,
        m.delete_template,
        &question,
        m.action_cannot_be_undone,
        m.confirm,
        m.cancel,
    );
}

pub fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let (message, warning) = match app.focus {
        Focus::Courses => (m.delete_course_question, m.delete_course_warning),
        Focus::Categories => (m.delete_category_question, m.delete_category_warning),
        Focus::Evaluations => (m.delete_evaluation_question, m.delete_evaluation_warning),
    };

    render_delete_confirmation(
        frame,
        m.confirm_delete,
        message,
        warning,
        m.confirm,
        m.cancel,
    );
}

pub fn draw_language_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(40, 30, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.select_language))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = Language::all()
        .iter()
        .map(|lang| {
            let is_current = *lang == app.language;
            let prefix = if is_current { "● " } else { "  " };
            let style = if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, lang.display_name()),
                style,
            )))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_language));

    frame.render_stateful_widget(list, inner, &mut state);
}
