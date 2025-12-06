//! User interface rendering using Ratatui.
//!
//! This module handles all visual rendering of the application,
//! including the main layout, panels, and popup dialogs.

use crate::app::{App, Focus, InputField, Screen};
use crate::model::{DEFAULT_PASSING_GRADE, WeightValidation};
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

    // Main area: 3-column layout for Course -> Category -> Evaluation hierarchy
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ])
        .split(chunks[0]);

    draw_courses_panel(frame, app, main_chunks[0]);
    draw_categories_panel(frame, app, main_chunks[1]);
    draw_evaluations_panel(frame, app, main_chunks[2]);
    draw_footer(frame, app, chunks[1]);

    // Draw popups on top
    match &app.screen {
        Screen::SelectingTemplate => {
            draw_template_popup(frame, app);
        }
        Screen::EditingCourse { is_new } => {
            draw_course_popup(frame, app, *is_new);
        }
        Screen::EditingCategory { is_new } => {
            draw_category_popup(frame, app, *is_new);
        }
        Screen::EditingEvaluation { is_new } => {
            draw_evaluation_popup(frame, app, *is_new);
        }
        Screen::ConfirmDelete => {
            draw_delete_popup(frame, app);
        }
        Screen::ConfirmDeleteTemplate => {
            draw_delete_template_popup(frame, app);
        }
        Screen::SavingTemplate => {
            draw_save_template_popup(frame, app);
        }
        Screen::Main => {}
    }
}

// =============================================================================
// Panel Drawing
// =============================================================================

fn draw_courses_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Courses;
    let border_style = focused_border_style(is_focused);

    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| {
            let status = c.calculate_required_grade();

            // Weight validation indicator
            let weight_status = match c.validate_weights() {
                WeightValidation::Valid => Span::styled(" [OK]", Style::default().fg(Color::Green)),
                WeightValidation::Under(w) => {
                    Span::styled(format!(" [{:.0}%]", w), Style::default().fg(Color::Yellow))
                }
                WeightValidation::Over(w) => {
                    Span::styled(format!(" [{:.0}%!]", w), Style::default().fg(Color::Red))
                }
                WeightValidation::Empty => {
                    Span::styled(" [Empty]", Style::default().fg(Color::DarkGray))
                }
            };

            // Current grade color
            let grade_color = match c.current_grade() {
                Some(g) if g >= c.passing_grade => Color::Green,
                Some(_) => Color::Red,
                None => Color::DarkGray,
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                    weight_status,
                ]),
                Line::from(Span::styled(
                    format!("  {}", status),
                    Style::default().fg(grade_color),
                )),
            ])
        })
        .collect();

    let title = format!(" Courses ({}) ", app.courses.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
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

fn draw_categories_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Categories;
    let border_style = focused_border_style(is_focused);

    let Some(course) = app.current_course() else {
        let block = Block::default()
            .title(" Categories ")
            .borders(Borders::ALL)
            .border_style(border_style);
        let paragraph = Paragraph::new("Select a course to view categories.")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    };

    // Split area for header info, course average, and category list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with weight info
            Constraint::Length(3), // Course average
            Constraint::Min(0),    // Category list
        ])
        .split(area);

    // Course info header with weight validation
    let validation = course.validate_weights();
    let validation_color = match &validation {
        WeightValidation::Valid => Color::Green,
        WeightValidation::Under(_) => Color::Yellow,
        WeightValidation::Over(_) => Color::Red,
        WeightValidation::Empty => Color::DarkGray,
    };

    let header_text = format!(
        "Pass: {:.0} | Total: {:.0}% | {}",
        course.passing_grade,
        course.total_weight(),
        validation.message()
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(validation_color))
        .block(
            Block::default()
                .title(format!(" {} ", course.name))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
    frame.render_widget(header, chunks[0]);

    // Course total average with pass/fail status
    let avg_text = course.current_grade_status();
    let avg_color = match course.current_grade() {
        Some(g) if course.is_passing_grade(g) => Color::Green,
        Some(_) => Color::Red,
        None => Color::DarkGray,
    };
    let avg_block = Block::default()
        .title(" Course Average ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(avg_color));
    let avg_widget = Paragraph::new(avg_text)
        .style(Style::default().fg(avg_color))
        .block(avg_block);
    frame.render_widget(avg_widget, chunks[1]);

    // Category list
    let items: Vec<ListItem> = course
        .categories
        .iter()
        .map(|cat| {
            let avg = cat
                .average_grade()
                .map(|g| format!("{:.1}", g))
                .unwrap_or_else(|| "-".to_string());

            let progress = format!("{}/{}", cat.graded_count(), cat.evaluations.len());

            // Color based on passing status
            let avg_color = match cat.is_passing() {
                Some(true) => Color::Green,
                Some(false) => Color::Red,
                None => Color::DarkGray,
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&cat.name, Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!(" ({:.0}%)", cat.weight),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Avg: "),
                    Span::styled(avg, Style::default().fg(avg_color)),
                    Span::styled(
                        format!(" | {}", progress),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ])
        })
        .collect();

    let title = format!(" Categories ({}) ", course.categories.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
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
    state.select(app.selected_category);

    frame.render_stateful_widget(list, chunks[2], &mut state);
}

fn draw_evaluations_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Evaluations;
    let border_style = focused_border_style(is_focused);

    let Some(category) = app.current_category() else {
        let block = Block::default()
            .title(" Evaluations ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let message = if app.current_course().is_some() {
            "Select a category to view evaluations."
        } else {
            "Select a course first."
        };

        let paragraph = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Header with category info
    let avg = category
        .average_grade()
        .map(|g| format!("{:.1}", g))
        .unwrap_or_else(|| "-".to_string());

    let avg_color = match category.is_passing() {
        Some(true) => Color::Green,
        Some(false) => Color::Red,
        None => Color::DarkGray,
    };

    let header_text = Line::from(vec![
        Span::raw(format!(
            "{} ({:.0}%) | Avg: ",
            category.name, category.weight
        )),
        Span::styled(avg, Style::default().fg(avg_color)),
        Span::raw(format!(
            " | {}/{} graded",
            category.graded_count(),
            category.evaluations.len()
        )),
    ]);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Evaluations table
    let header_cells = ["#", "Name", "Grade"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = category
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
                .map(|g| format!("{:.0}", g))
                .unwrap_or_else(|| "-".to_string());

            let grade_style = match e.grade {
                Some(g) if g >= DEFAULT_PASSING_GRADE => Style::default().fg(Color::Green),
                Some(_) => Style::default().fg(Color::Red),
                None => Style::default().fg(Color::DarkGray),
            };

            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(e.name.as_str()),
                Cell::from(grade_str).style(grade_style),
            ])
            .style(style)
        })
        .collect();

    let title = format!(" Evaluations ({}) ", category.evaluations.len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(7),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(table, chunks[1]);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keys = match &app.screen {
        Screen::Main => match app.focus {
            Focus::Courses => {
                "q: Quit | n: New | Enter: Edit | d: Delete | t: Save as Template | b: Balance"
            }
            Focus::Categories => {
                "q: Quit | n: New Category | Enter: Edit | d: Delete | Tab/Arrows: Navigate"
            }
            Focus::Evaluations => {
                "q: Quit | n: New Eval | Enter: Edit | d: Delete | Tab/Arrows: Navigate"
            }
        },
        Screen::SelectingTemplate => {
            if app.is_user_template_selected() {
                "Up/Down/Tab: Select | Enter: Confirm | d: Delete | Esc: Cancel"
            } else {
                "Up/Down/Tab: Select | Enter: Confirm | Esc: Cancel"
            }
        }
        Screen::EditingCourse { .. }
        | Screen::EditingCategory { .. }
        | Screen::EditingEvaluation { .. }
        | Screen::SavingTemplate => "Tab: Next field | Enter: Confirm | Esc: Cancel",
        Screen::ConfirmDelete | Screen::ConfirmDeleteTemplate => "Enter/y: Confirm | Esc/n: Cancel",
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

// =============================================================================
// Popup Drawing
// =============================================================================

fn draw_template_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Course Template ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Combine built-in and user templates
    let all_templates = app.all_templates();
    let built_in_count = app.built_in_templates.len();

    let items: Vec<ListItem> = all_templates
        .iter()
        .enumerate()
        .map(|(i, t)| {
            // Mark user templates with a different style
            let is_user_template = i >= built_in_count;
            let prefix = if is_user_template { "★ " } else { "" };

            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("{}{}", prefix, t.name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {}", t.description),
                    Style::default().fg(if is_user_template {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_template));

    frame.render_stateful_widget(list, inner, &mut state);
}

fn draw_course_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let area = centered_rect(50, 35, frame.size());
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

    render_input_field(
        frame,
        "Name",
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        "Passing Grade (0-100)",
        &app.edit_passing_grade,
        app.input_field == InputField::PassingGrade,
        inner[2],
    );
}

fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let area = centered_rect(50, 40, frame.size());
    frame.render_widget(Clear, area);

    let title = if is_new {
        " New Category "
    } else {
        " Edit Category "
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
            Constraint::Length(2),
        ])
        .split(area);

    render_input_field(
        frame,
        "Name (e.g., Certamenes, Controles)",
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        "Weight % (e.g., 80)",
        &app.edit_weight,
        app.input_field == InputField::Weight,
        inner[2],
    );

    // Show current weight status
    if let Some(course) = app.current_course() {
        let current_total = course.total_weight();
        let hint = format!("Current total: {:.0}%", current_total);
        let hint_color = if current_total > 100.0 {
            Color::Red
        } else if current_total < 100.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        let hint_widget = Paragraph::new(hint).style(Style::default().fg(hint_color));
        frame.render_widget(hint_widget, inner[3]);
    }
}

fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let area = centered_rect(60, 50, frame.size());
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
            Constraint::Length(3), // Grade field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Required grade info
        ])
        .split(area);

    // Grade field FIRST
    render_input_field(
        frame,
        "Grade (0-100)",
        &app.edit_grade,
        app.input_field == InputField::Grade,
        inner[0],
    );

    // Name field second
    render_input_field(
        frame,
        "Name",
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[2],
    );

    // Show what grade is needed in this evaluation to pass
    if let Some(course) = app.current_course() {
        let info = if let (Some(cat_idx), Some(eval_idx)) =
            (app.selected_category, app.selected_evaluation)
        {
            course.calculate_needed_for_evaluation(cat_idx, eval_idx)
        } else {
            course.calculate_required_grade()
        };

        let info_color =
            if info.contains("impossible") || info.contains("below") || info.contains("FAILED") {
                Color::Red
            } else if info.contains("passing") || info.contains("0+") || info.contains("PASSED") {
                Color::Green
            } else {
                Color::Yellow
            };

        let info_block = Block::default()
            .title(" Need to Pass ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(info_color));
        let info_widget = Paragraph::new(info)
            .style(Style::default().fg(info_color))
            .block(info_block);
        frame.render_widget(info_widget, inner[4]);
    }
}

fn draw_save_template_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Save as Template ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Description field
            Constraint::Length(1), // Spacing
            Constraint::Length(2), // Info text
        ])
        .split(area);

    render_input_field(
        frame,
        "Template Name",
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        "Description",
        &app.edit_description,
        app.input_field == InputField::Description,
        inner[2],
    );

    // Show info about what will be saved
    if let Some(course) = app.current_course() {
        let info = format!(
            "Will save {} categories from '{}'",
            course.categories.len(),
            course.name
        );
        let info_widget = Paragraph::new(info).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(info_widget, inner[4]);
    }
}

fn draw_delete_template_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, frame.size());
    frame.render_widget(Clear, area);

    let template_name = app
        .current_template()
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    let block = Block::default()
        .title(" Delete Template ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            format!("Delete '{}'?", template_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(45, 25, frame.size());
    frame.render_widget(Clear, area);

    let (message, warning) = match app.focus {
        Focus::Courses => (
            "Delete this course?",
            "All categories and evaluations will be lost.",
        ),
        Focus::Categories => (
            "Delete this category?",
            "All evaluations in this category will be lost.",
        ),
        Focus::Evaluations => ("Delete this evaluation?", "This action cannot be undone."),
    };

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            message,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(warning, Style::default().fg(Color::Yellow))),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn focused_border_style(is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
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

fn render_input_field(frame: &mut Frame, label: &str, value: &str, is_active: bool, area: Rect) {
    let style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Show cursor indicator when active
    let display_value = if is_active {
        format!("{}|", value)
    } else {
        value.to_string()
    };

    let input = Paragraph::new(display_value).style(style).block(
        Block::default()
            .title(label)
            .borders(Borders::ALL)
            .border_style(style),
    );

    frame.render_widget(input, area);
}
