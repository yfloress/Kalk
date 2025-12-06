//! User interface rendering using Ratatui.
//!
//! This module handles all visual rendering of the application,
//! including the main layout, panels, and popup dialogs.

use crate::app::{App, Focus, InputField, Screen};
use crate::i18n::Language;
use crate::model::{DEFAULT_PASSING_GRADE, MAX_GRADE, WeightValidation};
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
            draw_evaluation_popup(frame, app, *is_new, !*is_new);
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
        Screen::SelectingLanguage => {
            draw_language_popup(frame, app);
        }
        Screen::Main => {}
    }
}

// =============================================================================
// Panel Drawing
// =============================================================================

fn draw_courses_panel(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let is_focused = app.focus == Focus::Courses;
    let border_style = focused_border_style(is_focused);

    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| {
            let status = format_course_status(c, m);

            // Weight validation indicator
            let weight_status = match c.validate_weights() {
                WeightValidation::Valid => Span::styled(" [OK]", Style::default().fg(Color::Green)),
                WeightValidation::Under(w) => {
                    Span::styled(format!(" [{:.0}%]", w), Style::default().fg(Color::Yellow))
                }
                WeightValidation::Over(w) => {
                    Span::styled(format!(" [{:.0}%!]", w), Style::default().fg(Color::Red))
                }
                WeightValidation::Empty => Span::styled(
                    format!(" [{}]", m.no_categories),
                    Style::default().fg(Color::DarkGray),
                ),
            };

            // Current grade color
            let grade_color = match c.current_grade() {
                Some(g) if c.is_passing_grade(g) => Color::Green,
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

    let title = format!(" {} ({}) ", m.courses, app.courses.len());
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
    let m = app.messages();
    let is_focused = app.focus == Focus::Categories;
    let border_style = focused_border_style(is_focused);

    let Some(course) = app.current_course() else {
        let block = Block::default()
            .title(format!(" {} ", m.categories))
            .borders(Borders::ALL)
            .border_style(border_style);
        let paragraph = Paragraph::new(m.select_course_to_view)
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

    let validation_msg = format_weight_validation(&validation, m);
    let header_text = format!(
        "{}: {:.0} | Total: {:.0}% | {}",
        m.passing_grade.split(' ').next().unwrap_or("Pass"),
        course.passing_grade,
        course.total_weight(),
        validation_msg
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
    let avg_text = format_grade_status(course, m);
    let avg_color = match course.current_grade() {
        Some(g) if course.is_passing_grade(g) => Color::Green,
        Some(_) => Color::Red,
        None => Color::DarkGray,
    };
    let avg_block = Block::default()
        .title(format!(" {} ", m.course_average))
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

            let progress = format!(
                "{}/{} {}",
                cat.graded_count(),
                cat.evaluations.len(),
                m.graded
            );

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
                    Span::raw(format!("  {}: ", m.avg)),
                    Span::styled(avg, Style::default().fg(avg_color)),
                    Span::styled(
                        format!(" | {}", progress),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ])
        })
        .collect();

    let title = format!(" {} ({}) ", m.categories, course.categories.len());
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
    let m = app.messages();
    let is_focused = app.focus == Focus::Evaluations;
    let border_style = focused_border_style(is_focused);

    let Some(category) = app.current_category() else {
        let block = Block::default()
            .title(format!(" {} ", m.evaluations))
            .borders(Borders::ALL)
            .border_style(border_style);

        let message = if app.current_course().is_some() {
            m.select_category_to_view
        } else {
            m.select_course_first
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
            "{} ({:.0}%) | {}: ",
            category.name, category.weight, m.avg
        )),
        Span::styled(avg, Style::default().fg(avg_color)),
        Span::raw(format!(
            " | {}/{} {}",
            category.graded_count(),
            category.evaluations.len(),
            m.graded
        )),
    ]);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(header, chunks[0]);

    // Evaluations table
    let header_labels = ["#", m.name, m.grade];
    let header_cells = header_labels
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

    let title = format!(" {} ({}) ", m.evaluations, category.evaluations.len());
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
    let m = app.messages();
    let keys = match &app.screen {
        Screen::Main => match app.focus {
            Focus::Courses => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | t: {} | b: {} | Ctrl+L: {}",
                    m.quit,
                    m.new,
                    m.edit,
                    m.delete,
                    m.save_as_template,
                    m.balance,
                    m.change_language
                )
            }
            Focus::Categories => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | Ctrl+L: {}",
                    m.quit, m.new_category, m.edit, m.delete, m.change_language
                )
            }
            Focus::Evaluations => {
                format!(
                    "q: {} | n: {} | Enter: {} | d: {} | Ctrl+L: {}",
                    m.quit, m.new_eval, m.edit, m.delete, m.change_language
                )
            }
        },
        Screen::SelectingTemplate => {
            if app.is_user_template_selected() {
                format!(
                    "↑/↓/Tab: {} | Enter: {} | d: {} | Esc: {}",
                    m.select, m.confirm, m.delete, m.cancel
                )
            } else {
                format!(
                    "↑/↓/Tab: {} | Enter: {} | Esc: {}",
                    m.select, m.confirm, m.cancel
                )
            }
        }
        Screen::SelectingLanguage => {
            format!(
                "↑/↓/Tab: {} | Enter: {} | Esc: {}",
                m.select, m.confirm, m.cancel
            )
        }
        Screen::EditingCourse { .. }
        | Screen::EditingCategory { .. }
        | Screen::EditingEvaluation { .. }
        | Screen::SavingTemplate => {
            format!(
                "Tab: {} | Enter: {} | Esc: {}",
                m.next_field, m.confirm, m.cancel
            )
        }
        Screen::ConfirmDelete | Screen::ConfirmDeleteTemplate => {
            format!("Enter/y: {} | Esc/n: {}", m.confirm, m.cancel)
        }
    };

    let footer = Paragraph::new(keys.to_string())
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
    let m = app.messages();
    let area = centered_rect(60, 70, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.select_course_template))
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
    let m = app.messages();
    let area = centered_rect(50, 35, frame.size());
    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_course)
    } else {
        format!(" {} ", m.edit_course)
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
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        m.passing_grade,
        &app.edit_passing_grade,
        app.input_field == InputField::PassingGrade,
        inner[2],
    );
}

fn draw_category_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    let area = centered_rect(50, 40, frame.size());
    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_category_title)
    } else {
        format!(" {} ", m.edit_category)
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
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        m.weight,
        &app.edit_weight,
        app.input_field == InputField::Weight,
        inner[2],
    );

    // Show current weight status
    if let Some(course) = app.current_course() {
        let current_total = course.total_weight();
        let hint = format!("{}: {:.0}%", m.current_total, current_total);
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

fn draw_evaluation_popup(frame: &mut Frame, app: &App, is_new: bool, is_editing: bool) {
    let m = app.messages();
    let area = centered_rect(60, 50, frame.size());
    frame.render_widget(Clear, area);

    let title = if is_new {
        format!(" {} ", m.new_evaluation)
    } else {
        format!(" {} ", m.edit_evaluation)
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
        m.grade,
        &app.edit_grade,
        app.input_field == InputField::Grade,
        inner[0],
    );

    // Name field second
    render_input_field(
        frame,
        m.name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[2],
    );

    // Show what grade is needed in this evaluation to pass
    if let Some(course) = app.current_course() {
        let (info, info_color) = if let (Some(cat_idx), Some(eval_idx)) =
            (app.selected_category, app.selected_evaluation)
        {
            let needed = format_needed_for_evaluation(course, cat_idx, eval_idx, m, is_editing);
            let color = match needed.status {
                NeedGradeStatus::Success => Color::Green,
                NeedGradeStatus::Failure => Color::Red,
                NeedGradeStatus::Warning => Color::Yellow,
                NeedGradeStatus::Info => Color::DarkGray,
            };
            (needed.message, color)
        } else {
            let text = format_course_status(course, m);
            let color = match course.current_grade() {
                Some(g) if course.is_passing_grade(g) => Color::Green,
                Some(_) => Color::Red,
                None => Color::DarkGray,
            };
            (text, color)
        };

        let info_block = Block::default()
            .title(format!(" {} ", m.need_to_pass))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(info_color));
        let info_widget = Paragraph::new(info)
            .style(Style::default().fg(info_color))
            .block(info_block);
        frame.render_widget(info_widget, inner[4]);
    }
}

fn draw_save_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.save_as_template_title))
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
        m.template_name,
        &app.edit_name,
        app.input_field == InputField::Name,
        inner[0],
    );

    render_input_field(
        frame,
        m.description,
        &app.edit_description,
        app.input_field == InputField::Description,
        inner[2],
    );

    // Show info about what will be saved
    if let Some(course) = app.current_course() {
        let info = format!(
            "{} {} '{}' ",
            m.will_save_categories,
            course.categories.len(),
            course.name
        );
        let info_widget = Paragraph::new(info).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(info_widget, inner[4]);
    }
}

fn draw_delete_template_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(50, 25, frame.size());
    frame.render_widget(Clear, area);

    let template_name = app
        .current_template()
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    let block = Block::default()
        .title(format!(" {} ", m.delete_template))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            format!("{} '{}'?", m.delete_template_question, template_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            m.action_cannot_be_undone,
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(45, 25, frame.size());
    frame.render_widget(Clear, area);

    let (message, warning) = match app.focus {
        Focus::Courses => (m.delete_course_question, m.delete_course_warning),
        Focus::Categories => (m.delete_category_question, m.delete_category_warning),
        Focus::Evaluations => (m.delete_evaluation_question, m.delete_evaluation_warning),
    };

    let block = Block::default()
        .title(format!(" {} ", m.confirm_delete))
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

fn draw_language_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let area = centered_rect(40, 30, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", m.select_language))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = Language::all()
        .iter()
        .map(|lang| {
            let is_current = *lang == app.language;
            let prefix = if is_current { "● " } else { "  " };
            let style = if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, lang.display_name()),
                style,
            )))
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
    state.select(Some(app.selected_language));

    frame.render_stateful_widget(list, inner, &mut state);
}

// =============================================================================
// Formatting Helper Functions (i18n)
// =============================================================================

use crate::i18n::Messages;
use crate::model::Course;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeedGradeStatus {
    Success,
    Failure,
    Warning,
    Info,
}

struct NeededGrade {
    message: String,
    status: NeedGradeStatus,
}

/// Format course status message with translations.
fn format_course_status(course: &Course, m: &Messages) -> String {
    // Check if there are any evaluations at all
    let has_evaluations = course.categories.iter().any(|c| !c.evaluations.is_empty());

    if !has_evaluations {
        return m.no_evaluations.to_string();
    }

    match course.current_grade() {
        Some(grade) => {
            let rounded = Course::round_grade(grade);
            if course.is_passing_grade(grade) {
                format!(
                    "{}: {:.1} → {:.0} ({})",
                    m.current, grade, rounded, m.passed
                )
            } else {
                format!(
                    "{}: {:.1} → {:.0} ({})",
                    m.current, grade, rounded, m.failed
                )
            }
        }
        None => m.no_evaluations.to_string(),
    }
}

/// Format weight validation message with translations.
fn format_weight_validation(validation: &WeightValidation, m: &Messages) -> String {
    match validation {
        WeightValidation::Valid => m.weights_ok.to_string(),
        WeightValidation::Under(total) => format!("{} {:.1}%", m.weights_warning, total),
        WeightValidation::Over(total) => format!("{} {:.1}%", m.weights_error, total),
        WeightValidation::Empty => m.no_categories.to_string(),
    }
}

/// Format current grade status with translations.
fn format_grade_status(course: &Course, m: &Messages) -> String {
    match course.current_grade() {
        Some(grade) => {
            let rounded = Course::round_grade(grade);
            if course.is_passing_grade(grade) {
                format!(
                    "{}: {:.1} → {:.0} ({})",
                    m.current, grade, rounded, m.passed
                )
            } else {
                format!(
                    "{}: {:.1} → {:.0} ({})",
                    m.current, grade, rounded, m.failed
                )
            }
        }
        None => m.no_grades_yet.to_string(),
    }
}

/// Format needed grade for evaluation with translations.
fn format_needed_for_evaluation(
    course: &Course,
    category_idx: usize,
    eval_idx: usize,
    m: &Messages,
    is_editing: bool,
) -> NeededGrade {
    let Some(category) = course.categories.get(category_idx) else {
        return NeededGrade {
            message: m.invalid_category.to_string(),
            status: NeedGradeStatus::Failure,
        };
    };

    let Some(eval) = category.evaluations.get(eval_idx) else {
        return NeededGrade {
            message: m.invalid_evaluation.to_string(),
            status: NeedGradeStatus::Failure,
        };
    };

    if category.evaluations.is_empty() {
        return NeededGrade {
            message: m.no_evaluations.to_string(),
            status: NeedGradeStatus::Info,
        };
    }

    // If already graded and NOT editing, show that info
    // When editing, we ignore the current grade to show what's needed
    if !is_editing {
        if let Some(grade) = eval.grade {
            if grade >= course.passing_grade {
                return NeededGrade {
                    message: format!("{}: {:.0} ({})", eval.name, grade, m.passing),
                    status: NeedGradeStatus::Success,
                };
            }
            return NeededGrade {
                message: format!("{}: {:.0} ({})", eval.name, grade, m.below_passing),
                status: NeedGradeStatus::Failure,
            };
        }
    }

    // Categories with zero weight cannot change the course outcome
    if category.weight.abs() < f64::EPSILON {
        return NeededGrade {
            message: m.cannot_pass.to_string(),
            status: NeedGradeStatus::Failure,
        };
    }

    let eval_count = category.evaluations.len() as f64;
    if eval_count == 0.0 {
        return NeededGrade {
            message: m.no_evaluations.to_string(),
            status: NeedGradeStatus::Info,
        };
    }

    // Calculate total contribution from all categories, treating ungraded evals as 0
    // except for the current evaluation we're calculating for
    let mut total_contribution = 0.0;

    for (ci, cat) in course.categories.iter().enumerate() {
        if ci == category_idx {
            // For the current category, sum all other evaluations (ungraded = 0)
            // When editing, we also treat the current evaluation as 0
            let other_sum: f64 = cat
                .evaluations
                .iter()
                .enumerate()
                .map(|(ei, e)| {
                    if ei == eval_idx {
                        0.0 // Always treat current eval as 0 for calculation
                    } else {
                        e.grade.unwrap_or(0.0)
                    }
                })
                .sum();
            // This will be added later with the needed grade
            total_contribution += other_sum * cat.weight / (100.0 * cat.evaluations.len() as f64);
        } else {
            // For other categories, all ungraded evals count as 0
            if !cat.evaluations.is_empty() {
                let sum: f64 = cat.evaluations.iter().map(|e| e.grade.unwrap_or(0.0)).sum();
                let avg = sum / cat.evaluations.len() as f64;
                total_contribution += avg * cat.weight / 100.0;
            }
        }
    }

    // Weight of this single evaluation in the final grade
    let eval_weight = category.weight / (100.0 * eval_count);

    if eval_weight.abs() < f64::EPSILON {
        return NeededGrade {
            message: m.cannot_pass.to_string(),
            status: NeedGradeStatus::Failure,
        };
    }

    // We need: total_contribution + (needed_grade * eval_weight) >= 54.5
    // So: needed_grade = (54.5 - total_contribution) / eval_weight
    let effective_passing = course.passing_grade - 0.5; // 54.5
    let needed_grade = (effective_passing - total_contribution) / eval_weight;

    if !needed_grade.is_finite() {
        return NeededGrade {
            message: m.cannot_pass.to_string(),
            status: NeedGradeStatus::Failure,
        };
    }

    if needed_grade <= 0.0 {
        return NeededGrade {
            message: format!("{}: 0+ ({})", m.need, m.need_grade_any),
            status: NeedGradeStatus::Success,
        };
    } else if needed_grade > MAX_GRADE {
        let rounded_up = needed_grade.ceil() as i32;
        return NeededGrade {
            message: format!(
                "{}: {:.2} → {} ({})",
                m.need, needed_grade, rounded_up, m.need_grade_impossible
            ),
            status: NeedGradeStatus::Failure,
        };
    } else {
        let rounded_up = needed_grade.ceil() as i32;
        // Only show arrow if there are decimals
        let message = if (needed_grade - needed_grade.floor()).abs() < 0.01 {
            format!("{} {} {}", m.need, rounded_up, m.need_grade_in_eval)
        } else {
            format!(
                "{} {:.2} → {} {}",
                m.need, needed_grade, rounded_up, m.need_grade_in_eval
            )
        };

        NeededGrade {
            message,
            status: NeedGradeStatus::Warning,
        }
    }
}
