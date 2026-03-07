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

//! Application state and logic.
//!
//! This module contains the main `App` struct that holds all application state,
//! as well as methods for navigation, getters, and state management.
//! Form handling (course/category/evaluation editing, templates, language)
//! lives in the `forms` submodule.

mod forms;

use crate::i18n::{Language, Messages};
use crate::model::{
    AveragingMethod, Category, Course, CourseTemplate, Evaluation, MinimumNotMetAction,
};
use crate::persistence;
use crate::templates;

// =============================================================================
// Enums
// =============================================================================

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Courses,
    Categories,
    Evaluations,
}

/// Current screen/mode of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Main,
    SelectingTemplate,
    EditingCourse { is_new: bool },
    EditingCategory { is_new: bool },
    EditingEvaluation { is_new: bool },
    ConfirmDelete,
    ConfirmDeleteTemplate,
    SavingTemplate,
    SelectingLanguage,
}

/// Input field being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Name,
    PassingGrade,
    Weight,
    Grade,
    Description,
    // Category rule fields
    DropLowest,
    MinimumAverage,
    MinPerEval,
    // Category rule toggle fields (cycled with Space/Enter, not typed)
    AvgMethod,
    OnMinNotMet,
    RoundBeforeWeight,
}

impl InputField {
    /// Returns true if this field is a toggle (cycled, not typed).
    pub fn is_toggle(&self) -> bool {
        matches!(
            self,
            InputField::AvgMethod | InputField::OnMinNotMet | InputField::RoundBeforeWeight
        )
    }
}

// =============================================================================
// App Struct
// =============================================================================

/// Main application state.
#[derive(Debug)]
pub struct App {
    pub courses: Vec<Course>,
    pub built_in_templates: Vec<CourseTemplate>,
    pub user_templates: Vec<CourseTemplate>,

    // Selection state
    pub selected_course: Option<usize>,
    pub selected_category: Option<usize>,
    pub selected_evaluation: Option<usize>,
    pub selected_template: usize,
    pub selected_language: usize,

    // UI state
    pub focus: Focus,
    pub screen: Screen,
    pub should_quit: bool,

    /// Temporary status message shown to the user (errors, confirmations, etc.)
    /// Cleared on the next action.
    pub status_message: Option<String>,

    // Language
    pub language: Language,

    // Input state for forms
    pub input_field: InputField,
    pub edit_name: String,
    pub edit_passing_grade: String,
    pub edit_weight: String,
    pub edit_grade: String,
    pub edit_description: String,

    // Category rule editing state
    pub edit_drop_lowest: String,
    pub edit_min_average: String,
    pub edit_min_per_eval: String,
    pub edit_averaging_method: AveragingMethod,
    pub edit_on_min_not_met: MinimumNotMetAction,
    pub edit_round_before_weighting: bool,
}

impl Default for App {
    fn default() -> Self {
        let language = Language::default();
        Self {
            courses: Vec::new(),
            built_in_templates: templates::built_in_templates(language),
            user_templates: Vec::new(),
            selected_course: None,
            selected_category: None,
            selected_evaluation: None,
            selected_template: 0,
            selected_language: 0,
            focus: Focus::Courses,
            screen: Screen::Main,
            should_quit: false,
            status_message: None,
            language,
            input_field: InputField::Name,
            edit_name: String::new(),
            edit_passing_grade: String::new(),
            edit_weight: String::new(),
            edit_grade: String::new(),
            edit_description: String::new(),
            edit_drop_lowest: String::new(),
            edit_min_average: String::new(),
            edit_min_per_eval: String::new(),
            edit_averaging_method: AveragingMethod::default(),
            edit_on_min_not_met: MinimumNotMetAction::default(),
            edit_round_before_weighting: false,
        }
    }
}

// =============================================================================
// Core Methods
// =============================================================================

impl App {
    /// Load application state from disk, or create empty state if file doesn't exist.
    pub fn load() -> Self {
        // Load config first to get language
        let config = persistence::load_config();
        let language = config.language;
        let m = language.messages();
        let mut status_message: Option<String> = None;

        let courses = match persistence::load_data() {
            Ok(courses) => courses,
            Err(_) => {
                status_message = Some(m.load_error.to_string());
                Vec::new()
            }
        };

        let user_templates = match persistence::load_user_templates() {
            Ok(templates) => templates,
            Err(_) => {
                status_message = Some(m.load_template_error.to_string());
                Vec::new()
            }
        };

        // Generate built-in templates for current language
        let built_in_templates = templates::built_in_templates(language);

        let selected_course = if courses.is_empty() { None } else { Some(0) };

        // If we have a course selected, also select first category if exists
        let selected_category = if let Some(idx) = selected_course {
            if let Some(course) = courses.get(idx) {
                if course.categories.is_empty() {
                    None
                } else {
                    Some(0)
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            courses,
            built_in_templates,
            user_templates,
            selected_course,
            selected_category,
            status_message,
            language,
            ..Default::default()
        }
    }

    /// Get the current messages for the selected language.
    pub fn messages(&self) -> &'static Messages {
        self.language.messages()
    }

    /// Save current state to disk.
    pub fn save(&self) -> color_eyre::Result<()> {
        persistence::save_data(&self.courses)
    }

    /// Clear the status message (called before each user action).
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Set a status message visible to the user.
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some(msg);
    }

    /// Persist state to disk. Shows error to user via status message on failure.
    pub(crate) fn persist(&mut self) {
        if self.save().is_err() {
            let msg = self.messages().save_error.to_string();
            self.set_status(msg);
        }
    }

    // =========================================================================
    // Getters
    // =========================================================================

    /// Get the currently selected Course, if any.
    pub fn current_course(&self) -> Option<&Course> {
        self.selected_course.and_then(|i| self.courses.get(i))
    }

    /// Get the currently selected Category, if any.
    pub fn current_category(&self) -> Option<&Category> {
        self.current_course()
            .and_then(|c| self.selected_category.and_then(|i| c.categories.get(i)))
    }

    /// Get the currently selected Evaluation, if any.
    pub fn current_evaluation(&self) -> Option<&Evaluation> {
        self.current_category()
            .and_then(|c| self.selected_evaluation.and_then(|i| c.evaluations.get(i)))
    }

    /// Get all templates combined: built-in first, then user templates.
    pub fn all_templates(&self) -> Vec<&CourseTemplate> {
        self.built_in_templates
            .iter()
            .chain(self.user_templates.iter())
            .collect()
    }

    /// Get the currently selected template.
    pub fn current_template(&self) -> Option<&CourseTemplate> {
        self.all_templates().get(self.selected_template).copied()
    }

    /// Get the total number of templates.
    pub fn templates_count(&self) -> usize {
        self.built_in_templates.len() + self.user_templates.len()
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    pub fn next_course(&mut self) {
        if self.courses.is_empty() {
            self.selected_course = None;
            return;
        }
        self.selected_course = Some(match self.selected_course {
            Some(i) => (i + 1).min(self.courses.len() - 1),
            None => 0,
        });
        // Auto-select first category when changing course
        self.selected_category = self.current_course().and_then(|c| {
            if c.categories.is_empty() {
                None
            } else {
                Some(0)
            }
        });
        self.selected_evaluation = None;
    }

    pub fn previous_course(&mut self) {
        if self.courses.is_empty() {
            self.selected_course = None;
            return;
        }
        self.selected_course = Some(match self.selected_course {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
        // Auto-select first category when changing course
        self.selected_category = self.current_course().and_then(|c| {
            if c.categories.is_empty() {
                None
            } else {
                Some(0)
            }
        });
        self.selected_evaluation = None;
    }

    pub fn next_category(&mut self) {
        let Some(course) = self.current_course() else {
            return;
        };
        if course.categories.is_empty() {
            self.selected_category = None;
            return;
        }
        let len = course.categories.len();
        self.selected_category = Some(match self.selected_category {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        });
        // Auto-select first evaluation when changing category
        self.selected_evaluation = self.current_category().and_then(|c| {
            if c.evaluations.is_empty() {
                None
            } else {
                Some(0)
            }
        });
    }

    pub fn previous_category(&mut self) {
        let Some(course) = self.current_course() else {
            return;
        };
        if course.categories.is_empty() {
            self.selected_category = None;
            return;
        }
        self.selected_category = Some(match self.selected_category {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
        // Auto-select first evaluation when changing category
        self.selected_evaluation = self.current_category().and_then(|c| {
            if c.evaluations.is_empty() {
                None
            } else {
                Some(0)
            }
        });
    }

    pub fn next_evaluation(&mut self) {
        let Some(category) = self.current_category() else {
            return;
        };
        if category.evaluations.is_empty() {
            self.selected_evaluation = None;
            return;
        }
        let len = category.evaluations.len();
        self.selected_evaluation = Some(match self.selected_evaluation {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        });
    }

    pub fn previous_evaluation(&mut self) {
        let Some(category) = self.current_category() else {
            return;
        };
        if category.evaluations.is_empty() {
            self.selected_evaluation = None;
            return;
        }
        self.selected_evaluation = Some(match self.selected_evaluation {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
    }

    pub fn next_template(&mut self) {
        let count = self.templates_count();
        if count > 0 {
            self.selected_template = (self.selected_template + 1) % count;
        }
    }

    pub fn previous_template(&mut self) {
        let count = self.templates_count();
        if count > 0 {
            self.selected_template = if self.selected_template == 0 {
                count - 1
            } else {
                self.selected_template - 1
            };
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Courses => Focus::Categories,
            Focus::Categories => Focus::Evaluations,
            Focus::Evaluations => Focus::Courses,
        };
    }

    pub fn focus_left(&mut self) {
        self.focus = match self.focus {
            Focus::Courses | Focus::Categories => Focus::Courses,
            Focus::Evaluations => Focus::Categories,
        };
    }

    pub fn focus_right(&mut self) {
        self.focus = match self.focus {
            Focus::Courses => Focus::Categories,
            Focus::Categories | Focus::Evaluations => Focus::Evaluations,
        };
    }

    // =========================================================================
    // Input Field Navigation
    // =========================================================================

    pub fn next_input_field(&mut self) {
        self.input_field = match (&self.screen, &self.input_field) {
            (Screen::EditingCourse { .. }, InputField::Name) => InputField::PassingGrade,
            (Screen::EditingCourse { .. }, InputField::PassingGrade) => InputField::Name,
            // Category: Name → Weight → DropLowest → AvgMethod → MinAverage → (OnMinNotMet) → MinPerEval → RoundBeforeWeight → Name
            (Screen::EditingCategory { .. }, InputField::Name) => InputField::Weight,
            (Screen::EditingCategory { .. }, InputField::Weight) => InputField::DropLowest,
            (Screen::EditingCategory { .. }, InputField::DropLowest) => InputField::AvgMethod,
            (Screen::EditingCategory { .. }, InputField::AvgMethod) => InputField::MinimumAverage,
            (Screen::EditingCategory { .. }, InputField::MinimumAverage) => {
                // Only show OnMinNotMet if a minimum average is set
                if !self.edit_min_average.trim().is_empty() {
                    InputField::OnMinNotMet
                } else {
                    InputField::MinPerEval
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinNotMet) => InputField::MinPerEval,
            (Screen::EditingCategory { .. }, InputField::MinPerEval) => {
                InputField::RoundBeforeWeight
            }
            (Screen::EditingCategory { .. }, InputField::RoundBeforeWeight) => InputField::Name,
            (Screen::EditingEvaluation { .. }, InputField::Grade) => InputField::Name,
            (Screen::EditingEvaluation { .. }, InputField::Name) => InputField::Grade,
            (Screen::SavingTemplate, InputField::Name) => InputField::Description,
            (Screen::SavingTemplate, InputField::Description) => InputField::Name,
            _ => self.input_field,
        };
    }

    pub fn current_input_buffer(&mut self) -> &mut String {
        match self.input_field {
            InputField::Name => &mut self.edit_name,
            InputField::PassingGrade => &mut self.edit_passing_grade,
            InputField::Weight => &mut self.edit_weight,
            InputField::Grade => &mut self.edit_grade,
            InputField::Description => &mut self.edit_description,
            InputField::DropLowest => &mut self.edit_drop_lowest,
            InputField::MinimumAverage => &mut self.edit_min_average,
            InputField::MinPerEval => &mut self.edit_min_per_eval,
            // Toggle fields don't have text buffers — return a dummy
            // (these are cycled, not typed into)
            InputField::AvgMethod | InputField::OnMinNotMet | InputField::RoundBeforeWeight => {
                &mut self.edit_name // unreachable in practice: toggles use cycle_toggle_field()
            }
        }
    }

    /// Cycle the current toggle field to its next value.
    pub fn cycle_toggle_field(&mut self) {
        match self.input_field {
            InputField::AvgMethod => {
                self.edit_averaging_method = match self.edit_averaging_method {
                    AveragingMethod::Arithmetic => AveragingMethod::Geometric,
                    AveragingMethod::Geometric => AveragingMethod::Arithmetic,
                };
            }
            InputField::OnMinNotMet => {
                self.edit_on_min_not_met = match self.edit_on_min_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::RoundBeforeWeight => {
                self.edit_round_before_weighting = !self.edit_round_before_weighting;
            }
            _ => {}
        }
    }

    pub fn cancel_edit(&mut self) {
        self.screen = Screen::Main;
    }
}
