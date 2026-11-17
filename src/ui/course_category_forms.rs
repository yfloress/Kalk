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

//! Course and category fields; layout rows are declared once.

use super::{
    form_view::{FormRow, FormView},
    helpers::{contextual_field_help, draw_category_help_overlay, draw_course_help_overlay},
    icons::icons,
    theme::theme,
};
use crate::app::{App, InputField};
use crate::i18n::Messages;
use crate::model::{AveragingMethod, GlobalExamPolicy, MinimumNotMetAction, WeightValidation};
use ratatui::{Frame, style::Style, text::Line};

pub(super) fn draw_course_popup(frame: &mut Frame, app: &App, is_new: bool) {
    use FormRow::{Input, Toggle};
    use InputField::*;
    let m = app.messages();
    let policy = match app.edit_global_policy {
        GlobalExamPolicy::None => m.global_policy_none,
        GlobalExamPolicy::Weighted { .. } => m.global_policy_weighted,
        GlobalExamPolicy::ReplacesWorstGrade => m.global_policy_replaces,
    };
    let mut rows = vec![
        Input(Name, m.name, &app.edit_name),
        Input(PassingGrade, m.passing_grade, &app.edit_passing_grade),
        Input(Credits, m.credits_label, &app.edit_credits),
        Toggle(GlobalPolicy, m.global_policy, policy.to_string()),
    ];
    if matches!(app.edit_global_policy, GlobalExamPolicy::Weighted { .. }) {
        rows.push(Input(
            GlobalSemesterWeight,
            m.global_semester_weight,
            &app.edit_global_semester_weight,
        ));
        rows.push(Input(
            GlobalExamWeight,
            m.global_exam_weight,
            &app.edit_global_exam_weight,
        ));
    }
    if !matches!(app.edit_global_policy, GlobalExamPolicy::None) {
        rows.push(Input(
            GlobalMinGrade,
            m.global_min_grade,
            &app.edit_global_min_grade,
        ));
    }
    FormView {
        title: if is_new { m.new_course } else { m.edit_course },
        width: 58,
        rows,
        help: contextual_field_help(app, m),
        hints: &[
            ("Enter", m.confirm),
            ("Esc", m.cancel),
            ("Tab", m.next_field),
            ("?", m.help_open),
        ],
    }
    .render(frame, app);
    if app.show_field_help {
        draw_course_help_overlay(frame, m);
    }
}

fn action_label(action: MinimumNotMetAction, m: &Messages) -> &'static str {
    match action {
        MinimumNotMetAction::FinalEqualsAverage => m.action_final_equals_avg,
        MinimumNotMetAction::RequiresGlobal => m.action_requires_global,
        MinimumNotMetAction::FailCourse => m.action_fail_course,
        MinimumNotMetAction::CapFinalGrade => m.action_cap_final_grade,
    }
}

pub(super) fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    use FormRow::{Input, Note, Toggle};
    use InputField::*;
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);
    let mut rows = vec![
        Input(Name, m.name, &app.edit_name),
        Input(Weight, m.weight, &app.edit_weight),
    ];
    if let Some(course) = app.current_course() {
        let color = match course.validate_weights() {
            WeightValidation::Valid => t.status_pass,
            WeightValidation::Over(_) => t.status_fail,
            _ => t.status_warn,
        };
        rows.push(Note(
            Line::from(format!(
                "{}: {:.0}%",
                m.current_total,
                course.total_weight()
            ))
            .style(Style::default().fg(color)),
        ));
    }
    let icon = if app.show_advanced_rules {
        ic.advanced_collapse
    } else {
        ic.advanced_expand
    };
    rows.push(Note(
        Line::from(format!("{}{}  Ctrl+R", icon, m.advanced_rules))
            .style(Style::default().fg(t.status_info)),
    ));
    if app.show_advanced_rules {
        rows.push(Toggle(
            DropLowest,
            m.drop_lowest,
            app.edit_drop_lowest.to_string(),
        ));
        rows.push(Toggle(
            AvgMethod,
            m.averaging_method,
            match app.edit_averaging_method {
                AveragingMethod::Arithmetic => m.averaging_arithmetic,
                AveragingMethod::Geometric => m.averaging_geometric,
            }
            .to_string(),
        ));
        rows.push(Input(
            MinimumAverage,
            m.minimum_average,
            &app.edit_min_average,
        ));
        if !app.edit_min_average.trim().is_empty() {
            rows.push(Toggle(
                OnMinNotMet,
                m.on_minimum_not_met,
                action_label(app.edit_on_min_not_met, m).to_string(),
            ));
        }
        rows.push(Input(
            MinPerEval,
            m.minimum_per_evaluation,
            &app.edit_min_per_eval,
        ));
        if !app.edit_min_per_eval.trim().is_empty() {
            rows.push(Toggle(
                OnMinPerEvalNotMet,
                m.on_minimum_not_met,
                action_label(app.edit_on_min_per_eval_not_met, m).to_string(),
            ));
        }
        rows.push(Input(
            MinOneEval,
            m.minimum_one_eval,
            &app.edit_min_one_eval,
        ));
        if !app.edit_min_one_eval.trim().is_empty() {
            rows.push(Toggle(
                OnMinOneEvalNotMet,
                m.on_minimum_not_met,
                action_label(app.edit_on_min_one_eval_not_met, m).to_string(),
            ));
        }
        rows.push(Input(
            CapFinalGrade,
            m.cap_final_grade,
            &app.edit_cap_final_grade,
        ));
        rows.push(Toggle(
            RoundBeforeWeight,
            m.round_before_weighting,
            if app.edit_round_before_weighting {
                m.yes
            } else {
                m.no
            }
            .to_string(),
        ));
        rows.push(Toggle(
            WeightedEvals,
            m.weighted_evaluations,
            if app.edit_weighted_evaluations {
                m.yes
            } else {
                m.no
            }
            .to_string(),
        ));
    }
    FormView {
        title: if is_new {
            m.new_category_title
        } else {
            m.edit_category
        },
        width: 58,
        rows,
        help: contextual_field_help(app, m),
        hints: &[
            ("Enter", m.confirm),
            ("Esc", m.cancel),
            ("Tab", m.next_field),
            ("?", m.help_open),
        ],
    }
    .render(frame, app);
    if app.show_field_help {
        draw_category_help_overlay(frame, m);
    }
}
