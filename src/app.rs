//! Application state and logic.
//!
//! This module contains the main App struct that holds all application state,
//! as well as methods for navigation, form handling, and state management.

use crate::model::{
    Category, Course, CourseTemplate, DEFAULT_PASSING_GRADE, Evaluation, MAX_GRADE, MIN_GRADE,
};
use crate::persistence;
use crate::templates;

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
}

/// Input field being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Name,
    PassingGrade,
    Weight,
    Grade,
    Description,
}

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

    // UI state
    pub focus: Focus,
    pub screen: Screen,
    pub should_quit: bool,

    // Input state for forms
    pub input_field: InputField,
    pub edit_name: String,
    pub edit_passing_grade: String,
    pub edit_weight: String,
    pub edit_grade: String,
    pub edit_description: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            courses: Vec::new(),
            built_in_templates: templates::built_in_templates(),
            user_templates: Vec::new(),
            selected_course: None,
            selected_category: None,
            selected_evaluation: None,
            selected_template: 0,
            focus: Focus::Courses,
            screen: Screen::Main,
            should_quit: false,
            input_field: InputField::Name,
            edit_name: String::new(),
            edit_passing_grade: String::new(),
            edit_weight: String::new(),
            edit_grade: String::new(),
            edit_description: String::new(),
        }
    }
}

impl App {
    /// Load application state from disk, or create empty state if file doesn't exist.
    pub fn load() -> Self {
        let courses = match persistence::load_data() {
            Ok(courses) => courses,
            Err(err) => {
                eprintln!("Warning: failed to load saved data, starting fresh: {err}");
                Vec::new()
            }
        };

        let user_templates = match persistence::load_user_templates() {
            Ok(templates) => templates,
            Err(err) => {
                eprintln!("Warning: failed to load user templates: {err}");
                Vec::new()
            }
        };

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
            user_templates,
            selected_course,
            selected_category,
            ..Default::default()
        }
    }

    /// Save current state to disk.
    pub fn save(&self) -> color_eyre::Result<()> {
        persistence::save_data(&self.courses)
    }

    // ==========================================================================
    // Getters
    // ==========================================================================

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

    // ==========================================================================
    // Navigation
    // ==========================================================================

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

    // ==========================================================================
    // Form Handling - Course
    // ==========================================================================

    pub fn start_new_course(&mut self) {
        self.screen = Screen::SelectingTemplate;
        self.selected_template = 0;
    }

    pub fn confirm_template_selection(&mut self) {
        self.screen = Screen::EditingCourse { is_new: true };
        self.input_field = InputField::Name;
        self.edit_name.clear();
        self.edit_passing_grade = format!("{DEFAULT_PASSING_GRADE:.0}");
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
        self.edit_name = course.name.clone();
        let pg = course.passing_grade;
        self.edit_passing_grade = format!("{pg:.0}");
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

        match self.screen {
            Screen::EditingCourse { is_new: true } => {
                let course = if let Some(template) = self.current_template() {
                    Course::from_template(name, passing_grade, template)
                } else {
                    Course::new(name, passing_grade)
                };
                self.courses.push(course);
                self.selected_course = Some(self.courses.len() - 1);

                // Auto-select first category if course was created from template
                self.selected_category = self.current_course().and_then(|c| {
                    if c.categories.is_empty() {
                        None
                    } else {
                        Some(0)
                    }
                });

                // Auto-select first evaluation if category has evaluations
                self.selected_evaluation = self.current_category().and_then(|c| {
                    if c.evaluations.is_empty() {
                        None
                    } else {
                        Some(0)
                    }
                });
            }
            Screen::EditingCourse { is_new: false } => {
                if let Some(idx) = self.selected_course
                    && let Some(course) = self.courses.get_mut(idx)
                {
                    course.name = name;
                    course.passing_grade = passing_grade;
                }
            }
            _ => {}
        }

        self.screen = Screen::Main;
        self.persist();
    }

    // ==========================================================================
    // Form Handling - Category
    // ==========================================================================

    pub fn start_new_category(&mut self) {
        if self.current_course().is_some() {
            self.screen = Screen::EditingCategory { is_new: true };
            self.input_field = InputField::Name;
            self.edit_name.clear();
            self.edit_weight = "20.0".to_string();
        }
    }

    pub fn start_edit_category(&mut self) {
        let Some((name, weight)) = self
            .current_category()
            .map(|category| (category.name.clone(), category.weight))
        else {
            return;
        };

        self.screen = Screen::EditingCategory { is_new: false };
        self.input_field = InputField::Name;
        self.edit_name = name;
        self.edit_weight = format!("{weight:.1}");
    }

    pub fn confirm_category(&mut self) {
        let name = self.edit_name.trim().to_string();
        let weight: f64 = self
            .edit_weight
            .parse::<f64>()
            .unwrap_or(20.0)
            .clamp(MIN_GRADE, MAX_GRADE);

        if name.is_empty() {
            return;
        }

        let selected_cat = self.selected_category;

        match self.screen {
            Screen::EditingCategory { is_new: true } => {
                if let Some(idx) = self.selected_course
                    && let Some(course) = self.courses.get_mut(idx)
                {
                    let category = Category::new(name, weight);
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
                }
            }
            _ => {}
        }

        self.screen = Screen::Main;
        self.persist();
    }

    // ==========================================================================
    // Form Handling - Evaluation
    // ==========================================================================

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
        self.input_field = InputField::Grade; // Start with grade field
        self.edit_name = name;
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
        self.persist();
    }

    // ==========================================================================
    // Deletion
    // ==========================================================================

    pub fn request_delete(&mut self) {
        let can_delete = match self.focus {
            Focus::Courses => self.selected_course.is_some(),
            Focus::Categories => self.selected_category.is_some(),
            Focus::Evaluations => self.selected_evaluation.is_some(),
        };
        if can_delete {
            self.screen = Screen::ConfirmDelete;
        }
    }

    pub fn delete_current(&mut self) {
        match self.focus {
            Focus::Courses => {
                if let Some(idx) = self.selected_course {
                    self.courses.remove(idx);
                    self.selected_course = if self.courses.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.courses.len() - 1))
                    };
                    // Update category selection for new course
                    self.selected_category = self.current_course().and_then(|c| {
                        if c.categories.is_empty() {
                            None
                        } else {
                            Some(0)
                        }
                    });
                    self.selected_evaluation = None;
                }
            }
            Focus::Categories => {
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
            Focus::Evaluations => {
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
        self.persist();
    }

    // ==========================================================================
    // Weight Management
    // ==========================================================================

    /// Auto-balance weights for current course
    pub fn auto_balance_weights(&mut self) {
        if let Some(course_idx) = self.selected_course
            && let Some(course) = self.courses.get_mut(course_idx)
        {
            course.auto_balance_weights();
            self.persist();
        }
    }

    // ==========================================================================
    // Input Field Navigation
    // ==========================================================================

    pub fn next_input_field(&mut self) {
        self.input_field = match (&self.screen, &self.input_field) {
            (Screen::EditingCourse { .. }, InputField::Name) => InputField::PassingGrade,
            (Screen::EditingCourse { .. }, InputField::PassingGrade) => InputField::Name,
            (Screen::EditingCategory { .. }, InputField::Name) => InputField::Weight,
            (Screen::EditingCategory { .. }, InputField::Weight) => InputField::Name,
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
        }
    }

    pub fn cancel_edit(&mut self) {
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
        if let Err(err) = persistence::save_user_templates(&self.user_templates) {
            eprintln!("Warning: failed to save user templates: {err}");
        }

        self.screen = Screen::SelectingTemplate;
    }

    /// Persist state and log (without panicking) on failure.
    fn persist(&self) {
        if let Err(err) = self.save() {
            eprintln!("Warning: failed to save data: {err}");
        }
    }

    // ==========================================================================
    // Save as Template
    // ==========================================================================

    /// Start the "save as template" flow for current course.
    pub fn start_save_as_template(&mut self) {
        let Some(course) = self.current_course() else {
            return;
        };

        // Pre-fill with course name and description
        let name = format!("{} Template", course.name);
        let description = self.generate_template_description(course);

        self.edit_name = name;
        self.edit_description = description;
        self.input_field = InputField::Name;
        self.screen = Screen::SavingTemplate;
    }

    /// Generate a description based on course structure.
    fn generate_template_description(&self, course: &Course) -> String {
        if course.categories.is_empty() {
            return "Empty template".to_string();
        }

        course
            .categories
            .iter()
            .map(|c| format!("{} {}x ({:.0}%)", c.evaluations.len(), c.name, c.weight))
            .collect::<Vec<_>>()
            .join(" + ")
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
        if let Err(err) = persistence::save_user_templates(&self.user_templates) {
            eprintln!("Warning: failed to save user templates: {err}");
        }

        self.screen = Screen::Main;
    }
}
