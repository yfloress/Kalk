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
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::helpers::{
    centered_rect, format_course_status, format_needed_grade, render_input_field,
    render_toggle_field,
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
                        Color::DarkGray
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
    let area = centered_rect(50, 35, frame.size());
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

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);

    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        m.passing_grade,
        &app.edit_passing_grade,
        app.input_field == InputField::PassingGrade,
        inner[2],
    );
}

pub fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    // Determine if OnMinNotMet field is visible (only when min average is set)
    let show_on_min_not_met = !app.edit_min_average.trim().is_empty();
    let field_count: u16 = if show_on_min_not_met { 9 } else { 8 };
    // Each field row = 3 lines (border+content+border), plus 2 for margin, plus 2 for weight hint
    let popup_height = (field_count * 3 + 6).min(85);
    let area = centered_rect(55, popup_height, frame.size());
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

    // Build constraints dynamically
    let mut constraints = vec![
        Constraint::Length(3), // Name
        Constraint::Length(3), // Weight
        Constraint::Length(2), // Weight hint
        Constraint::Length(3), // Drop Lowest
        Constraint::Length(3), // Averaging Method (toggle)
        Constraint::Length(3), // Minimum Average
    ];
    if show_on_min_not_met {
        constraints.push(Constraint::Length(3)); // On Min Not Met (toggle)
    }
    constraints.push(Constraint::Length(3)); // Min Per Eval
    constraints.push(Constraint::Length(3)); // Round Before Weighting (toggle)

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(constraints)
        .split(area);

    let mut slot = 0;

    // Name
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[slot],
    );
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

    // Weight hint
    if let Some(course) = app.current_course() {
        let current_total = course.total_weight();
        let hint = format!("{}: {:.0}%", m.current_total, current_total);
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

    // Drop Lowest
    render_input_field(
        frame,
        m.drop_lowest,
        &app.edit_drop_lowest,
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
}

pub fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool, is_editing: bool) {
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
            (format_needed_grade(course, &needed, m), color)
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
    let area = centered_rect(50, 25, frame.size());
    frame.render_widget(Clear, area);

    let template_name = app
        .current_template()
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    let block = Block::default()
        .title(format!(" {} ", m.delete_template))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            format!("{} '{}'?", m.delete_template_question, template_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            m.action_cannot_be_undone,
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(45, 25, frame.size());
    frame.render_widget(Clear, area);

    let (message, warning) = match app.focus {
        Focus::Courses => (m.delete_course_question, m.delete_course_warning),
        Focus::Categories => (m.delete_category_question, m.delete_category_warning),
        Focus::Evaluations => (m.delete_evaluation_question, m.delete_evaluation_warning),
    };

    let block = Block::default()
        .title(format!(" {} ", m.confirm_delete))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            message,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(warning, Style::default().fg(Color::Yellow))),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
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
