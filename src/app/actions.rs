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

//! Secondary actions for the application.
//!
//! This module contains methods for template management, language selection,
//! settings, global grade entry, evaluation yank/paste, and bulk-add
//! evaluations. Extracted from `forms.rs` to keep file sizes manageable.

use crate::i18n::Language;
use crate::model::{Evaluation, GlobalExamPolicy, MAX_GRADE, MIN_GRADE};
use crate::persistence;

use super::{App, InputField, Screen};

/// Parse a decimal string that may use comma as the decimal separator.
fn parse_decimal(s: &str) -> Result<f64, std::num::ParseFloatError> {
    s.trim().replace(',', ".").parse::<f64>()
}

/// Format a grade for display in form fields.
fn format_grade(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

impl App {
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
            self.set_error(msg);
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
            self.set_error(msg);
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
                self.set_error(msg);
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
            self.set_error(msg);
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
            self.set_error(format!("{}: {}", m.config_save_error, e));
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
            self.set_warning(m.global_no_policy.to_string());
            return;
        }

        self.edit_global_grade = course
            .global_exam_grade
            .map(format_grade)
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
        if self.courses.get(idx).is_none() {
            self.screen = Screen::Main;
            return;
        }

        let trimmed = self.edit_global_grade.trim().to_string();
        // Snapshot for undo (before applying the new global grade).
        self.push_undo();
        let Some(course) = self.courses.get_mut(idx) else {
            self.screen = Screen::Main;
            return;
        };
        if trimmed.is_empty() {
            // Empty input clears the global grade
            course.global_exam_grade = None;
        } else if let Ok(grade) = parse_decimal(&trimmed) {
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

    // =========================================================================
    // Yank / Paste Evaluations
    // =========================================================================

    /// Copy the currently selected evaluation to the clipboard.
    pub fn yank_evaluation(&mut self) {
        let m = self.messages();
        let Some(eval) = self.current_evaluation() else {
            self.set_warning(m.no_eval_to_yank.to_string());
            return;
        };
        let name = eval.name.clone();
        let grade = eval.grade;
        let weight = eval.weight;
        self.clipboard_evaluation = Some((name.clone(), grade, weight));
        self.set_status(format!("{}: {}", m.yanked_eval, name));
    }

    /// Paste the clipboard evaluation into the current category.
    pub fn paste_evaluation(&mut self) {
        let Some((name, grade, weight)) = self.clipboard_evaluation.clone() else {
            let msg = self.messages().no_eval_in_clipboard.to_string();
            self.set_warning(msg);
            return;
        };
        // Snapshot for undo (paste appends a new evaluation).
        self.push_undo();
        if let Some(ci) = self.selected_course
            && let Some(cati) = self.selected_category
            && let Some(course) = self.courses.get_mut(ci)
            && let Some(category) = course.categories.get_mut(cati)
        {
            let mut eval = Evaluation::new(name.clone());
            eval.grade = grade;
            // Preserve evaluation weight only if the target category uses weighted evaluations
            if category.rules.weighted_evaluations {
                eval.weight = weight;
            }
            category.evaluations.push(eval);
            self.selected_evaluation = Some(category.evaluations.len() - 1);
            let pasted_label = self.messages().pasted_eval;
            self.set_status(format!("{}: {}", pasted_label, name));
            self.persist();
        }
    }

    // =========================================================================
    // Bulk-Add Evaluations
    // =========================================================================

    /// Open the bulk-add evaluations popup.
    pub fn start_bulk_add_evaluations(&mut self) {
        if let Some(cat) = self.current_category() {
            self.edit_name = cat.name.clone();
            self.edit_bulk_count.clear();
            self.input_field = InputField::Name;
            self.screen = Screen::BulkAddEvaluations;
        }
    }

    /// Confirm bulk-add: create N evaluations named "{CategoryName} 1..N".
    pub fn confirm_bulk_add(&mut self) {
        let count: usize = match self.edit_bulk_count.trim().parse() {
            Ok(n) if n > 0 => n,
            _ => {
                self.screen = Screen::Main;
                return;
            }
        };

        let base_name = self.edit_name.trim().to_string();
        if base_name.is_empty() {
            self.screen = Screen::Main;
            return;
        }

        // Snapshot for undo (bulk-add can create many evaluations at once).
        self.push_undo();
        let mut inserted = false;
        if let Some(ci) = self.selected_course
            && let Some(cati) = self.selected_category
            && let Some(course) = self.courses.get_mut(ci)
            && let Some(category) = course.categories.get_mut(cati)
        {
            for i in 1..=count {
                let eval = Evaluation::new(format!("{base_name} {i}"));
                category.evaluations.push(eval);
            }
            self.selected_evaluation = Some(category.evaluations.len() - 1);
            inserted = true;
        }

        if inserted {
            let label = self.messages().bulk_added_evals;
            self.set_status(format!("{count} {label}"));
            self.persist();
        } else {
            // Snapshot was taken but nothing changed — discard it.
            self.undo_stack.pop();
        }

        self.screen = Screen::Main;
    }

    /// Cancel bulk-add without creating anything.
    pub fn cancel_bulk_add(&mut self) {
        self.screen = Screen::Main;
    }
}
