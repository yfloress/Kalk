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

//! Form handling methods for the application.
//!
//! This module contains methods that handle form input, confirmation, and
//! cancellation for courses, categories, and evaluations. Secondary actions
//! (templates, language, settings, global grade, yank/paste, bulk-add) live
//! in `actions.rs`.

use crate::model::{
    Category, CategoryRules, Course, DEFAULT_PASSING_GRADE, Evaluation, GlobalEligibility,
    GlobalExamPolicy, MAX_GRADE, MIN_GRADE,
};

use super::{App, InputField, Screen};

/// Parse a decimal string that may use comma as the decimal separator.
/// Normalises `,` → `.` before parsing.
fn parse_decimal(s: &str) -> Result<f64, std::num::ParseFloatError> {
    s.trim().replace(',', ".").parse::<f64>()
}

/// Format a grade for display in form fields.
/// Shows decimals only when the value is not a whole number (e.g. 40.1 stays
/// "40.1", but 55.0 becomes "55").
fn format_grade(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        // Up to 2 decimal places, trimming trailing zeros
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

impl App {
    // =========================================================================
    // Form Handling — Course
    // =========================================================================

    pub fn start_new_course(&mut self) {
        self.screen = Screen::SelectingTemplate;
        self.selected_template = 0;
    }

    pub fn confirm_template_selection(&mut self) {
        // The synthetic AI entry at index 0 launches the import wizard instead
        // of opening the manual course form.
        if self.is_ai_template_selected() {
            self.start_ai_import();
            return;
        }

        self.screen = Screen::EditingCourse { is_new: true };
        self.input_field = InputField::Name;
        self.edit_name.clear();
        self.edit_passing_grade = format_grade(DEFAULT_PASSING_GRADE);
        self.show_field_help = false;

        // Pre-populate global fields from the selected template.
        // Clone the fields we need before mutating self (borrow checker).
        let tpl_global = self
            .current_template()
            .map(|t| (t.global_policy.clone(), t.global_eligibility.clone()));

        if let Some((policy, eligibility)) = tpl_global {
            match &policy {
                GlobalExamPolicy::Weighted {
                    semester_weight,
                    global_weight,
                } => {
                    self.edit_global_semester_weight = format_grade(semester_weight * 100.0);
                    self.edit_global_exam_weight = format_grade(global_weight * 100.0);
                }
                _ => {
                    self.edit_global_semester_weight = "70".to_string();
                    self.edit_global_exam_weight = "30".to_string();
                }
            }
            self.edit_global_min_grade =
                eligibility.min_grade.map(format_grade).unwrap_or_default();
            self.edit_global_policy = policy;
        } else {
            self.init_global_fields_default();
        }
    }

    pub fn start_edit_course(&mut self) {
        let Some(idx) = self.selected_course else {
            return;
        };
        let Some(course) = self.courses.get(idx) else {
            return;
        };

        self.screen = Screen::EditingCourse { is_new: false };
        self.input_field = InputField::Name;
        self.show_field_help = false;
        self.edit_name = course.name.clone();
        let pg = course.passing_grade;
        self.edit_passing_grade = format_grade(pg);

        // Load global exam fields from the course
        self.edit_global_policy = course.global_policy.clone();
        match &course.global_policy {
            GlobalExamPolicy::Weighted {
                semester_weight,
                global_weight,
            } => {
                self.edit_global_semester_weight = format_grade(semester_weight * 100.0);
                self.edit_global_exam_weight = format_grade(global_weight * 100.0);
            }
            _ => {
                self.edit_global_semester_weight = "70".to_string();
                self.edit_global_exam_weight = "30".to_string();
            }
        }
        self.edit_global_min_grade = course
            .global_eligibility
            .min_grade
            .map(format_grade)
            .unwrap_or_default();
    }

    pub fn confirm_course(&mut self) {
        let name = self.edit_name.trim().to_string();
        let passing_grade: f64 = parse_decimal(&self.edit_passing_grade)
            .unwrap_or(DEFAULT_PASSING_GRADE)
            .clamp(MIN_GRADE, MAX_GRADE);

        if name.is_empty() {
            return;
        }

        // Build global policy from form fields
        let global_policy = self.build_global_policy();
        let global_eligibility = self.build_global_eligibility();

        // Snapshot for undo (after validation, before mutation).
        self.push_undo();

        match self.screen {
            Screen::EditingCourse { is_new: true } => {
                let mut course = if let Some(template) = self.current_template() {
                    Course::from_template(name, passing_grade, template)
                } else {
                    Course::new(name, passing_grade)
                };
                course.global_policy = global_policy;
                course.global_eligibility = global_eligibility;
                self.courses.push(course);
                self.selected_course = Some(self.courses.len() - 1);
                self.reset_category_selection();
                self.reset_evaluation_selection();
            }
            Screen::EditingCourse { is_new: false } => {
                if let Some(idx) = self.selected_course
                    && let Some(course) = self.courses.get_mut(idx)
                {
                    course.name = name;
                    course.passing_grade = passing_grade;
                    course.global_policy = global_policy;
                    course.global_eligibility = global_eligibility;
                }
            }
            _ => {}
        }

        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    /// Initialise global exam form fields to defaults (no global).
    fn init_global_fields_default(&mut self) {
        self.edit_global_policy = GlobalExamPolicy::None;
        self.edit_global_semester_weight = "70".to_string();
        self.edit_global_exam_weight = "30".to_string();
        self.edit_global_min_grade.clear();
    }

    /// Build a `GlobalExamPolicy` from the current form fields.
    fn build_global_policy(&self) -> GlobalExamPolicy {
        match &self.edit_global_policy {
            GlobalExamPolicy::None => GlobalExamPolicy::None,
            GlobalExamPolicy::Weighted { .. } => {
                let sw = parse_decimal(&self.edit_global_semester_weight)
                    .unwrap_or(70.0)
                    .clamp(0.0, 100.0)
                    / 100.0;
                let gw = parse_decimal(&self.edit_global_exam_weight)
                    .unwrap_or(30.0)
                    .clamp(0.0, 100.0)
                    / 100.0;
                GlobalExamPolicy::Weighted {
                    semester_weight: sw,
                    global_weight: gw,
                }
            }
            GlobalExamPolicy::ReplacesWorstGrade => GlobalExamPolicy::ReplacesWorstGrade,
        }
    }

    /// Build `GlobalEligibility` from the current form fields.
    fn build_global_eligibility(&self) -> GlobalEligibility {
        GlobalEligibility {
            min_grade: parse_decimal(&self.edit_global_min_grade)
                .ok()
                .map(|g| g.clamp(MIN_GRADE, MAX_GRADE)),
        }
    }

    // =========================================================================
    // Form Handling — Category
    // =========================================================================

    pub fn start_new_category(&mut self) {
        if let Some(course) = self.current_course() {
            let remaining = course.remaining_weight();
            self.screen = Screen::EditingCategory { is_new: true };
            self.input_field = InputField::Name;
            self.edit_name.clear();
            self.edit_weight = format!("{remaining:.1}");
            // Initialize rule fields to defaults
            self.init_rule_fields_default();
            // New categories start with advanced rules collapsed and help hidden
            self.show_advanced_rules = false;
            self.show_field_help = false;
        }
    }

    pub fn start_edit_category(&mut self) {
        let Some(category) = self.current_category() else {
            return;
        };

        // Clone all needed data before mutating self
        let name = category.name.clone();
        let weight = category.weight;
        let rules = category.rules.clone();
        let has_custom_rules = !rules.is_default();

        self.screen = Screen::EditingCategory { is_new: false };
        self.input_field = InputField::Name;
        self.edit_name = name;
        self.edit_weight = format!("{weight:.1}");

        // Populate rule fields from cloned rules
        self.edit_drop_lowest = rules.drop_lowest;
        self.edit_averaging_method = rules.averaging_method;
        self.edit_min_average = rules.minimum_average.map(format_grade).unwrap_or_default();
        self.edit_on_min_not_met = rules.on_minimum_not_met;
        self.edit_min_per_eval = rules
            .minimum_per_evaluation
            .map(format_grade)
            .unwrap_or_default();
        self.edit_on_min_per_eval_not_met = rules.on_min_per_eval_not_met;
        self.edit_min_one_eval = rules.minimum_one_eval.map(format_grade).unwrap_or_default();
        self.edit_on_min_one_eval_not_met = rules.on_min_one_eval_not_met;
        self.edit_round_before_weighting = rules.round_before_weighting;
        self.edit_weighted_evaluations = rules.weighted_evaluations;

        // Auto-expand advanced rules if any non-default rules are configured
        self.show_advanced_rules = has_custom_rules;
        self.show_field_help = false;
    }

    /// Initialize category rule editing fields to their defaults.
    fn init_rule_fields_default(&mut self) {
        self.edit_drop_lowest = 0;
        self.edit_averaging_method = Default::default();
        self.edit_min_average.clear();
        self.edit_on_min_not_met = Default::default();
        self.edit_min_per_eval.clear();
        self.edit_on_min_per_eval_not_met = Default::default();
        self.edit_min_one_eval.clear();
        self.edit_on_min_one_eval_not_met = Default::default();
        self.edit_round_before_weighting = false;
        self.edit_weighted_evaluations = false;
    }

    /// Build a `CategoryRules` from the current editing state.
    fn build_rules_from_fields(&self) -> CategoryRules {
        let drop_lowest = self.edit_drop_lowest;

        let minimum_average = if self.edit_min_average.trim().is_empty() {
            None
        } else {
            parse_decimal(&self.edit_min_average)
                .ok()
                .map(|v| v.clamp(MIN_GRADE, MAX_GRADE))
        };

        let minimum_per_evaluation = if self.edit_min_per_eval.trim().is_empty() {
            None
        } else {
            parse_decimal(&self.edit_min_per_eval)
                .ok()
                .map(|v| v.clamp(MIN_GRADE, MAX_GRADE))
        };

        let minimum_one_eval = if self.edit_min_one_eval.trim().is_empty() {
            None
        } else {
            parse_decimal(&self.edit_min_one_eval)
                .ok()
                .map(|v| v.clamp(MIN_GRADE, MAX_GRADE))
        };

        CategoryRules {
            drop_lowest,
            averaging_method: self.edit_averaging_method,
            minimum_average,
            on_minimum_not_met: self.edit_on_min_not_met,
            minimum_per_evaluation,
            on_min_per_eval_not_met: self.edit_on_min_per_eval_not_met,
            minimum_one_eval,
            on_min_one_eval_not_met: self.edit_on_min_one_eval_not_met,
            round_before_weighting: self.edit_round_before_weighting,
            weighted_evaluations: self.edit_weighted_evaluations,
        }
    }

    pub fn confirm_category(&mut self) {
        let name = self.edit_name.trim().to_string();
        let weight: f64 = parse_decimal(&self.edit_weight)
            .unwrap_or(MIN_GRADE)
            .clamp(MIN_GRADE, MAX_GRADE);

        if name.is_empty() {
            return;
        }

        let rules = self.build_rules_from_fields();
        let selected_cat = self.selected_category;

        // Snapshot for undo (after validation, before mutation).
        self.push_undo();

        match self.screen {
            Screen::EditingCategory { is_new: true } => {
                if let Some(idx) = self.selected_course
                    && let Some(course) = self.courses.get_mut(idx)
                {
                    let category = Category::with_rules(name, weight, Vec::new(), rules);
                    course.categories.push(category);
                    self.selected_category = Some(course.categories.len() - 1);
                    self.selected_evaluation = None;
                }
            }
            Screen::EditingCategory { is_new: false } => {
                if let Some(course_idx) = self.selected_course
                    && let Some(cat_idx) = selected_cat
                    && let Some(course) = self.courses.get_mut(course_idx)
                    && let Some(category) = course.categories.get_mut(cat_idx)
                {
                    category.name = name;
                    category.weight = weight;
                    category.rules = rules;
                }
            }
            _ => {}
        }

        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    // =========================================================================
    // Form Handling — Evaluation
    // =========================================================================

    pub fn start_new_evaluation(&mut self) {
        if self.current_category().is_some() {
            self.screen = Screen::EditingEvaluation { is_new: true };
            self.input_field = InputField::Grade; // Start with grade field
            self.edit_name.clear();
            self.edit_grade.clear();
            self.edit_eval_weight.clear();
        }
    }

    pub fn start_edit_evaluation(&mut self) {
        let Some((name, grade, weight)) = self
            .current_evaluation()
            .map(|eval| (eval.name.clone(), eval.grade, eval.weight))
        else {
            return;
        };

        self.screen = Screen::EditingEvaluation { is_new: false };
        self.input_field = InputField::Grade;
        self.edit_name = name;
        // Pre-fill with current grade so user doesn't lose it accidentally
        self.edit_grade = grade.map(format_grade).unwrap_or_default();
        self.edit_eval_weight = weight.map(format_grade).unwrap_or_default();
    }

    pub fn confirm_evaluation(&mut self) {
        let name = self.edit_name.trim().to_string();
        let grade: Option<f64> = if self.edit_grade.trim().is_empty() {
            None
        } else {
            parse_decimal(&self.edit_grade)
                .ok()
                .map(|g: f64| g.clamp(MIN_GRADE, MAX_GRADE))
        };

        // Parse evaluation weight (only relevant for weighted-evaluations categories)
        let eval_weight: Option<f64> = if self.category_has_weighted_evals() {
            if self.edit_eval_weight.trim().is_empty() {
                Some(0.0)
            } else {
                parse_decimal(&self.edit_eval_weight)
                    .ok()
                    .map(|w: f64| w.clamp(MIN_GRADE, MAX_GRADE))
            }
        } else {
            None
        };

        if name.is_empty() {
            return;
        }

        let course_idx = self.selected_course;
        let cat_idx = self.selected_category;
        let eval_idx = self.selected_evaluation;

        // Snapshot for undo (after validation, before mutation).
        self.push_undo();

        match self.screen {
            Screen::EditingEvaluation { is_new: true } => {
                if let Some(ci) = course_idx
                    && let Some(cati) = cat_idx
                    && let Some(course) = self.courses.get_mut(ci)
                    && let Some(category) = course.categories.get_mut(cati)
                {
                    let mut eval = Evaluation::new(name);
                    eval.grade = grade;
                    eval.weight = eval_weight;
                    category.evaluations.push(eval);
                    self.selected_evaluation = Some(category.evaluations.len() - 1);
                }
            }
            Screen::EditingEvaluation { is_new: false } => {
                if let Some(ci) = course_idx
                    && let Some(cati) = cat_idx
                    && let Some(ei) = eval_idx
                    && let Some(course) = self.courses.get_mut(ci)
                    && let Some(category) = course.categories.get_mut(cati)
                    && let Some(eval) = category.evaluations.get_mut(ei)
                {
                    eval.name = name;
                    eval.grade = grade;
                    eval.weight = eval_weight;
                }
            }
            _ => {}
        }

        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    // =========================================================================
    // Deletion
    // =========================================================================

    pub fn request_delete(&mut self) {
        let can_delete = match self.focus {
            super::Focus::Courses => self.selected_course.is_some(),
            super::Focus::Categories => self.selected_category.is_some(),
            super::Focus::Evaluations => self.selected_evaluation.is_some(),
        };
        if can_delete {
            self.screen = Screen::ConfirmDelete;
        }
    }

    pub fn delete_current(&mut self) {
        // Snapshot for undo (delete is the canonical destructive action).
        self.push_undo();
        match self.focus {
            super::Focus::Courses => {
                if let Some(idx) = self.selected_course {
                    self.courses.remove(idx);
                    self.selected_course = if self.courses.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.courses.len() - 1))
                    };
                    self.reset_category_selection();
                }
            }
            super::Focus::Categories => {
                if let Some(course_idx) = self.selected_course
                    && let Some(cat_idx) = self.selected_category
                    && let Some(course) = self.courses.get_mut(course_idx)
                {
                    course.categories.remove(cat_idx);
                    self.selected_category = if course.categories.is_empty() {
                        None
                    } else {
                        Some(cat_idx.min(course.categories.len() - 1))
                    };
                    self.selected_evaluation = None;
                }
            }
            super::Focus::Evaluations => {
                if let Some(course_idx) = self.selected_course
                    && let Some(cat_idx) = self.selected_category
                    && let Some(eval_idx) = self.selected_evaluation
                    && let Some(course) = self.courses.get_mut(course_idx)
                    && let Some(category) = course.categories.get_mut(cat_idx)
                {
                    category.evaluations.remove(eval_idx);
                    self.selected_evaluation = if category.evaluations.is_empty() {
                        None
                    } else {
                        Some(eval_idx.min(category.evaluations.len() - 1))
                    };
                }
            }
        }
        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    // =========================================================================
    // Weight Management
    // =========================================================================

    /// Auto-balance weights for current course.
    pub fn auto_balance_weights(&mut self) {
        if let Some(course_idx) = self.selected_course
            && self.courses.get(course_idx).is_some()
        {
            // Snapshot for undo before mutating weights.
            self.push_undo();
            if let Some(course) = self.courses.get_mut(course_idx) {
                course.auto_balance_weights();
            }
            self.clear_status();
            self.persist();
        }
    }
}
