//! Kalk - Your academic dashboard in the terminal
//!
//! A TUI application for managing courses and calculating grades.

// =============================================================================
// MODULES (inline for now, can be extracted to separate files later)
// =============================================================================

mod model {
    //! Data models for the application.

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// Represents a single evaluation within a course.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Evaluation {
        pub id: Uuid,
        pub name: String,
        /// Weight of this evaluation (0.0 - 100.0 percentage)
        pub weight: f64,
        /// Grade obtained (None if not yet graded)
        pub grade: Option<f64>,
    }

    impl Evaluation {
        pub fn new(name: String, weight: f64) -> Self {
            Self {
                id: Uuid::new_v4(),
                name,
                weight,
                grade: None,
            }
        }
    }

    /// Represents a course with its evaluations.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Course {
        pub id: Uuid,
        pub name: String,
        /// Minimum grade required to pass (e.g., 4.0)
        pub passing_grade: f64,
        pub evaluations: Vec<Evaluation>,
    }

    impl Course {
        pub fn new(name: String, passing_grade: f64) -> Self {
            Self {
                id: Uuid::new_v4(),
                name,
                passing_grade,
                evaluations: Vec::new(),
            }
        }

        /// Calculates the current weighted average of graded evaluations.
        pub fn current_grade(&self) -> Option<f64> {
            let (weighted_sum, total_weight): (f64, f64) = self
                .evaluations
                .iter()
                .filter_map(|e| e.grade.map(|g| (g * e.weight, e.weight)))
                .fold((0.0, 0.0), |(sum, weight), (g, w)| (sum + g, weight + w));

            if total_weight > 0.0 {
                Some(weighted_sum / total_weight)
            } else {
                None
            }
        }

        /// Calculates the remaining weight percentage not yet graded.
        pub fn remaining_weight(&self) -> f64 {
            let graded_weight: f64 = self
                .evaluations
                .iter()
                .filter(|e| e.grade.is_some())
                .map(|e| e.weight)
                .sum();

            100.0 - graded_weight
        }

        /// Returns a descriptive string about the grade status.
        /// Tells the user what grade they need in the remaining evaluations to pass.
        pub fn calculate_required_grade(&self) -> String {
            let remaining_weight = self.remaining_weight();

            if remaining_weight <= 0.0 {
                // All evaluations are graded
                return match self.current_grade() {
                    Some(grade) if grade >= self.passing_grade => {
                        format!("Passed with {:.1}", grade)
                    }
                    Some(grade) => format!("Failed with {:.1}", grade),
                    None => "No evaluations".to_string(),
                };
            }

            let current_grade = self.current_grade();
            let graded_weight = 100.0 - remaining_weight;

            match current_grade {
                Some(current) => {
                    // Calculate: (current * graded_weight + X * remaining_weight) / 100 = passing_grade
                    // X = (passing_grade * 100 - current * graded_weight) / remaining_weight
                    let required_grade =
                        (self.passing_grade * 100.0 - current * graded_weight) / remaining_weight;

                    if required_grade <= 1.0 {
                        format!(
                            "Current: {:.1} | Already passing (need >=1.0 in remaining {:.0}%)",
                            current, remaining_weight
                        )
                    } else if required_grade > 7.0 {
                        format!(
                            "Current: {:.1} | Impossible to pass (would need {:.1})",
                            current, required_grade
                        )
                    } else {
                        format!(
                            "Current: {:.1} | Need {:.1} in remaining {:.0}%",
                            current, required_grade, remaining_weight
                        )
                    }
                }
                None => {
                    format!("No grades | Need {:.1} to pass", self.passing_grade)
                }
            }
        }
    }
}

mod state {
    //! Application state management.

    use crate::model::{Course, Evaluation};
    use crate::persistence;

    /// Which panel is currently focused.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Focus {
        Courses,
        Evaluations,
    }

    /// Current screen/mode of the application.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Screen {
        Main,
        EditingCourse { is_new: bool },
        EditingEvaluation { is_new: bool },
        ConfirmDelete,
    }

    /// Input field being edited.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InputField {
        Name,
        PassingGrade,
        Weight,
        Grade,
    }

    /// Main application state.
    #[derive(Debug)]
    pub struct App {
        pub courses: Vec<Course>,
        pub selected_course: Option<usize>,
        pub selected_evaluation: Option<usize>,
        pub focus: Focus,
        pub screen: Screen,
        pub should_quit: bool,

        // Input state for forms
        pub input_field: InputField,
        pub edit_name: String,
        pub edit_passing_grade: String,
        pub edit_weight: String,
        pub edit_grade: String,
    }

    impl Default for App {
        fn default() -> Self {
            Self {
                courses: Vec::new(),
                selected_course: None,
                selected_evaluation: None,
                focus: Focus::Courses,
                screen: Screen::Main,
                should_quit: false,
                input_field: InputField::Name,
                edit_name: String::new(),
                edit_passing_grade: String::new(),
                edit_weight: String::new(),
                edit_grade: String::new(),
            }
        }
    }

    impl App {
        /// Load application state from disk, or create empty state if file doesn't exist.
        pub fn load() -> Self {
            let courses = persistence::load_data().unwrap_or_default();
            let selected_course = if courses.is_empty() { None } else { Some(0) };

            Self {
                courses,
                selected_course,
                ..Default::default()
            }
        }

        /// Save current state to disk.
        pub fn save(&self) -> color_eyre::Result<()> {
            persistence::save_data(&self.courses)
        }

        /// Get the currently selected Course, if any.
        pub fn current_course(&self) -> Option<&Course> {
            self.selected_course.and_then(|i| self.courses.get(i))
        }

        /// Get the currently selected Evaluation, if any.
        pub fn current_evaluation(&self) -> Option<&Evaluation> {
            self.current_course()
                .and_then(|c| self.selected_evaluation.and_then(|i| c.evaluations.get(i)))
        }

        // Navigation methods
        pub fn next_course(&mut self) {
            if self.courses.is_empty() {
                self.selected_course = None;
                return;
            }
            self.selected_course = Some(match self.selected_course {
                Some(i) => (i + 1).min(self.courses.len() - 1),
                None => 0,
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
            self.selected_evaluation = None;
        }

        pub fn next_evaluation(&mut self) {
            let Some(course) = self.current_course() else {
                return;
            };
            if course.evaluations.is_empty() {
                self.selected_evaluation = None;
                return;
            }
            let len = course.evaluations.len();
            self.selected_evaluation = Some(match self.selected_evaluation {
                Some(i) => (i + 1).min(len - 1),
                None => 0,
            });
        }

        pub fn previous_evaluation(&mut self) {
            let Some(course) = self.current_course() else {
                return;
            };
            if course.evaluations.is_empty() {
                self.selected_evaluation = None;
                return;
            }
            self.selected_evaluation = Some(match self.selected_evaluation {
                Some(i) => i.saturating_sub(1),
                None => 0,
            });
        }

        pub fn toggle_focus(&mut self) {
            self.focus = match self.focus {
                Focus::Courses => Focus::Evaluations,
                Focus::Evaluations => Focus::Courses,
            };
        }

        // Form handling
        pub fn start_new_course(&mut self) {
            self.screen = Screen::EditingCourse { is_new: true };
            self.input_field = InputField::Name;
            self.edit_name.clear();
            self.edit_passing_grade = "4.0".to_string();
        }

        pub fn start_edit_course(&mut self) {
            let Some(idx) = self.selected_course else {
                return;
            };
            let Some(course) = self.courses.get(idx) else {
                return;
            };

            let name = course.name.clone();
            let passing_grade = format!("{:.1}", course.passing_grade);

            self.screen = Screen::EditingCourse { is_new: false };
            self.input_field = InputField::Name;
            self.edit_name = name;
            self.edit_passing_grade = passing_grade;
        }

        pub fn start_new_evaluation(&mut self) {
            if self.current_course().is_some() {
                self.screen = Screen::EditingEvaluation { is_new: true };
                self.input_field = InputField::Name;
                self.edit_name.clear();
                self.edit_weight = "20.0".to_string();
                self.edit_grade.clear();
            }
        }

        pub fn start_edit_evaluation(&mut self) {
            let Some(eval) = self.current_evaluation().cloned() else {
                return;
            };

            self.screen = Screen::EditingEvaluation { is_new: false };
            self.input_field = InputField::Name;
            self.edit_name = eval.name;
            self.edit_weight = format!("{:.1}", eval.weight);
            self.edit_grade = eval.grade.map(|g| format!("{:.1}", g)).unwrap_or_default();
        }

        pub fn confirm_course(&mut self) {
            let name = self.edit_name.trim().to_string();
            let passing_grade: f64 = self.edit_passing_grade.parse().unwrap_or(4.0);

            if name.is_empty() {
                return;
            }

            match self.screen {
                Screen::EditingCourse { is_new: true } => {
                    let course = Course::new(name, passing_grade);
                    self.courses.push(course);
                    self.selected_course = Some(self.courses.len() - 1);
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
            let _ = self.save();
        }

        pub fn confirm_evaluation(&mut self) {
            let name = self.edit_name.trim().to_string();
            let weight: f64 = self.edit_weight.parse().unwrap_or(20.0);
            let grade: Option<f64> = self.edit_grade.parse().ok();

            if name.is_empty() {
                return;
            }

            let selected_eval = self.selected_evaluation;

            match self.screen {
                Screen::EditingEvaluation { is_new: true } => {
                    if let Some(idx) = self.selected_course
                        && let Some(course) = self.courses.get_mut(idx)
                    {
                        let mut eval = Evaluation::new(name, weight);
                        eval.grade = grade;
                        course.evaluations.push(eval);
                        self.selected_evaluation = Some(course.evaluations.len() - 1);
                    }
                }
                Screen::EditingEvaluation { is_new: false } => {
                    if let Some(course_idx) = self.selected_course
                        && let Some(eval_idx) = selected_eval
                        && let Some(course) = self.courses.get_mut(course_idx)
                        && let Some(eval) = course.evaluations.get_mut(eval_idx)
                    {
                        eval.name = name;
                        eval.weight = weight;
                        eval.grade = grade;
                    }
                }
                _ => {}
            }

            self.screen = Screen::Main;
            let _ = self.save();
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
                        self.selected_evaluation = None;
                    }
                }
                Focus::Evaluations => {
                    let course_idx = self.selected_course;
                    let eval_idx = self.selected_evaluation;

                    if let Some(ci) = course_idx
                        && let Some(ei) = eval_idx
                        && let Some(course) = self.courses.get_mut(ci)
                    {
                        course.evaluations.remove(ei);
                        self.selected_evaluation = if course.evaluations.is_empty() {
                            None
                        } else {
                            Some(ei.min(course.evaluations.len() - 1))
                        };
                    }
                }
            }
            self.screen = Screen::Main;
            let _ = self.save();
        }

        pub fn next_input_field(&mut self) {
            self.input_field = match (&self.screen, &self.input_field) {
                (Screen::EditingCourse { .. }, InputField::Name) => InputField::PassingGrade,
                (Screen::EditingCourse { .. }, InputField::PassingGrade) => InputField::Name,
                (Screen::EditingEvaluation { .. }, InputField::Name) => InputField::Weight,
                (Screen::EditingEvaluation { .. }, InputField::Weight) => InputField::Grade,
                (Screen::EditingEvaluation { .. }, InputField::Grade) => InputField::Name,
                _ => self.input_field,
            };
        }

        pub fn current_input_buffer(&mut self) -> &mut String {
            match self.input_field {
                InputField::Name => &mut self.edit_name,
                InputField::PassingGrade => &mut self.edit_passing_grade,
                InputField::Weight => &mut self.edit_weight,
                InputField::Grade => &mut self.edit_grade,
            }
        }
    }
}

mod persistence {
    //! Data persistence using JSON and XDG directories.

    use crate::model::Course;
    use color_eyre::eyre::{Context, Result};
    use directories::ProjectDirs;
    use std::fs;
    use std::path::PathBuf;

    /// Get the data file path.
    fn data_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "kalk").map(|dirs| dirs.data_dir().join("data.json"))
    }

    /// Load courses from disk.
    pub fn load_data() -> Result<Vec<Course>> {
        let Some(path) = data_path() else {
            return Ok(Vec::new());
        };

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path).context("Failed to read data file")?;

        let courses: Vec<Course> =
            serde_json::from_str(&content).context("Failed to parse data")?;

        Ok(courses)
    }

    /// Save courses to disk.
    pub fn save_data(courses: &[Course]) -> Result<()> {
        let Some(path) = data_path() else {
            return Ok(());
        };

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create data directory")?;
        }

        let content = serde_json::to_string_pretty(courses).context("Failed to serialize data")?;

        fs::write(&path, content).context("Failed to write data file")?;

        Ok(())
    }
}

mod ui {
    //! User interface rendering.

    use crate::state::{App, Focus, InputField, Screen};
    use ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{
            Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
        },
    };

    /// Main UI rendering function.
    pub fn draw(frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(frame.size());

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[0]);

        draw_courses_list(frame, app, main_chunks[0]);
        draw_details_panel(frame, app, main_chunks[1]);
        draw_footer(frame, app, chunks[1]);

        // Draw popups on top
        match &app.screen {
            Screen::EditingCourse { is_new } => {
                draw_course_popup(frame, app, *is_new);
            }
            Screen::EditingEvaluation { is_new } => {
                draw_evaluation_popup(frame, app, *is_new);
            }
            Screen::ConfirmDelete => {
                draw_delete_popup(frame, app);
            }
            Screen::Main => {}
        }
    }

    fn draw_courses_list(frame: &mut Frame, app: &App, area: Rect) {
        let is_focused = app.focus == Focus::Courses;
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items: Vec<ListItem> = app
            .courses
            .iter()
            .map(|c| {
                let status = c.calculate_required_grade();
                ListItem::new(vec![
                    Line::from(Span::styled(
                        &c.name,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {}", status),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Courses ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(app.selected_course);

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn draw_details_panel(frame: &mut Frame, app: &App, area: Rect) {
        let is_focused = app.focus == Focus::Evaluations;
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let Some(course) = app.current_course() else {
            let block = Block::default()
                .title(" Evaluations ")
                .borders(Borders::ALL)
                .border_style(border_style);
            let paragraph = Paragraph::new("Select or create a course to view its evaluations.")
                .block(block)
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, area);
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        // Header with course info
        let header_block = Block::default()
            .title(format!(" {} ", course.name))
            .borders(Borders::ALL)
            .border_style(border_style);

        let header_text = format!(
            "Passing grade: {:.1} | {}",
            course.passing_grade,
            course.calculate_required_grade()
        );

        let header = Paragraph::new(header_text)
            .block(header_block)
            .wrap(Wrap { trim: true });

        frame.render_widget(header, chunks[0]);

        // Evaluations table
        let header_cells = ["Name", "Weight %", "Grade"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header_row = Row::new(header_cells).height(1);

        let rows: Vec<Row> = course
            .evaluations
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let is_selected = app.selected_evaluation == Some(i) && is_focused;
                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let grade_str = e
                    .grade
                    .map(|g| format!("{:.1}", g))
                    .unwrap_or_else(|| "-".to_string());

                Row::new(vec![
                    Cell::from(e.name.clone()),
                    Cell::from(format!("{:.1}", e.weight)),
                    Cell::from(grade_str),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(header_row)
        .block(
            Block::default()
                .title(" Evaluations ")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        );

        frame.render_widget(table, chunks[1]);
    }

    fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
        let keys = match &app.screen {
            Screen::Main => match app.focus {
                Focus::Courses => {
                    "q: Quit | n: New Course | Tab: Switch focus | Enter: Edit | d: Delete"
                }
                Focus::Evaluations => {
                    "q: Quit | n: New Eval | Tab: Switch focus | Enter: Edit | d: Delete"
                }
            },
            Screen::EditingCourse { .. } | Screen::EditingEvaluation { .. } => {
                "Tab: Next field | Enter: Confirm | Esc: Cancel"
            }
            Screen::ConfirmDelete => "Enter: Confirm | Esc: Cancel",
        };

        let footer = Paragraph::new(keys)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(footer, area);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    fn draw_course_popup(frame: &mut Frame, app: &App, is_new: bool) {
        let area = centered_rect(50, 40, frame.size());
        frame.render_widget(Clear, area);

        let title = if is_new {
            " New Course "
        } else {
            " Edit Course "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(block, area);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area);

        let name_style = if app.input_field == InputField::Name {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let name_input = Paragraph::new(app.edit_name.as_str())
            .style(name_style)
            .block(
                Block::default()
                    .title("Name")
                    .borders(Borders::ALL)
                    .border_style(name_style),
            );

        let grade_style = if app.input_field == InputField::PassingGrade {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let grade_input = Paragraph::new(app.edit_passing_grade.as_str())
            .style(grade_style)
            .block(
                Block::default()
                    .title("Passing Grade")
                    .borders(Borders::ALL)
                    .border_style(grade_style),
            );

        frame.render_widget(name_input, inner[0]);
        frame.render_widget(grade_input, inner[2]);
    }

    fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool) {
        let area = centered_rect(50, 50, frame.size());
        frame.render_widget(Clear, area);

        let title = if is_new {
            " New Evaluation "
        } else {
            " Edit Evaluation "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(block, area);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area);

        let fields = [
            ("Name", &app.edit_name, InputField::Name),
            ("Weight %", &app.edit_weight, InputField::Weight),
            ("Grade (optional)", &app.edit_grade, InputField::Grade),
        ];

        for (i, (label, value, field)) in fields.iter().enumerate() {
            let style = if app.input_field == *field {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let input = Paragraph::new(value.as_str()).style(style).block(
                Block::default()
                    .title(*label)
                    .borders(Borders::ALL)
                    .border_style(style),
            );

            frame.render_widget(input, inner[i * 2]);
        }
    }

    fn draw_delete_popup(frame: &mut Frame, app: &App) {
        let area = centered_rect(40, 20, frame.size());
        frame.render_widget(Clear, area);

        let message = match app.focus {
            Focus::Courses => "Delete this course?",
            Focus::Evaluations => "Delete this evaluation?",
        };

        let block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let paragraph = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

mod event {
    //! Event handling.

    use crate::state::{App, Focus, Screen};
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use std::time::Duration;

    /// Handle events and update application state.
    pub fn handle_events(app: &mut App) -> color_eyre::Result<bool> {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Only handle key press events, not release
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            match &app.screen {
                Screen::Main => handle_main_keys(app, key.code),
                Screen::EditingCourse { .. } => handle_edit_course_keys(app, key.code),
                Screen::EditingEvaluation { .. } => handle_edit_evaluation_keys(app, key.code),
                Screen::ConfirmDelete => handle_delete_keys(app, key.code),
            }
        }
        Ok(app.should_quit)
    }

    fn handle_main_keys(app: &mut App, key: KeyCode) {
        match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Tab => app.toggle_focus(),
            KeyCode::Char('n') => match app.focus {
                Focus::Courses => app.start_new_course(),
                Focus::Evaluations => app.start_new_evaluation(),
            },
            KeyCode::Enter => match app.focus {
                Focus::Courses => app.start_edit_course(),
                Focus::Evaluations => app.start_edit_evaluation(),
            },
            KeyCode::Char('d') => {
                let can_delete = match app.focus {
                    Focus::Courses => app.selected_course.is_some(),
                    Focus::Evaluations => app.selected_evaluation.is_some(),
                };
                if can_delete {
                    app.screen = Screen::ConfirmDelete;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => match app.focus {
                Focus::Courses => app.previous_course(),
                Focus::Evaluations => app.previous_evaluation(),
            },
            KeyCode::Down | KeyCode::Char('j') => match app.focus {
                Focus::Courses => app.next_course(),
                Focus::Evaluations => app.next_evaluation(),
            },
            _ => {}
        }
    }

    fn handle_edit_course_keys(app: &mut App, key: KeyCode) {
        match key {
            KeyCode::Esc => app.screen = Screen::Main,
            KeyCode::Tab => app.next_input_field(),
            KeyCode::Enter => app.confirm_course(),
            KeyCode::Backspace => {
                app.current_input_buffer().pop();
            }
            KeyCode::Char(c) => {
                app.current_input_buffer().push(c);
            }
            _ => {}
        }
    }

    fn handle_edit_evaluation_keys(app: &mut App, key: KeyCode) {
        match key {
            KeyCode::Esc => app.screen = Screen::Main,
            KeyCode::Tab => app.next_input_field(),
            KeyCode::Enter => app.confirm_evaluation(),
            KeyCode::Backspace => {
                app.current_input_buffer().pop();
            }
            KeyCode::Char(c) => {
                app.current_input_buffer().push(c);
            }
            _ => {}
        }
    }

    fn handle_delete_keys(app: &mut App, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Char('y') => app.delete_current(),
            KeyCode::Esc | KeyCode::Char('n') => app.screen = Screen::Main,
            _ => {}
        }
    }
}

// =============================================================================
// MAIN APPLICATION
// =============================================================================

use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};

use state::App;

/// Initialize panic hooks to restore terminal on panic.
fn init_panic_hook() -> Result<()> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal before printing panic
        let _ = restore_terminal();
        hook(panic_info);
    }));

    color_eyre::install()?;
    Ok(())
}

/// Setup terminal for TUI.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal state.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Main application loop.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::handle_events(app)? {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // Initialize panic hooks first
    init_panic_hook()?;

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Load application state
    let mut app = App::load();

    // Run the application
    let result = run_app(&mut terminal, &mut app);

    // Always restore terminal
    restore_terminal()?;

    // Propagate any errors
    result
}
