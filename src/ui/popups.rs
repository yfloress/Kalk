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
//! Popup dialogs: template selection, course/category/evaluation editing,
//! delete confirmations, language selection, save-as-template, and settings.

use super::helpers::{centered_rect, render_delete_confirmation, render_input_field};
use super::icons::icons;
use super::keyhints::render_hint;
use super::surfaces::popup_block;
use super::theme::theme;
use crate::app::{App, Focus, InputField};
use crate::i18n::Language;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

pub fn draw_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = popup_block(format!(" {} ", m.select_course_template));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // The synthetic "Create with AI" entry is always at index 0, above the
    // real templates.  Built-in templates follow, then user templates.
    let all_templates = app.all_templates();
    let built_in_count = app.built_in_templates.len();

    let mut items: Vec<ListItem> = Vec::with_capacity(1 + all_templates.len());

    // AI entry — uses the popup-border (Lavender) accent so it stands apart.
    items.push(ListItem::new(vec![
        Line::from(Span::styled(
            format!("{}{}", ic.template, m.tpl_ai),
            Style::default()
                .fg(t.popup_border)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", m.tpl_ai_desc),
            Style::default().fg(t.text_secondary),
        )),
    ]));

    items.extend(all_templates.iter().enumerate().map(|(i, tmpl)| {
        let is_user_template = i >= built_in_count;
        let prefix = if is_user_template {
            ic.user_template
        } else {
            ic.template
        };

        ListItem::new(vec![
            Line::from(Span::styled(
                format!("{}{}", prefix, tmpl.name),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("  {}", tmpl.description),
                Style::default().fg(if is_user_template {
                    t.user_template
                } else {
                    t.text_secondary
                }),
            )),
        ])
    }));

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let mut state = ListState::default();
    state.select(Some(app.selected_template));

    frame.render_stateful_widget(list, rows[0], &mut state);
    render_hint(
        frame,
        &[
            ("j/k", m.help_move_updown),
            ("/", m.search_title),
            ("Enter", m.confirm),
            ("d", m.delete),
            ("Esc", m.cancel),
        ],
        rows[1],
    );
}

pub fn draw_save_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);
    let block = popup_block(format!(" {} ", m.save_as_template_title));
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
            Constraint::Length(1), // Key hints
        ])
        .split(area);

    render_input_field(
        frame,
        app,
        m.template_name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        app,
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
        let info_widget = Paragraph::new(info).style(Style::default().fg(t.text_muted));
        frame.render_widget(info_widget, inner[4]);
    }

    render_hint(
        frame,
        &[
            ("Tab", m.help_cycle_focus),
            ("Enter", m.confirm),
            ("Esc", m.cancel),
        ],
        inner[5],
    );
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
        app.use_nerd_fonts,
    );
}

pub fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();

    // Build a warning string that includes the concrete blast radius of the
    // delete (how many categories/evaluations the user will lose), so the
    // confirmation is informed rather than abstract.
    let (message, warning_owned) = match app.focus {
        Focus::Courses => {
            let course = app.current_course();
            let cat_count = course.map(|c| c.categories.len()).unwrap_or(0);
            let eval_count: usize = course
                .map(|c| c.categories.iter().map(|cat| cat.evaluations.len()).sum())
                .unwrap_or(0);
            let cat_label = if cat_count == 1 {
                m.category_singular
            } else {
                m.category_plural
            };
            let eval_label = if eval_count == 1 {
                m.eval_singular
            } else {
                m.eval_plural
            };
            let warning = format!(
                "{} ({} {}, {} {})",
                m.delete_course_warning, cat_count, cat_label, eval_count, eval_label
            );
            (m.delete_course_question, warning)
        }
        Focus::Categories => {
            let eval_count = app
                .current_category()
                .map(|c| c.evaluations.len())
                .unwrap_or(0);
            let eval_label = if eval_count == 1 {
                m.eval_singular
            } else {
                m.eval_plural
            };
            let warning = format!(
                "{} ({} {})",
                m.delete_category_warning, eval_count, eval_label
            );
            (m.delete_category_question, warning)
        }
        Focus::Evaluations => (
            m.delete_evaluation_question,
            m.delete_evaluation_warning.to_string(),
        ),
    };

    render_delete_confirmation(
        frame,
        m.confirm_delete,
        message,
        &warning_owned,
        m.confirm,
        m.cancel,
        app.use_nerd_fonts,
    );
}

pub fn draw_language_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);
    let block = popup_block(format!(" {}{} ", ic.language, m.select_language));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = Language::all()
        .iter()
        .map(|lang| {
            let is_current = *lang == app.language;
            let prefix = if is_current {
                ic.selected
            } else {
                ic.unselected
            };
            let style = if is_current {
                Style::default()
                    .fg(t.footer_key)
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
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let mut state = ListState::default();
    state.select(Some(app.selected_language));

    frame.render_stateful_widget(list, rows[0], &mut state);
    render_hint(
        frame,
        &[
            ("j/k", m.help_move_updown),
            ("Enter", m.confirm),
            ("Esc", m.cancel),
        ],
        rows[1],
    );
}
