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

//! Application state and logic.
//!
//! This module contains the main `App` struct that holds all application state,
//! as well as methods for navigation, getters, and state management.
//! Form handling (course/category/evaluation editing) lives in `forms`.
//! Secondary actions (templates, language, settings, global grade, yank/paste,
//! bulk-add) live in `actions`.

mod actions;
mod attendance;
mod forms;
mod history;
pub mod import;
mod input;
mod semesters;
mod status;
mod welcome;

pub use input::InputField;
pub use status::StatusSeverity;

use crate::i18n::{Language, Messages};
use crate::model::{
    AttendanceAction, AveragingMethod, Category, Course, CourseTemplate, Evaluation,
    GlobalExamPolicy, MinimumNotMetAction, NeededGradeStatus, Semester,
};
use crate::persistence;
use crate::templates;

use history::Snapshot;

/// Auto-generated name for the semester at `order`, e.g. "Semester 1".
/// Only a starting point — the user renames it.
fn semester_name(prefix: &str, order: u32) -> String {
    format!("{} {}", prefix, order + 1)
}

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
    EditingCourse {
        is_new: bool,
    },
    EditingCategory {
        is_new: bool,
    },
    EditingEvaluation {
        is_new: bool,
    },
    ConfirmDelete,
    ConfirmDeleteTemplate,
    SavingTemplate,
    SelectingLanguage,
    Settings,
    BulkAddEvaluations,
    EnteringGlobalGrade,
    /// Full keyboard cheat-sheet overlay (opened with `?` from Main).
    Help,
    /// AI import wizard — step 1: shows the prompt to copy to clipboard.
    ImportPrompt,
    /// AI import wizard — step 2: waits for the user to paste the AI response.
    ImportPaste,
    /// AI import wizard — step 3: previews the parsed course and confirms.
    ImportPreview,
    /// Semester list and dashboard.
    Home,
    /// First run, step 1: pick a language.
    Welcome,
    /// First run, step 2: confirm whether Nerd Font icons render.
    WelcomeFonts,
    EditingSemester {
        is_new: bool,
    },
    ConfirmDeleteSemester,
    /// Attendance for the selected course. Its own popup rather than four more
    /// rows in a course form that already fills a short terminal.
    EditingAttendance,
}

// =============================================================================
// App Struct
// =============================================================================

/// Main application state.
#[derive(Debug)]
pub struct App {
    /// Oldest first by `order`. Invariant: never empty.
    pub semesters: Vec<Semester>,
    pub built_in_templates: Vec<CourseTemplate>,
    pub user_templates: Vec<CourseTemplate>,

    // Selection state
    pub selected_semester: usize,
    pub selected_course: Option<usize>,
    pub selected_category: Option<usize>,
    pub selected_evaluation: Option<usize>,
    pub selected_template: usize,
    pub selected_language: usize,

    // UI state
    pub focus: Focus,
    pub screen: Screen,
    /// Screen to return to when a full-screen overlay (help, language) closes.
    pub return_screen: Screen,
    pub should_quit: bool,

    /// Clipboard for yanked evaluation (name, grade, weight).
    pub clipboard_evaluation: Option<(String, Option<f64>, Option<f64>)>,

    /// Temporary status message shown to the user (errors, confirmations, etc.)
    /// Cleared on the next action.
    pub status_message: Option<String>,
    /// Severity of the current `status_message` — controls its colour.
    pub status_severity: StatusSeverity,

    /// Undo stack: snapshots taken before each mutating action.
    pub undo_stack: Vec<Snapshot>,
    /// Redo stack: snapshots displaced by undo, repopulated on redo.
    pub redo_stack: Vec<Snapshot>,

    // Language
    pub language: Language,

    // Input state for forms
    pub input_field: InputField,
    pub edit_name: String,
    pub edit_passing_grade: String,
    /// Optional course credits, empty string means "not declared".
    pub edit_credits: String,
    pub edit_weight: String,
    pub edit_grade: String,
    pub edit_description: String,
    /// Temporary buffer for evaluation weight (%) in weighted-evaluations categories.
    pub edit_eval_weight: String,

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
    /// Ceiling used by the CapFinalGrade action, as typed.
    pub edit_cap_final_grade: String,
    /// Whether the category uses individually weighted evaluations.
    pub edit_weighted_evaluations: bool,

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

    // Attendance editing state (course form)
    pub edit_classes_total: String,
    pub edit_classes_missed: String,
    pub edit_attendance_required: String,
    pub edit_attendance_action: AttendanceAction,

    /// Whether to use Nerd Font icons (persisted in config).
    pub use_nerd_fonts: bool,
    /// False until the first-run wizard is finished; quitting midway keeps it
    /// false so the wizard comes back.
    pub configured: bool,
    /// Index of the focused setting in the settings popup.
    pub selected_setting: usize,

    // -- AI import wizard state ----------------------------------------------
    /// Set to `true` after the user copies the prompt in step 1 so the UI can
    /// flash a "Copied" confirmation.  Cleared on screen change.
    pub import_copied: bool,
    /// When `true`, step 1 renders the prompt full-screen with no borders so
    /// the user can select-and-copy with the mouse in any terminal (universal
    /// fallback when OSC 52 isn't supported).
    pub import_prompt_fullscreen: bool,
    /// Vertical scroll offset for the step-3 preview, which can outgrow the
    /// popup once a course has many evaluations.
    pub import_preview_scroll: u16,
    /// Vertical scroll offset (in source lines) for the step-1 prompt view —
    /// shared between the popup and full-screen variants so the user keeps
    /// their place when toggling.
    pub import_prompt_scroll: u16,
    /// Error message produced by the last paste attempt in step 2 (if any).
    pub import_paste_error: Option<String>,
    /// Course parsed from the AI response, ready to confirm in step 3.
    pub import_parsed: Option<Course>,
    /// Whether the parsed course was renamed because of a name collision
    /// (so the preview can show a note).
    pub import_renamed_from: Option<String>,
    /// Cached weight sum from the parsed schema (for the preview banner).
    pub import_total_weight: f64,
    /// Cached evaluation count from the parsed schema.
    pub import_total_evals: usize,
}

impl Default for App {
    fn default() -> Self {
        let language = Language::default();
        Self {
            semesters: vec![Semester::new(
                semester_name(language.messages().semester_name_prefix, 0),
                0,
            )],
            built_in_templates: templates::built_in_templates(language),
            user_templates: Vec::new(),
            selected_semester: 0,
            selected_course: None,
            selected_category: None,
            selected_evaluation: None,
            selected_template: 0,
            selected_language: 0,
            focus: Focus::Courses,
            screen: Screen::Main,
            return_screen: Screen::Main,
            should_quit: false,
            clipboard_evaluation: None,
            status_message: None,
            status_severity: StatusSeverity::Info,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            language,
            input_field: InputField::Name,
            edit_name: String::new(),
            edit_passing_grade: String::new(),
            edit_credits: String::new(),
            edit_weight: String::new(),
            edit_grade: String::new(),
            edit_description: String::new(),
            edit_eval_weight: String::new(),
            edit_drop_lowest: 0,
            edit_min_average: String::new(),
            edit_min_per_eval: String::new(),
            edit_min_one_eval: String::new(),
            edit_averaging_method: AveragingMethod::default(),
            edit_on_min_not_met: MinimumNotMetAction::default(),
            edit_on_min_per_eval_not_met: MinimumNotMetAction::default(),
            edit_on_min_one_eval_not_met: MinimumNotMetAction::default(),
            edit_round_before_weighting: false,
            edit_cap_final_grade: String::new(),
            edit_weighted_evaluations: false,
            edit_global_policy: GlobalExamPolicy::default(),
            edit_global_semester_weight: String::new(),
            edit_global_exam_weight: String::new(),
            edit_global_min_grade: String::new(),
            edit_global_grade: String::new(),
            edit_bulk_count: String::new(),
            edit_classes_total: String::new(),
            edit_classes_missed: String::new(),
            edit_attendance_required: String::new(),
            edit_attendance_action: AttendanceAction::default(),
            show_advanced_rules: false,
            show_field_help: false,
            use_nerd_fonts: true,
            configured: true,
            selected_setting: 0,
            import_copied: false,
            import_prompt_fullscreen: false,
            import_prompt_scroll: 0,
            import_preview_scroll: 0,
            import_paste_error: None,
            import_parsed: None,
            import_renamed_from: None,
            import_total_weight: 0.0,
            import_total_evals: 0,
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
        let m = language.messages();
        let mut status_message: Option<String> = None;
        let mut status_severity = StatusSeverity::Info;

        // Surface config load warning via i18n
        if config_warning.is_some() {
            status_message = Some(m.config_load_warning.to_string());
            status_severity = StatusSeverity::Warning;
        }

        let mut semesters = match persistence::load_data(&semester_name(m.semester_name_prefix, 0))
        {
            Ok(semesters) => semesters,
            Err(_) => {
                status_message = Some(m.load_error.to_string());
                status_severity = StatusSeverity::Error;
                Vec::new()
            }
        };

        // Upholds the "never empty" invariant on a fresh install or failed load.
        if semesters.is_empty() {
            semesters.push(Semester::new(semester_name(m.semester_name_prefix, 0), 0));
        }
        semesters.sort_by_key(|s| s.order);

        let user_templates = match persistence::load_user_templates() {
            Ok(templates) => templates,
            Err(_) => {
                status_message = Some(m.load_template_error.to_string());
                status_severity = StatusSeverity::Warning;
                Vec::new()
            }
        };

        // Generate built-in templates for current language
        let built_in_templates = templates::built_in_templates(language);

        // Reopen where the last session ended, falling back to the most
        // recent semester.
        let selected_semester = config
            .last_semester
            .and_then(|id| semesters.iter().position(|s| s.id == id))
            .unwrap_or(semesters.len() - 1);
        let courses = &semesters[selected_semester].courses;

        let selected_course = if courses.is_empty() { None } else { Some(0) };

        let selected_category = selected_course.and_then(|idx| {
            courses
                .get(idx)
                .filter(|c| !c.categories.is_empty())
                .map(|_| 0)
        });

        let screen = if !config.configured {
            Screen::Welcome
        } else if config.start_on_home {
            Screen::Home
        } else {
            Screen::Main
        };

        Self {
            semesters,
            screen,
            built_in_templates,
            user_templates,
            selected_semester,
            selected_course,
            selected_category,
            status_message,
            status_severity,
            language,
            use_nerd_fonts,
            configured: config.configured,
            ..Default::default()
        }
    }

    /// Get the current messages for the selected language.
    pub fn messages(&self) -> &'static Messages {
        self.language.messages()
    }

    /// Save current state to disk.
    pub fn save(&self) -> color_eyre::Result<()> {
        persistence::save_data(&self.semesters)
    }

    // =========================================================================
    // Semester access
    // =========================================================================

    /// The semester currently in view.
    pub fn current_semester(&self) -> &Semester {
        let idx = self.selected_semester.min(self.semesters.len() - 1);
        &self.semesters[idx]
    }

    /// Courses of the semester currently in view.
    pub fn courses(&self) -> &[Course] {
        &self.current_semester().courses
    }

    /// Mutable courses of the semester currently in view.
    pub fn courses_mut(&mut self) -> &mut Vec<Course> {
        debug_assert!(
            !self.semesters.is_empty(),
            "App must always hold at least one semester"
        );
        if self.semesters.is_empty() {
            self.semesters.push(Semester::new(String::new(), 0));
        }
        let idx = self.selected_semester.min(self.semesters.len() - 1);
        &mut self.semesters[idx].courses
    }

    /// Persist state to disk. Shows error to user via status message on failure.
    pub(crate) fn persist(&mut self) {
        if self.save().is_err() {
            let msg = self.messages().save_error.to_string();
            self.set_error(msg);
        }
    }

    // =========================================================================
    // Getters
    // =========================================================================

    /// Get the currently selected Course, if any.
    pub fn current_course(&self) -> Option<&Course> {
        self.selected_course.and_then(|i| self.courses().get(i))
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

    /// Get all real templates combined: built-in first, then user templates.
    ///
    /// Note: the AI import option occupies the synthetic index 0 in the UI;
    /// real templates start at displayed index 1.  This getter still returns
    /// only the real templates — callers translating from `selected_template`
    /// must subtract 1.
    pub fn all_templates(&self) -> Vec<&CourseTemplate> {
        self.built_in_templates
            .iter()
            .chain(self.user_templates.iter())
            .collect()
    }

    /// `true` when the synthetic "Create with AI" entry is selected (index 0).
    pub fn is_ai_template_selected(&self) -> bool {
        self.selected_template == 0
    }

    /// Get the currently selected real template, or `None` if the AI entry
    /// (index 0) is the current selection.
    pub fn current_template(&self) -> Option<&CourseTemplate> {
        if self.selected_template == 0 {
            return None;
        }
        self.all_templates()
            .get(self.selected_template - 1)
            .copied()
    }

    /// Count of real templates (excluding the synthetic AI entry).
    pub fn templates_count(&self) -> usize {
        self.built_in_templates.len() + self.user_templates.len()
    }

    /// Total number of selectable rows in the template picker, including the
    /// AI entry at index 0.
    pub fn selectable_templates_count(&self) -> usize {
        1 + self.templates_count()
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    pub fn next_course(&mut self) {
        if self.courses().is_empty() {
            self.selected_course = None;
            return;
        }
        self.selected_course = Some(match self.selected_course {
            Some(i) => (i + 1).min(self.courses().len() - 1),
            None => 0,
        });
        self.reset_category_selection();
    }

    pub fn previous_course(&mut self) {
        if self.courses().is_empty() {
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
        let count = self.selectable_templates_count();
        if count > 0 {
            self.selected_template = (self.selected_template + 1) % count;
        }
    }

    pub fn previous_template(&mut self) {
        let count = self.selectable_templates_count();
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

    pub fn cancel_edit(&mut self) {
        self.screen = Screen::Main;
    }

    /// Returns true if the currently selected category has weighted evaluations enabled.
    pub fn category_has_weighted_evals(&self) -> bool {
        self.current_category()
            .is_some_and(|cat| cat.rules.weighted_evaluations)
    }
}
