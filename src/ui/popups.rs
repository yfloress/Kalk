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

use super::helpers::{
    centered_rect, contextual_field_help, draw_category_help_overlay, draw_course_help_overlay,
    render_delete_confirmation, render_input_field, render_toggle_field,
};
use super::icons::icons;
use super::theme::theme;
use crate::app::{App, Focus, InputField};
use crate::i18n::{Language, Messages};
use crate::model::{AveragingMethod, GlobalExamPolicy, MinimumNotMetAction};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn draw_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.select_course_template))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

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

    let mut state = ListState::default();
    state.select(Some(app.selected_template));

    frame.render_stateful_widget(list, inner, &mut state);
}

pub fn draw_course_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    let t = theme();
    let popup_w = 55u16;

    // --- Compute help text height dynamically ---
    let inner_text_width = popup_w.saturating_sub(6).max(1) as usize;
    let help_text = contextual_field_help(app, m);
    let help_lines_needed: u16 = if help_text.is_empty() {
        1
    } else {
        help_text.len().div_ceil(inner_text_width).max(1) as u16
    };

    // Dynamic height based on selected global policy:
    // Content: top_sp(1) + Name(3) + sp(1) + PassingGrade(3) + sp(1) + Credits(3)
    //          + sp(1) + GlobalPolicy(3) = 16
    // Weighted adds: sp(1) + SemWeight(3) + sp(1) + GlobWeight(3) = 8
    // Both non-None add eligibility: sp(1) + MinGrade(3) = 4
    // Overhead: border(2)
    let extra = match &app.edit_global_policy {
        GlobalExamPolicy::None => 0u16,
        GlobalExamPolicy::Weighted { .. } => 8 + 4, // weights + eligibility
        GlobalExamPolicy::ReplacesWorstGrade => 4,  // eligibility only
    };
    let popup_h = (16 + extra + help_lines_needed + 2).min(frame.area().height);
    let term = frame.area();
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
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build constraints dynamically
    let mut constraints: Vec<Constraint> = vec![
        Constraint::Length(1), // [0] top spacer
        Constraint::Length(3), // [1] Name field
        Constraint::Length(1), // spacing
        Constraint::Length(3), // [3] Passing Grade field
        Constraint::Length(1), // spacing
        Constraint::Length(3), // [5] Credits field
        Constraint::Length(1), // spacing
        Constraint::Length(3), // [7] Global Policy toggle
    ];

    // Indices for optional fields (tracked for rendering)
    let mut weight_sem_idx: Option<usize> = None;
    let mut weight_glob_idx: Option<usize> = None;
    let mut min_grade_idx: Option<usize> = None;

    if matches!(app.edit_global_policy, GlobalExamPolicy::Weighted { .. }) {
        constraints.push(Constraint::Length(1)); // spacing
        weight_sem_idx = Some(constraints.len());
        constraints.push(Constraint::Length(3)); // Semester Weight
        constraints.push(Constraint::Length(1)); // spacing
        weight_glob_idx = Some(constraints.len());
        constraints.push(Constraint::Length(3)); // Global Weight
    }

    if !matches!(app.edit_global_policy, GlobalExamPolicy::None) {
        constraints.push(Constraint::Length(1)); // spacing
        min_grade_idx = Some(constraints.len());
        constraints.push(Constraint::Length(3)); // Min Grade
    }

    // Contextual help hint at the bottom (dynamically sized)
    constraints.push(Constraint::Length(help_lines_needed));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints(constraints)
        .split(inner);

    // -- Fixed fields --
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        layout[1],
    );

    render_input_field(
        frame,
        m.passing_grade,
        &app.edit_passing_grade,
        app.input_field == InputField::PassingGrade,
        layout[3],
    );

    render_input_field(
        frame,
        m.credits_label,
        &app.edit_credits,
        app.input_field == InputField::Credits,
        layout[5],
    );

    // Global Policy toggle
    let policy_label = match &app.edit_global_policy {
        GlobalExamPolicy::None => m.global_policy_none,
        GlobalExamPolicy::Weighted { .. } => m.global_policy_weighted,
        GlobalExamPolicy::ReplacesWorstGrade => m.global_policy_replaces,
    };
    render_toggle_field(
        frame,
        m.global_policy,
        policy_label,
        app.input_field == InputField::GlobalPolicy,
        layout[7],
    );

    // -- Conditional Weighted fields --
    if let Some(idx) = weight_sem_idx {
        render_input_field(
            frame,
            m.global_semester_weight,
            &app.edit_global_semester_weight,
            app.input_field == InputField::GlobalSemesterWeight,
            layout[idx],
        );
    }
    if let Some(idx) = weight_glob_idx {
        render_input_field(
            frame,
            m.global_exam_weight,
            &app.edit_global_exam_weight,
            app.input_field == InputField::GlobalExamWeight,
            layout[idx],
        );
    }

    // -- Conditional Eligibility fields --
    if let Some(idx) = min_grade_idx {
        render_input_field(
            frame,
            m.global_min_grade,
            &app.edit_global_min_grade,
            app.input_field == InputField::GlobalMinGrade,
            layout[idx],
        );
    }

    // -- Contextual help hint (always at bottom of popup) --
    let help_slot = layout.len() - 1;
    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(t.text_muted))
        .wrap(Wrap { trim: true });
    frame.render_widget(help_widget, layout[help_slot]);

    // -- Full help overlay (when ? is pressed) --
    if app.show_field_help {
        draw_course_help_overlay(frame, m);
    }
}

/// Render an "If Not Met" action toggle for a minimum rule.
fn render_action_toggle(
    frame: &mut Frame,
    m: &Messages,
    action: MinimumNotMetAction,
    field: InputField,
    current_field: InputField,
    area: Rect,
) {
    let label = match action {
        MinimumNotMetAction::FinalEqualsAverage => m.action_final_equals_avg,
        MinimumNotMetAction::RequiresGlobal => m.action_requires_global,
        MinimumNotMetAction::FailCourse => m.action_fail_course,
    };
    render_toggle_field(
        frame,
        m.on_minimum_not_met,
        label,
        current_field == field,
        area,
    );
}

pub fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let show_advanced = app.show_advanced_rules;
    let show_help = app.show_field_help;

    // Each rule has its own independent "If Not Met" toggle, shown only when that rule has a value
    let show_on_min_avg_not_met = show_advanced && !app.edit_min_average.trim().is_empty();
    let show_on_min_per_eval_not_met = show_advanced && !app.edit_min_per_eval.trim().is_empty();
    let show_on_min_one_eval_not_met = show_advanced && !app.edit_min_one_eval.trim().is_empty();

    let popup_width = 50u16;
    let inner_text_width = popup_width.saturating_sub(6).max(1) as usize;
    let help_text = contextual_field_help(app, m);
    let help_lines_needed: u16 = if help_text.is_empty() {
        1
    } else {
        help_text.len().div_ceil(inner_text_width).max(1) as u16
    };

    let base_height: u16 = 8 + 2 + help_lines_needed + 4;
    let advanced_height: u16 = if show_advanced {
        // Base: DropLowest + AvgMethod + MinAvg + MinPerEval + MinOneEval + RoundBeforeWeight + WeightedEvals = 7
        let mut fields: u16 = 7;
        if show_on_min_avg_not_met {
            fields += 1;
        }
        if show_on_min_per_eval_not_met {
            fields += 1;
        }
        if show_on_min_one_eval_not_met {
            fields += 1;
        }
        fields * 3 + 1
    } else {
        0
    };
    let popup_height = base_height + advanced_height;
    let term = frame.area();
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
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    frame.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(3), // Name
        Constraint::Length(1), // Spacing
        Constraint::Length(3), // Weight
        Constraint::Length(1), // Weight hint
        Constraint::Length(2), // Advanced rules toggle line
    ];
    if show_advanced {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(3)); // Drop Lowest
        constraints.push(Constraint::Length(3)); // Averaging Method
        constraints.push(Constraint::Length(3)); // Minimum Average
        if show_on_min_avg_not_met {
            constraints.push(Constraint::Length(3));
        }
        constraints.push(Constraint::Length(3)); // Min Per Eval
        if show_on_min_per_eval_not_met {
            constraints.push(Constraint::Length(3));
        }
        constraints.push(Constraint::Length(3)); // Min One Eval
        if show_on_min_one_eval_not_met {
            constraints.push(Constraint::Length(3));
        }
        constraints.push(Constraint::Length(3)); // Round Before Weighting
        constraints.push(Constraint::Length(3)); // Weighted Evaluations
    }
    constraints.push(Constraint::Length(help_lines_needed));

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
    slot += 1; // Spacing

    // Weight
    render_input_field(
        frame,
        m.weight,
        &app.edit_weight,
        app.input_field == InputField::Weight,
        inner[slot],
    );
    slot += 1;
    if let Some(course) = app.current_course() {
        let current_total = course.total_weight();
        let display_total = if current_total == 0.0 {
            0.0
        } else {
            current_total
        };
        let hint = format!("  {}: {:.0}%", m.current_total, display_total);
        let hint_color = if current_total > 100.0 {
            t.status_fail
        } else if current_total < 100.0 {
            t.status_warn
        } else {
            t.status_pass
        };
        let hint_widget = Paragraph::new(hint).style(Style::default().fg(hint_color));
        frame.render_widget(hint_widget, inner[slot]);
    }
    slot += 1;

    let toggle_text = if show_advanced {
        format!(
            "{}{} (Ctrl+R) ────────",
            ic.advanced_collapse, m.advanced_rules
        )
    } else {
        format!(
            "{}{} (Ctrl+R) ───────",
            ic.advanced_expand, m.advanced_rules
        )
    };
    let toggle_color = if show_advanced {
        t.status_info
    } else {
        t.text_muted
    };
    let toggle_widget = Paragraph::new(toggle_text).style(Style::default().fg(toggle_color));
    frame.render_widget(toggle_widget, inner[slot]);
    slot += 1;

    if show_advanced {
        slot += 1; // Spacing after separator

        let drop_label = format!("{}", app.edit_drop_lowest);
        render_toggle_field(
            frame,
            m.drop_lowest,
            &drop_label,
            app.input_field == InputField::DropLowest,
            inner[slot],
        );
        slot += 1;
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
        render_input_field(
            frame,
            m.minimum_average,
            &app.edit_min_average,
            app.input_field == InputField::MinimumAverage,
            inner[slot],
        );
        slot += 1;
        if show_on_min_avg_not_met {
            render_action_toggle(
                frame,
                m,
                app.edit_on_min_not_met,
                InputField::OnMinNotMet,
                app.input_field,
                inner[slot],
            );
            slot += 1;
        }
        render_input_field(
            frame,
            m.minimum_per_evaluation,
            &app.edit_min_per_eval,
            app.input_field == InputField::MinPerEval,
            inner[slot],
        );
        slot += 1;
        if show_on_min_per_eval_not_met {
            render_action_toggle(
                frame,
                m,
                app.edit_on_min_per_eval_not_met,
                InputField::OnMinPerEvalNotMet,
                app.input_field,
                inner[slot],
            );
            slot += 1;
        }
        render_input_field(
            frame,
            m.minimum_one_eval,
            &app.edit_min_one_eval,
            app.input_field == InputField::MinOneEval,
            inner[slot],
        );
        slot += 1;
        if show_on_min_one_eval_not_met {
            render_action_toggle(
                frame,
                m,
                app.edit_on_min_one_eval_not_met,
                InputField::OnMinOneEvalNotMet,
                app.input_field,
                inner[slot],
            );
            slot += 1;
        }
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

        // Weighted Evaluations toggle
        let weighted_label = if app.edit_weighted_evaluations {
            m.yes
        } else {
            m.no
        };
        render_toggle_field(
            frame,
            m.weighted_evaluations,
            weighted_label,
            app.input_field == InputField::WeightedEvals,
            inner[slot],
        );
        slot += 1;
    }

    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(t.text_muted))
        .wrap(Wrap { trim: true });
    frame.render_widget(help_widget, inner[slot]);
    if show_help {
        draw_category_help_overlay(frame, m);
    }
}

pub fn draw_save_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {} ", m.save_as_template_title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
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
        let info_widget = Paragraph::new(info).style(Style::default().fg(t.text_muted));
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

/// Render the global keyboard cheat-sheet overlay.
/// Groups shortcuts by purpose so the user can scan instead of read top-down.
pub fn draw_help_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let term = frame.area();

    // Centred popup with breathing room around the content.
    let popup_w = 62u16.min(term.width.saturating_sub(2));
    let popup_h = 38u16.min(term.height.saturating_sub(2));
    let x = term.width.saturating_sub(popup_w) / 2;
    let y = term.height.saturating_sub(popup_h) / 2;
    let area = Rect::new(x, y, popup_w, popup_h);

    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {} ", m.help_title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let key_style = Style::default()
        .fg(t.footer_key)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(t.text_primary);
    let group_style = Style::default()
        .fg(t.status_info)
        .add_modifier(Modifier::BOLD);
    let muted_style = Style::default().fg(t.text_muted);

    let row = |key: &str, desc: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {:<12} ", key), key_style),
            Span::styled(desc.to_string(), desc_style),
        ])
    };
    let group =
        |title: &str| -> Line<'static> { Line::from(Span::styled(title.to_string(), group_style)) };
    let blank = || Line::from("");

    let undo_redo = format!("{} / {}", m.undo, m.redo);
    let copy_paste = format!("{} / {}", m.yank, m.paste);

    let lines: Vec<Line> = vec![
        group(m.help_group_global),
        row("q", m.quit),
        row("?", m.help_open),
        row("Esc", m.help_close_popup),
        blank(),
        group(m.semesters),
        row("h", m.home_open),
        row("l / Enter", m.semester_open),
        row("n / r / d", m.semester_manage),
        row("J / K", m.semester_move),
        blank(),
        group(m.help_group_navigation),
        row("k / j  ↑/↓", m.help_move_updown),
        row("h / l  ←/→", m.help_focus_lr),
        row("h", m.home_open),
        row("Tab", m.help_cycle_focus),
        row("Home / End", m.jump_first_last),
        blank(),
        group(m.help_group_editing),
        row("n", m.help_new_generic),
        row("Enter", m.help_edit_selected),
        row("d", m.help_delete_selected),
        row("Ctrl+N", m.bulk_add),
        row("Ctrl+Z / Y", &undo_redo),
        blank(),
        group(m.help_group_actions),
        row("g", m.enter_global),
        row("t", m.save_as_template),
        row("b", m.balance),
        row("y / p", &copy_paste),
        blank(),
        group(m.help_group_view),
        row("S", m.settings),
        row("L", m.change_language),
        blank(),
        Line::from(Span::styled(
            format!("  {}", m.help_close_hint),
            muted_style,
        )),
    ];

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

pub fn draw_language_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {}{} ", ic.language, m.select_language))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

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

    let mut state = ListState::default();
    state.select(Some(app.selected_language));

    frame.render_stateful_widget(list, inner, &mut state);
}
