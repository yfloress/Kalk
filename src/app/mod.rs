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
//! Form handling (course/category/evaluation editing) lives in `forms`.
//! Secondary actions (templates, language, settings, global grade, yank/paste,
//! bulk-add) live in `actions`.

mod actions;
mod forms;

use crate::i18n::{Language, Messages};
use crate::model::{
    AveragingMethod, Category, Course, CourseTemplate, Evaluation, GlobalExamPolicy,
    MinimumNotMetAction, NeededGradeStatus,
};
use crate::persistence;
use crate::templates;

/// Maximum value for the "drop lowest" toggle cycle (0..=MAX_DROP_LOWEST).
const MAX_DROP_LOWEST: usize = 5;

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
    Settings,
    BulkAddEvaluations,
    EnteringGlobalGrade,
}

/// Input field being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Name,
    PassingGrade,
    Weight,
    Grade,
    Description,
    // Category rule fields (text input)
    MinimumAverage,
    MinPerEval,
    MinOneEval,
    // Category rule toggle fields (cycled with Space/Enter, not typed)
    DropLowest,
    AvgMethod,
    OnMinNotMet,
    OnMinPerEvalNotMet,
    OnMinOneEvalNotMet,
    RoundBeforeWeight,
    // Global exam fields (course form)
    GlobalPolicy,
    GlobalSemesterWeight,
    GlobalExamWeight,
    GlobalMinGrade,
}

impl InputField {
    /// Returns true if this field is a toggle (cycled, not typed).
    pub fn is_toggle(&self) -> bool {
        matches!(
            self,
            InputField::DropLowest
                | InputField::AvgMethod
                | InputField::OnMinNotMet
                | InputField::OnMinPerEvalNotMet
                | InputField::OnMinOneEvalNotMet
                | InputField::RoundBeforeWeight
                | InputField::GlobalPolicy
        )
    }

    /// Returns true if this field only accepts numeric input (digits, dot, comma).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            InputField::PassingGrade
                | InputField::Weight
                | InputField::Grade
                | InputField::MinimumAverage
                | InputField::MinPerEval
                | InputField::MinOneEval
                | InputField::GlobalSemesterWeight
                | InputField::GlobalExamWeight
                | InputField::GlobalMinGrade
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

    /// Clipboard for yanked evaluation (name, grade).
    pub clipboard_evaluation: Option<(String, Option<f64>)>,

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
    pub edit_drop_lowest: usize,
    pub edit_min_average: String,
    pub edit_min_per_eval: String,
    pub edit_min_one_eval: String,
    pub edit_averaging_method: AveragingMethod,
    pub edit_on_min_not_met: MinimumNotMetAction,
    pub edit_on_min_per_eval_not_met: MinimumNotMetAction,
    pub edit_on_min_one_eval_not_met: MinimumNotMetAction,
    pub edit_round_before_weighting: bool,

    /// Whether advanced rules section is expanded in the category popup.
    pub show_advanced_rules: bool,
    /// Whether the field help panel is visible in the category/course popup.
    pub show_field_help: bool,

    // Global exam editing state
    pub edit_global_policy: GlobalExamPolicy,
    pub edit_global_semester_weight: String,
    pub edit_global_exam_weight: String,
    pub edit_global_min_grade: String,
    /// Temporary buffer for the global grade entry popup.
    pub edit_global_grade: String,

    /// Temporary buffer for the bulk-add evaluation count popup.
    pub edit_bulk_count: String,

    /// Whether to use Nerd Font icons (persisted in config).
    pub use_nerd_fonts: bool,
    /// Whether to show courses in compact mode (single line per course).
    pub compact_courses: bool,
    /// Index of the focused setting in the settings popup.
    pub selected_setting: usize,
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
            clipboard_evaluation: None,
            status_message: None,
            language,
            input_field: InputField::Name,
            edit_name: String::new(),
            edit_passing_grade: String::new(),
            edit_weight: String::new(),
            edit_grade: String::new(),
            edit_description: String::new(),
            edit_drop_lowest: 0,
            edit_min_average: String::new(),
            edit_min_per_eval: String::new(),
            edit_min_one_eval: String::new(),
            edit_averaging_method: AveragingMethod::default(),
            edit_on_min_not_met: MinimumNotMetAction::default(),
            edit_on_min_per_eval_not_met: MinimumNotMetAction::default(),
            edit_on_min_one_eval_not_met: MinimumNotMetAction::default(),
            edit_round_before_weighting: false,
            edit_global_policy: GlobalExamPolicy::default(),
            edit_global_semester_weight: String::new(),
            edit_global_exam_weight: String::new(),
            edit_global_min_grade: String::new(),
            edit_global_grade: String::new(),
            edit_bulk_count: String::new(),
            show_advanced_rules: false,
            show_field_help: false,
            use_nerd_fonts: true,
            compact_courses: false,
            selected_setting: 0,
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
        let (config, config_warning) = persistence::load_config();
        let language = config.language;
        let use_nerd_fonts = config.use_nerd_fonts;
        let compact_courses = config.compact_courses;
        let m = language.messages();
        let mut status_message: Option<String> = None;

        // Surface config load warning via i18n
        if config_warning.is_some() {
            status_message = Some(m.config_load_warning.to_string());
        }

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

        let selected_category = selected_course.and_then(|idx| {
            courses
                .get(idx)
                .filter(|c| !c.categories.is_empty())
                .map(|_| 0)
        });

        Self {
            courses,
            built_in_templates,
            user_templates,
            selected_course,
            selected_category,
            status_message,
            language,
            use_nerd_fonts,
            compact_courses,
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
    /// Returns `None` when the virtual global category is selected.
    pub fn current_category(&self) -> Option<&Category> {
        self.current_course()
            .and_then(|c| self.selected_category.and_then(|i| c.categories.get(i)))
    }

    /// Whether the current course should display a virtual "Global" category
    /// at the end of the category list.  True when the course needs a global
    /// exam and it is still possible to pass (or the global grade has already
    /// been entered).
    pub fn shows_virtual_global(&self) -> bool {
        let Some(course) = self.current_course() else {
            return false;
        };
        if course.global_policy == GlobalExamPolicy::None {
            return false;
        }
        let result = course.compute_grade();
        if !result.needs_global {
            return false;
        }
        // If a global grade was already entered, always show the row so the
        // user can see / edit it.
        if course.global_exam_grade.is_some() {
            return true;
        }
        // Otherwise show only when it is still possible to pass.
        !matches!(
            course.needed_global_grade().status,
            NeededGradeStatus::Failure
        )
    }

    /// Whether the currently selected category index points to the virtual
    /// global row (index == categories.len()).
    pub fn is_on_virtual_global(&self) -> bool {
        let Some(course) = self.current_course() else {
            return false;
        };
        let Some(idx) = self.selected_category else {
            return false;
        };
        idx == course.categories.len() && self.shows_virtual_global()
    }

    /// Get the currently selected Evaluation, if any.
    pub fn current_evaluation(&self) -> Option<&Evaluation> {
        self.current_category()
            .and_then(|c| self.selected_evaluation.and_then(|i| c.evaluations.get(i)))
    }

    /// Total number of visible category rows for the current course,
    /// including the virtual global row when applicable.
    fn visible_category_count(&self) -> usize {
        let Some(course) = self.current_course() else {
            return 0;
        };
        let extra = if self.shows_virtual_global() { 1 } else { 0 };
        course.categories.len() + extra
    }

    /// Reset category selection to the first category of the current course
    /// (or None if the course has no categories). Also clears evaluation selection.
    pub fn reset_category_selection(&mut self) {
        let count = self.visible_category_count();
        self.selected_category = if count == 0 { None } else { Some(0) };
        self.selected_evaluation = None;
    }

    /// Reset evaluation selection to the first evaluation of the current category
    /// (or None if the category has no evaluations).
    pub fn reset_evaluation_selection(&mut self) {
        self.selected_evaluation = self.current_category().and_then(|c| {
            if c.evaluations.is_empty() {
                None
            } else {
                Some(0)
            }
        });
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
        self.reset_category_selection();
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
        self.reset_category_selection();
    }

    pub fn next_category(&mut self) {
        let count = self.visible_category_count();
        if count == 0 {
            self.selected_category = None;
            return;
        }
        self.selected_category = Some(match self.selected_category {
            Some(i) => (i + 1).min(count - 1),
            None => 0,
        });
        self.reset_evaluation_selection();
    }

    pub fn previous_category(&mut self) {
        let count = self.visible_category_count();
        if count == 0 {
            self.selected_category = None;
            return;
        }
        self.selected_category = Some(match self.selected_category {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
        self.reset_evaluation_selection();
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

    /// Toggle the advanced rules section in the category popup.
    /// When collapsing, snap focus back to Weight if currently on an advanced field.
    pub fn toggle_advanced_rules(&mut self) {
        self.show_advanced_rules = !self.show_advanced_rules;

        // If we just collapsed and the focus is on an advanced field, snap back
        if !self.show_advanced_rules {
            match self.input_field {
                InputField::DropLowest
                | InputField::AvgMethod
                | InputField::MinimumAverage
                | InputField::OnMinNotMet
                | InputField::MinPerEval
                | InputField::RoundBeforeWeight => {
                    self.input_field = InputField::Weight;
                }
                _ => {}
            }
        }
    }

    /// Toggle the field help panel in the category popup.
    pub fn toggle_field_help(&mut self) {
        self.show_field_help = !self.show_field_help;
    }

    pub fn next_input_field(&mut self) {
        self.input_field = match (&self.screen, &self.input_field) {
            (Screen::EditingCourse { .. }, InputField::Name) => InputField::PassingGrade,
            (Screen::EditingCourse { .. }, InputField::PassingGrade) => InputField::GlobalPolicy,
            (Screen::EditingCourse { .. }, InputField::GlobalPolicy) => {
                match self.edit_global_policy {
                    GlobalExamPolicy::None => InputField::Name,
                    GlobalExamPolicy::Weighted { .. } => InputField::GlobalSemesterWeight,
                    GlobalExamPolicy::ReplacesWorstGrade => InputField::GlobalMinGrade,
                }
            }
            (Screen::EditingCourse { .. }, InputField::GlobalSemesterWeight) => {
                InputField::GlobalExamWeight
            }
            (Screen::EditingCourse { .. }, InputField::GlobalExamWeight) => {
                InputField::GlobalMinGrade
            }
            (Screen::EditingCourse { .. }, InputField::GlobalMinGrade) => InputField::Name,
            // Category: Name → Weight → (advanced fields if expanded) → Name
            (Screen::EditingCategory { .. }, InputField::Name) => InputField::Weight,
            (Screen::EditingCategory { .. }, InputField::Weight) => {
                if self.show_advanced_rules {
                    InputField::DropLowest
                } else {
                    InputField::Name
                }
            }
            (Screen::EditingCategory { .. }, InputField::DropLowest) => InputField::AvgMethod,
            (Screen::EditingCategory { .. }, InputField::AvgMethod) => InputField::MinimumAverage,
            (Screen::EditingCategory { .. }, InputField::MinimumAverage) => {
                // Show OnMinNotMet only if a minimum average is set
                if !self.edit_min_average.trim().is_empty() {
                    InputField::OnMinNotMet
                } else {
                    InputField::MinPerEval
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinNotMet) => InputField::MinPerEval,
            (Screen::EditingCategory { .. }, InputField::MinPerEval) => {
                // Show OnMinPerEvalNotMet only if min per eval is set
                if !self.edit_min_per_eval.trim().is_empty() {
                    InputField::OnMinPerEvalNotMet
                } else {
                    InputField::MinOneEval
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinPerEvalNotMet) => {
                InputField::MinOneEval
            }
            (Screen::EditingCategory { .. }, InputField::MinOneEval) => {
                // Show OnMinOneEvalNotMet only if min one eval is set
                if !self.edit_min_one_eval.trim().is_empty() {
                    InputField::OnMinOneEvalNotMet
                } else {
                    InputField::RoundBeforeWeight
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinOneEvalNotMet) => {
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
            InputField::MinimumAverage => &mut self.edit_min_average,
            InputField::MinPerEval => &mut self.edit_min_per_eval,
            InputField::MinOneEval => &mut self.edit_min_one_eval,
            InputField::GlobalSemesterWeight => &mut self.edit_global_semester_weight,
            InputField::GlobalExamWeight => &mut self.edit_global_exam_weight,
            InputField::GlobalMinGrade => &mut self.edit_global_min_grade,
            // Toggle fields don't have text buffers — they are cycled, not typed into.
            // This branch should never be reached in practice.
            InputField::DropLowest
            | InputField::AvgMethod
            | InputField::OnMinNotMet
            | InputField::OnMinPerEvalNotMet
            | InputField::OnMinOneEvalNotMet
            | InputField::RoundBeforeWeight
            | InputField::GlobalPolicy => {
                debug_assert!(
                    false,
                    "current_input_buffer called on toggle field {:?}",
                    self.input_field
                );
                &mut self.edit_description
            }
        }
    }

    /// Cycle the current toggle field forward (→ / Space / Enter / l).
    pub fn cycle_toggle_field(&mut self) {
        match self.input_field {
            InputField::DropLowest => {
                self.edit_drop_lowest = (self.edit_drop_lowest + 1) % (MAX_DROP_LOWEST + 1);
            }
            InputField::AvgMethod => {
                self.edit_averaging_method = match self.edit_averaging_method {
                    AveragingMethod::Arithmetic => AveragingMethod::Geometric,
                    AveragingMethod::Geometric => AveragingMethod::Arithmetic,
                };
            }
            InputField::OnMinNotMet => {
                self.edit_on_min_not_met = match self.edit_on_min_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = match self.edit_on_min_per_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = match self.edit_on_min_one_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::RoundBeforeWeight => {
                self.edit_round_before_weighting = !self.edit_round_before_weighting;
            }
            InputField::GlobalPolicy => {
                self.edit_global_policy = match self.edit_global_policy {
                    GlobalExamPolicy::None => GlobalExamPolicy::Weighted {
                        semester_weight: 0.7,
                        global_weight: 0.3,
                    },
                    GlobalExamPolicy::Weighted { .. } => GlobalExamPolicy::ReplacesWorstGrade,
                    GlobalExamPolicy::ReplacesWorstGrade => GlobalExamPolicy::None,
                };
            }
            _ => {}
        }
    }

    /// Cycle the current toggle field backward (← / h).
    pub fn cycle_toggle_field_reverse(&mut self) {
        match self.input_field {
            InputField::DropLowest => {
                self.edit_drop_lowest = if self.edit_drop_lowest == 0 {
                    MAX_DROP_LOWEST
                } else {
                    self.edit_drop_lowest - 1
                };
            }
            // Two-state toggles: reverse == forward
            InputField::AvgMethod | InputField::RoundBeforeWeight => self.cycle_toggle_field(),
            // Three-state toggle: reverse cycle
            InputField::OnMinNotMet => {
                self.edit_on_min_not_met = match self.edit_on_min_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = match self.edit_on_min_per_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = match self.edit_on_min_one_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            // Three-state toggle: reverse cycle
            InputField::GlobalPolicy => {
                self.edit_global_policy = match self.edit_global_policy {
                    GlobalExamPolicy::None => GlobalExamPolicy::ReplacesWorstGrade,
                    GlobalExamPolicy::ReplacesWorstGrade => GlobalExamPolicy::Weighted {
                        semester_weight: 0.7,
                        global_weight: 0.3,
                    },
                    GlobalExamPolicy::Weighted { .. } => GlobalExamPolicy::None,
                };
            }
            _ => {}
        }
    }

    pub fn cancel_edit(&mut self) {
        self.screen = Screen::Main;
    }
}
