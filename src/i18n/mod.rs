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

//! Internationalization (i18n) support for Kalk.
//!
//! This module provides translations for all UI text in the application.
//! Currently supports English (default) and Spanish.

use serde::{Deserialize, Serialize};

/// Available languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    /// Get the display name of the language.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
        }
    }

    /// Get all available languages.
    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Spanish]
    }
}

/// All translatable messages in the application.
#[derive(Debug, Clone)]
pub struct Messages {
    // Panel titles
    pub courses: &'static str,
    pub categories: &'static str,
    pub evaluations: &'static str,

    // Course panel
    pub select_course_to_view: &'static str,
    pub select_category_to_view: &'static str,
    pub select_course_first: &'static str,
    pub course_average: &'static str,
    pub no_grades_yet: &'static str,
    pub passed: &'static str,
    pub failed: &'static str,
    pub current: &'static str,
    pub need: &'static str,
    pub cannot_pass: &'static str,
    pub no_evaluations: &'static str,

    // Weight validation
    pub weights_ok: &'static str,
    pub weights_warning: &'static str,
    pub weights_error: &'static str,
    pub no_categories: &'static str,

    // Footer / Keybindings
    pub quit: &'static str,
    pub new: &'static str,
    pub new_category: &'static str,
    pub new_eval: &'static str,
    pub edit: &'static str,
    pub delete: &'static str,
    pub save_as_template: &'static str,
    pub balance: &'static str,
    pub yank: &'static str,
    pub paste: &'static str,
    pub bulk_add: &'static str,
    pub select: &'static str,
    pub confirm: &'static str,
    pub cancel: &'static str,
    pub next_field: &'static str,
    pub change_language: &'static str,
    /// Footer label for the `g` shortcut that opens the global-grade input.
    pub enter_global: &'static str,
    /// Footer label for the Home/End jump shortcuts (combined "First / Last").
    pub jump_first_last: &'static str,

    // Popups
    pub select_course_template: &'static str,
    pub new_course: &'static str,
    pub edit_course: &'static str,
    pub new_category_title: &'static str,
    pub edit_category: &'static str,
    pub new_evaluation: &'static str,
    pub edit_evaluation: &'static str,
    pub confirm_delete: &'static str,
    pub delete_course_question: &'static str,
    pub delete_category_question: &'static str,
    pub delete_evaluation_question: &'static str,
    pub delete_course_warning: &'static str,
    pub delete_category_warning: &'static str,
    pub delete_evaluation_warning: &'static str,
    pub save_as_template_title: &'static str,
    pub delete_template: &'static str,
    pub delete_template_question: &'static str,
    pub action_cannot_be_undone: &'static str,
    pub select_language: &'static str,
    pub bulk_add_title: &'static str,
    pub bulk_add_hint: &'static str,
    pub bulk_add_count: &'static str,

    // Form labels
    pub name: &'static str,
    pub passing_grade: &'static str,
    pub weight: &'static str,
    pub grade: &'static str,
    pub template_name: &'static str,
    pub description: &'static str,
    pub current_total: &'static str,
    pub will_save_categories: &'static str,
    pub need_to_pass: &'static str,
    pub graded: &'static str,
    pub avg: &'static str,

    // Template category names
    pub tpl_exam: &'static str,
    pub tpl_quiz: &'static str,
    pub tpl_homework: &'static str,
    pub tpl_lab: &'static str,
    pub tpl_project: &'static str,
    pub tpl_assessment: &'static str,

    // Template names and descriptions
    pub tpl_exam_quiz_80_20: &'static str,
    pub tpl_exam_quiz_80_20_desc: &'static str,
    pub tpl_exam_quiz_70_30: &'static str,
    pub tpl_exam_quiz_70_30_desc: &'static str,
    pub tpl_exam_homework_quiz: &'static str,
    pub tpl_exam_homework_quiz_desc: &'static str,
    pub tpl_exam_labs: &'static str,
    pub tpl_exam_labs_desc: &'static str,
    pub tpl_project_exams: &'static str,
    pub tpl_project_exams_desc: &'static str,
    pub tpl_exams_only: &'static str,
    pub tpl_exams_only_desc: &'static str,
    pub tpl_continuous: &'static str,
    pub tpl_continuous_desc: &'static str,
    pub tpl_custom: &'static str,
    pub tpl_custom_desc: &'static str,

    // Status messages
    pub save_error: &'static str,
    pub save_template_error: &'static str,
    pub config_save_error: &'static str,
    pub load_error: &'static str,
    pub load_template_error: &'static str,
    pub config_load_warning: &'static str,

    // Misc
    pub unknown: &'static str,
    pub empty_template_desc: &'static str,
    pub template_suffix: &'static str,
    pub passing: &'static str,
    pub below_passing: &'static str,
    pub need_grade_impossible: &'static str,
    pub need_grade_any: &'static str,
    pub need_grade_in_eval: &'static str,

    // Category rules labels
    pub advanced_rules: &'static str,
    pub advanced_rules_show: &'static str,
    pub advanced_rules_hide: &'static str,
    pub drop_lowest: &'static str,
    pub averaging_method: &'static str,
    pub averaging_arithmetic: &'static str,
    pub averaging_geometric: &'static str,
    pub minimum_average: &'static str,
    pub on_minimum_not_met: &'static str,
    pub action_final_equals_avg: &'static str,
    pub action_requires_global: &'static str,
    pub action_fail_course: &'static str,
    pub minimum_per_evaluation: &'static str,
    pub minimum_one_eval: &'static str,
    pub round_before_weighting: &'static str,
    pub weighted_evaluations: &'static str,
    pub eval_weight: &'static str,
    pub yes: &'static str,
    pub no: &'static str,

    // Field help descriptions (shown in help panel)
    pub help_toggle: &'static str,
    pub help_name: &'static str,
    pub help_weight: &'static str,
    pub help_drop_lowest: &'static str,
    pub help_averaging_method: &'static str,
    pub help_minimum_average: &'static str,
    pub help_on_min_not_met: &'static str,
    pub help_min_per_eval: &'static str,
    pub help_min_one_eval: &'static str,
    pub help_round_before_weighting: &'static str,
    pub help_weighted_evaluations: &'static str,
    pub help_passing_grade: &'static str,
    pub help_global_policy: &'static str,
    pub help_global_weights: &'static str,
    pub help_global_eligibility: &'static str,

    // Status indicators for rules
    pub rules_active: &'static str,
    pub minimum_not_met: &'static str,
    pub needs_global: &'static str,
    pub grade_capped_by: &'static str,
    pub dropped: &'static str,
    pub eval_below_min: &'static str,
    pub eval_weights_ok: &'static str,
    pub eval_weights_warning: &'static str,
    pub eval_weights_error: &'static str,

    // Global exam
    pub global_exam: &'static str,
    pub global_policy: &'static str,
    pub global_policy_none: &'static str,
    pub global_policy_weighted: &'static str,
    pub global_policy_replaces: &'static str,
    pub global_semester_weight: &'static str,
    pub global_exam_weight: &'static str,
    pub global_min_grade: &'static str,
    pub global_grade: &'static str,
    pub global_needed: &'static str,
    pub global_result: &'static str,
    pub global_no_policy: &'static str,
    pub global_enter_hint: &'static str,

    // Settings
    pub settings: &'static str,
    pub settings_nerd_fonts: &'static str,
    pub settings_nerd_fonts_desc: &'static str,
    pub settings_language: &'static str,
    pub enabled: &'static str,
    pub disabled: &'static str,
    pub toggle: &'static str,
    pub yanked_eval: &'static str,
    pub pasted_eval: &'static str,
    pub no_eval_to_yank: &'static str,
    pub no_eval_in_clipboard: &'static str,
    pub bulk_added_evals: &'static str,

    // Undo / redo
    pub undo: &'static str,
    pub redo: &'static str,
    pub undo_done: &'static str,
    pub redo_done: &'static str,
    pub nothing_to_undo: &'static str,
    pub nothing_to_redo: &'static str,

    // Delete impact (singular / plural forms)
    pub eval_singular: &'static str,
    pub eval_plural: &'static str,
    pub category_singular: &'static str,
    pub category_plural: &'static str,

    // Help overlay
    pub help_title: &'static str,
    pub help_close_hint: &'static str,
    pub help_open: &'static str,
    pub help_group_global: &'static str,
    pub help_group_navigation: &'static str,
    pub help_group_editing: &'static str,
    pub help_group_actions: &'static str,
    pub help_group_view: &'static str,
    pub help_cycle_focus: &'static str,
    pub help_focus_lr: &'static str,
    pub help_move_updown: &'static str,
    pub help_close_popup: &'static str,
    pub help_new_generic: &'static str,
    pub help_edit_selected: &'static str,
    pub help_delete_selected: &'static str,

    // AI import wizard
    pub tpl_ai: &'static str,
    pub tpl_ai_desc: &'static str,
    pub import_step1_title: &'static str,
    pub import_step1_hint: &'static str,
    pub import_step1_copy: &'static str,
    pub import_step1_copied: &'static str,
    pub import_step1_next: &'static str,
    pub import_step1_fullscreen: &'static str,
    pub import_step1_fullscreen_exit: &'static str,
    pub import_step2_title: &'static str,
    pub import_step2_hint: &'static str,
    pub import_step2_back: &'static str,
    pub import_step3_title: &'static str,
    pub import_step3_total_weight: &'static str,
    pub import_step3_evaluations: &'static str,
    pub import_step3_save_as_template: &'static str,
    pub import_step3_confirm: &'static str,
    pub import_warning_weight_not_100: &'static str,
    pub import_err_invalid_json: &'static str,
    pub import_err_schema_version: &'static str,
    pub import_err_empty_name: &'static str,
    pub import_err_no_categories: &'static str,
    pub import_imported_ok: &'static str,
    pub import_copy_suffix: &'static str,
    pub import_renamed_to: &'static str,
    /// Full prompt template handed to the AI (raw multi-line string).
    pub import_prompt: &'static str,

    // Semesters
    /// Prefix for auto-generated semester names ("Semester 1", "Semester 2").
    pub semester_name_prefix: &'static str,
    pub welcome_language_title: &'static str,
    pub welcome_fonts_title: &'static str,
    pub welcome_fonts_question: &'static str,
    pub welcome_fonts_hint: &'static str,
    pub welcome_yes: &'static str,
    pub welcome_no: &'static str,
    pub welcome_next: &'static str,
    pub welcome_start: &'static str,
    pub welcome_back: &'static str,
    pub metric_global: &'static str,
    pub metric_pending_word: &'static str,
    pub metric_minimums: &'static str,
    pub metric_unmet_word: &'static str,
    pub metric_in_play: &'static str,
    pub metric_unrecoverable: &'static str,
    pub metric_needs: &'static str,
    pub metric_in: &'static str,
    pub metric_lost: &'static str,
    pub semester_manage: &'static str,
    pub home_open: &'static str,
    pub metric_status: &'static str,
    pub metric_progress: &'static str,
    pub metric_evaluations: &'static str,
    pub metric_trend: &'static str,
    pub metric_courses_breakdown: &'static str,
    pub credits_label: &'static str,
    pub course_singular: &'static str,
    pub course_plural: &'static str,
    pub semesters: &'static str,
    pub semester_new: &'static str,
    pub semester_rename: &'static str,
    pub semester_delete: &'static str,
    pub semester_move: &'static str,
    pub semester_open: &'static str,
    pub semester_cannot_delete_last: &'static str,
    pub semester_delete_warning: &'static str,
    pub semester_name_label: &'static str,
    pub home_no_courses: &'static str,
    pub metric_average: &'static str,
    pub metric_average_weighted: &'static str,
    pub metric_passed: &'static str,
    pub metric_failing: &'static str,
    pub metric_credits: &'static str,
    pub metric_credits_at_risk: &'static str,
    pub metric_critical: &'static str,
    pub metric_cumulative: &'static str,
    pub metric_no_data: &'static str,
}

mod en;
mod es;

pub use en::EN;
pub use es::ES;

impl Language {
    /// Get the messages for this language.
    pub fn messages(&self) -> &'static Messages {
        match self {
            Language::English => &EN,
            Language::Spanish => &ES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_default_is_english() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn test_language_display_names() {
        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::Spanish.display_name(), "Español");
    }

    #[test]
    fn hint_labels_never_embed_their_own_key() {
        // The key belongs to the code and the label to the translation. Baking
        // "Enter: Import" into a message makes it unrenderable as [Enter]
        // Import, and lets a translator translate the key by accident.
        for m in [&EN, &ES] {
            for label in [
                m.import_step1_copy,
                m.import_step1_next,
                m.import_step1_fullscreen,
                m.import_step1_fullscreen_exit,
                m.import_step2_back,
                m.import_step3_save_as_template,
                m.import_step3_confirm,
                m.help_close_hint,
                m.confirm,
                m.cancel,
                m.toggle,
                m.semester_open,
                m.semester_new,
                m.semester_delete,
                m.welcome_next,
                m.welcome_start,
            ] {
                assert!(!label.contains(':'), "`{label}` embeds its key");
                for key in ["Enter", "Esc", "Ctrl", "Tab", "Space"] {
                    assert!(!label.starts_with(key), "`{label}` embeds its key");
                }
            }
        }
    }

    #[test]
    fn test_all_languages() {
        let all = Language::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&Language::English));
        assert!(all.contains(&Language::Spanish));
    }
}
