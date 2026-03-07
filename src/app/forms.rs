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

//! Form handling methods for the application.
//!
//! This module contains all the methods that handle form input, confirmation,
//! and cancellation for courses, categories, evaluations, templates, and
//! language selection. These are extracted from `App` as an `impl` block
//! extension to keep file sizes manageable.

use crate::i18n::Language;
use crate::model::{
    Category, CategoryRules, Course, DEFAULT_PASSING_GRADE, Evaluation, GlobalEligibility,
    GlobalExamPolicy, MAX_GRADE, MIN_GRADE,
};
use crate::persistence;

use super::{App, InputField, Screen};

impl App {
    // =========================================================================
    // Form Handling — Course
    // =========================================================================

    pub fn start_new_course(&mut self) {
        self.screen = Screen::SelectingTemplate;
        self.selected_template = 0;
    }

    pub fn confirm_template_selection(&mut self) {
        self.screen = Screen::EditingCourse { is_new: true };
        self.input_field = InputField::Name;
        self.edit_name.clear();
        self.edit_passing_grade = format!("{DEFAULT_PASSING_GRADE:.0}");
        self.init_global_fields_default();
        self.show_field_help = false;
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
        self.edit_passing_grade = format!("{pg:.0}");

        // Load global exam fields from the course
        self.edit_global_policy = course.global_policy.clone();
        match &course.global_policy {
            GlobalExamPolicy::Weighted {
                semester_weight,
                global_weight,
            } => {
                self.edit_global_semester_weight = format!("{:.0}", semester_weight * 100.0);
                self.edit_global_exam_weight = format!("{:.0}", global_weight * 100.0);
            }
            _ => {
                self.edit_global_semester_weight = "70".to_string();
                self.edit_global_exam_weight = "30".to_string();
            }
        }
        self.edit_global_min_grade = course
            .global_eligibility
            .min_grade
            .map(|g| format!("{g:.0}"))
            .unwrap_or_default();
        self.edit_global_max_grade = course
            .global_eligibility
            .max_grade
            .map(|g| format!("{g:.0}"))
            .unwrap_or_default();
    }

    pub fn confirm_course(&mut self) {
        let name = self.edit_name.trim().to_string();
        let passing_grade: f64 = self
            .edit_passing_grade
            .parse::<f64>()
            .unwrap_or(DEFAULT_PASSING_GRADE)
            .clamp(MIN_GRADE, MAX_GRADE);

        if name.is_empty() {
            return;
        }

        // Build global policy from form fields
        let global_policy = self.build_global_policy();
        let global_eligibility = self.build_global_eligibility();

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
        self.edit_global_max_grade.clear();
    }

    /// Build a `GlobalExamPolicy` from the current form fields.
    fn build_global_policy(&self) -> GlobalExamPolicy {
        match &self.edit_global_policy {
            GlobalExamPolicy::None => GlobalExamPolicy::None,
            GlobalExamPolicy::Weighted { .. } => {
                let sw = self
                    .edit_global_semester_weight
                    .parse::<f64>()
                    .unwrap_or(70.0)
                    .clamp(0.0, 100.0)
                    / 100.0;
                let gw = self
                    .edit_global_exam_weight
                    .parse::<f64>()
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
            min_grade: self
                .edit_global_min_grade
                .trim()
                .parse::<f64>()
                .ok()
                .map(|g| g.clamp(MIN_GRADE, MAX_GRADE)),
            max_grade: self
                .edit_global_max_grade
                .trim()
                .parse::<f64>()
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
        self.edit_min_average = rules
            .minimum_average
            .map(|v| format!("{v:.0}"))
            .unwrap_or_default();
        self.edit_on_min_not_met = rules.on_minimum_not_met;
        self.edit_min_per_eval = rules
            .minimum_per_evaluation
            .map(|v| format!("{v:.0}"))
            .unwrap_or_default();
        self.edit_round_before_weighting = rules.round_before_weighting;

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
        self.edit_round_before_weighting = false;
    }

    /// Build a `CategoryRules` from the current editing state.
    fn build_rules_from_fields(&self) -> CategoryRules {
        let drop_lowest = self.edit_drop_lowest;

        let minimum_average = if self.edit_min_average.trim().is_empty() {
            None
        } else {
            self.edit_min_average
                .trim()
                .parse::<f64>()
                .ok()
                .map(|v| v.clamp(MIN_GRADE, MAX_GRADE))
        };

        let minimum_per_evaluation = if self.edit_min_per_eval.trim().is_empty() {
            None
        } else {
            self.edit_min_per_eval
                .trim()
                .parse::<f64>()
                .ok()
                .map(|v| v.clamp(MIN_GRADE, MAX_GRADE))
        };

        CategoryRules {
            drop_lowest,
            averaging_method: self.edit_averaging_method,
            minimum_average,
            on_minimum_not_met: self.edit_on_min_not_met,
            minimum_per_evaluation,
            round_before_weighting: self.edit_round_before_weighting,
        }
    }

    pub fn confirm_category(&mut self) {
        let name = self.edit_name.trim().to_string();
        let weight: f64 = self
            .edit_weight
            .parse::<f64>()
            .unwrap_or(MIN_GRADE)
            .clamp(MIN_GRADE, MAX_GRADE);

        if name.is_empty() {
            return;
        }

        let rules = self.build_rules_from_fields();
        let selected_cat = self.selected_category;

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
        }
    }

    pub fn start_edit_evaluation(&mut self) {
        let Some((name, grade)) = self
            .current_evaluation()
            .map(|eval| (eval.name.clone(), eval.grade))
        else {
            return;
        };

        self.screen = Screen::EditingEvaluation { is_new: false };
        self.input_field = InputField::Grade;
        self.edit_name = name;
        // Pre-fill with current grade so user doesn't lose it accidentally
        self.edit_grade = grade.map(|g| format!("{g:.0}")).unwrap_or_default();
    }

    pub fn confirm_evaluation(&mut self) {
        let name = self.edit_name.trim().to_string();
        let grade: Option<f64> = if self.edit_grade.trim().is_empty() {
            None
        } else {
            self.edit_grade
                .parse::<f64>()
                .ok()
                .map(|g: f64| g.clamp(MIN_GRADE, MAX_GRADE))
        };

        if name.is_empty() {
            return;
        }

        let course_idx = self.selected_course;
        let cat_idx = self.selected_category;
        let eval_idx = self.selected_evaluation;

        match self.screen {
            Screen::EditingEvaluation { is_new: true } => {
                if let Some(ci) = course_idx
                    && let Some(cati) = cat_idx
                    && let Some(course) = self.courses.get_mut(ci)
                    && let Some(category) = course.categories.get_mut(cati)
                {
                    let mut eval = Evaluation::new(name);
                    eval.grade = grade;
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
            && let Some(course) = self.courses.get_mut(course_idx)
        {
            course.auto_balance_weights();
            self.clear_status();
            self.persist();
        }
    }

    // =========================================================================
    // Save as Template
    // =========================================================================

    /// Start the "save as template" flow for current course.
    pub fn start_save_as_template(&mut self) {
        let Some(course) = self.current_course() else {
            return;
        };

        let m = self.messages();
        let name = format!("{} {}", course.name, m.template_suffix);
        let description = course.generate_template_description();
        let description = if description.is_empty() {
            m.empty_template_desc.to_string()
        } else {
            description
        };

        self.edit_name = name;
        self.edit_description = description;
        self.input_field = InputField::Name;
        self.screen = Screen::SavingTemplate;
    }

    /// Confirm saving the current course as a user template.
    pub fn confirm_save_template(&mut self) {
        let name = self.edit_name.trim().to_string();
        let description = self.edit_description.trim().to_string();

        if name.is_empty() {
            return;
        }

        let Some(course) = self.current_course() else {
            self.screen = Screen::Main;
            return;
        };

        let template = course.to_template(name, description);
        self.user_templates.push(template);

        // Save user templates to disk
        if persistence::save_user_templates(&self.user_templates).is_err() {
            let msg = self.messages().save_template_error.to_string();
            self.set_status(msg);
        }

        self.screen = Screen::Main;
    }

    /// Check if current selected template is a user template (can be deleted).
    pub fn is_user_template_selected(&self) -> bool {
        self.selected_template >= self.built_in_templates.len()
    }

    /// Request deletion of current user template.
    pub fn request_delete_template(&mut self) {
        if self.is_user_template_selected() {
            self.screen = Screen::ConfirmDeleteTemplate;
        }
    }

    /// Delete the currently selected user template.
    pub fn delete_current_template(&mut self) {
        if !self.is_user_template_selected() {
            self.screen = Screen::SelectingTemplate;
            return;
        }

        let user_template_idx = self.selected_template - self.built_in_templates.len();
        self.user_templates.remove(user_template_idx);

        // Adjust selection
        let total = self.templates_count();
        if total == 0 {
            self.selected_template = 0;
        } else if self.selected_template >= total {
            self.selected_template = total - 1;
        }

        // Save updated user templates
        if persistence::save_user_templates(&self.user_templates).is_err() {
            let msg = self.messages().save_template_error.to_string();
            self.set_status(msg);
        }

        self.screen = Screen::SelectingTemplate;
    }

    // =========================================================================
    // Language Selection
    // =========================================================================

    /// Show the language selection popup.
    pub fn show_language_popup(&mut self) {
        // Find current language index
        self.selected_language = Language::all()
            .iter()
            .position(|&l| l == self.language)
            .unwrap_or(0);
        self.screen = Screen::SelectingLanguage;
    }

    /// Move to next language in the list.
    pub fn next_language(&mut self) {
        let count = Language::all().len();
        if count > 0 {
            self.selected_language = (self.selected_language + 1) % count;
        }
    }

    /// Move to previous language in the list.
    pub fn previous_language(&mut self) {
        let count = Language::all().len();
        if count > 0 {
            self.selected_language = if self.selected_language == 0 {
                count - 1
            } else {
                self.selected_language - 1
            };
        }
    }

    /// Confirm language selection and save to config.
    pub fn confirm_language_selection(&mut self) {
        if let Some(&new_lang) = Language::all().get(self.selected_language) {
            self.language = new_lang;

            // Regenerate built-in templates for new language
            self.built_in_templates = crate::templates::built_in_templates(new_lang);

            // Save config
            let config = persistence::Config {
                language: new_lang,
                use_nerd_fonts: self.use_nerd_fonts,
                compact_courses: self.compact_courses,
            };
            if persistence::save_config(&config).is_err() {
                let msg = self.messages().config_save_error.to_string();
                self.set_status(msg);
            }
        }

        self.screen = Screen::Main;
    }

    /// Cancel language selection.
    pub fn cancel_language_selection(&mut self) {
        self.screen = Screen::Main;
    }

    // =========================================================================
    // Settings
    // =========================================================================

    /// Number of settings entries in the settings popup.
    const SETTINGS_COUNT: usize = 3;

    /// Show the settings popup.
    pub fn show_settings(&mut self) {
        self.selected_setting = 0;
        self.screen = Screen::Settings;
    }

    /// Move to the next setting in the list.
    pub fn next_setting(&mut self) {
        self.selected_setting = (self.selected_setting + 1) % Self::SETTINGS_COUNT;
    }

    /// Move to the previous setting in the list.
    pub fn previous_setting(&mut self) {
        self.selected_setting = if self.selected_setting == 0 {
            Self::SETTINGS_COUNT - 1
        } else {
            self.selected_setting - 1
        };
    }

    /// Toggle the currently selected setting value.
    pub fn toggle_current_setting(&mut self) {
        match self.selected_setting {
            // 0: Nerd Fonts
            0 => self.use_nerd_fonts = !self.use_nerd_fonts,
            // 1: Language — cycle forward through available languages
            1 => {
                let langs = Language::all();
                let cur = langs.iter().position(|&l| l == self.language).unwrap_or(0);
                let next = (cur + 1) % langs.len();
                self.language = langs[next];
                self.built_in_templates = crate::templates::built_in_templates(self.language);
            }
            // 2: Compact courses
            2 => self.compact_courses = !self.compact_courses,
            _ => {}
        }
    }

    /// Toggle the currently selected setting in reverse direction.
    pub fn toggle_current_setting_reverse(&mut self) {
        match self.selected_setting {
            0 => self.use_nerd_fonts = !self.use_nerd_fonts,
            1 => {
                let langs = Language::all();
                let cur = langs.iter().position(|&l| l == self.language).unwrap_or(0);
                let prev = if cur == 0 { langs.len() - 1 } else { cur - 1 };
                self.language = langs[prev];
                self.built_in_templates = crate::templates::built_in_templates(self.language);
            }
            2 => self.compact_courses = !self.compact_courses,
            _ => {}
        }
    }

    /// Confirm and persist settings, then return to the main screen.
    pub fn confirm_settings(&mut self) {
        let config = persistence::Config {
            language: self.language,
            use_nerd_fonts: self.use_nerd_fonts,
            compact_courses: self.compact_courses,
        };
        if persistence::save_config(&config).is_err() {
            let msg = self.messages().config_save_error.to_string();
            self.set_status(msg);
        }
        self.screen = Screen::Main;
    }

    /// Cancel settings without saving — restore persisted values.
    pub fn cancel_settings(&mut self) {
        let (config, _) = persistence::load_config();
        self.use_nerd_fonts = config.use_nerd_fonts;
        self.compact_courses = config.compact_courses;
        if self.language != config.language {
            self.language = config.language;
            self.built_in_templates = crate::templates::built_in_templates(self.language);
        }
        self.screen = Screen::Main;
    }

    // =========================================================================
    // Compact Courses Toggle
    // =========================================================================

    /// Toggle compact courses view and persist immediately.
    pub fn toggle_compact_courses(&mut self) {
        self.compact_courses = !self.compact_courses;
        let config = persistence::Config {
            language: self.language,
            use_nerd_fonts: self.use_nerd_fonts,
            compact_courses: self.compact_courses,
        };
        if let Err(e) = persistence::save_config(&config) {
            let m = self.messages();
            self.set_status(format!("{}: {}", m.config_save_error, e));
        }
    }

    // =========================================================================
    // Form Handling — Global Grade Entry
    // =========================================================================

    /// Open the global grade entry popup for the current course.
    pub fn start_global_grade_entry(&mut self) {
        let Some(idx) = self.selected_course else {
            return;
        };
        let Some(course) = self.courses.get(idx) else {
            return;
        };

        if course.global_policy == GlobalExamPolicy::None {
            let m = self.messages();
            self.set_status(m.global_no_policy.to_string());
            return;
        }

        self.edit_global_grade = course
            .global_exam_grade
            .map(|g| format!("{g:.0}"))
            .unwrap_or_default();
        self.screen = Screen::EnteringGlobalGrade;
        self.input_field = InputField::Grade;
    }

    /// Confirm the global grade entry and save to the current course.
    pub fn confirm_global_grade(&mut self) {
        let Some(idx) = self.selected_course else {
            self.screen = Screen::Main;
            return;
        };
        let Some(course) = self.courses.get_mut(idx) else {
            self.screen = Screen::Main;
            return;
        };

        let trimmed = self.edit_global_grade.trim();
        if trimmed.is_empty() {
            // Empty input clears the global grade
            course.global_exam_grade = None;
        } else if let Ok(grade) = trimmed.parse::<f64>() {
            course.global_exam_grade = Some(grade.clamp(MIN_GRADE, MAX_GRADE));
        }

        // Auto-detect target category for ReplacesWorstGrade if not set
        if course.global_policy == GlobalExamPolicy::ReplacesWorstGrade
            && course.global_target_category.is_none()
        {
            // resolve_global_target_category will auto-detect; persist the result
            // so it's stable across recalculations.
            let result = course.compute_grade();
            if result.needs_global {
                // Find the first RequiresGlobal category that fails
                for (i, fm) in result.failed_minimums.iter().enumerate() {
                    if fm.action == crate::model::MinimumNotMetAction::RequiresGlobal {
                        course.global_target_category = Some(fm.category_idx);
                        break;
                    }
                    // Fallback: use first failure
                    if i == 0 {
                        course.global_target_category = Some(fm.category_idx);
                    }
                }
            }
        }

        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    /// Cancel global grade entry without saving.
    pub fn cancel_global_grade(&mut self) {
        self.screen = Screen::Main;
    }
}
