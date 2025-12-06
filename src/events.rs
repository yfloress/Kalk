//! Event handling for keyboard input.
//!
//! This module handles all keyboard events and dispatches them
//! to the appropriate handlers based on the current screen.

use crate::app::{App, Focus, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

/// Poll for events and handle them.
/// Returns true if the application should quit.
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
            Screen::SelectingTemplate => handle_template_keys(app, key.code),
            Screen::EditingCourse { .. } => handle_edit_course_keys(app, key.code),
            Screen::EditingCategory { .. } => handle_edit_category_keys(app, key.code),
            Screen::EditingEvaluation { .. } => handle_edit_evaluation_keys(app, key.code),
            Screen::ConfirmDelete => handle_delete_keys(app, key.code),
            Screen::ConfirmDeleteTemplate => handle_delete_template_keys(app, key.code),
            Screen::SavingTemplate => handle_save_template_keys(app, key.code),
        }
    }
    Ok(app.should_quit)
}

/// Handle keys in the main screen.
fn handle_main_keys(app: &mut App, key: KeyCode) {
    match key {
        // Quit
        KeyCode::Char('q') => app.should_quit = true,

        // Focus navigation
        KeyCode::Tab => app.cycle_focus(),
        KeyCode::Left | KeyCode::Char('h') => app.focus_left(),
        KeyCode::Right | KeyCode::Char('l') => app.focus_right(),

        // Item navigation (up/down within current focus)
        KeyCode::Up | KeyCode::Char('k') => match app.focus {
            Focus::Courses => app.previous_course(),
            Focus::Categories => app.previous_category(),
            Focus::Evaluations => app.previous_evaluation(),
        },
        KeyCode::Down | KeyCode::Char('j') => match app.focus {
            Focus::Courses => app.next_course(),
            Focus::Categories => app.next_category(),
            Focus::Evaluations => app.next_evaluation(),
        },

        // Create new item
        KeyCode::Char('n') => match app.focus {
            Focus::Courses => app.start_new_course(),
            Focus::Categories => app.start_new_category(),
            Focus::Evaluations => app.start_new_evaluation(),
        },

        // Edit selected item
        KeyCode::Enter => match app.focus {
            Focus::Courses => app.start_edit_course(),
            Focus::Categories => app.start_edit_category(),
            Focus::Evaluations => app.start_edit_evaluation(),
        },

        // Delete selected item
        KeyCode::Char('d') => app.request_delete(),

        // Auto-balance weights for current course
        KeyCode::Char('b') => {
            if app.current_course().is_some() {
                app.auto_balance_weights();
            }
        }

        // Save current course as template
        KeyCode::Char('t') => {
            if app.current_course().is_some() {
                app.start_save_as_template();
            }
        }

        _ => {}
    }
}

/// Handle keys in the template selection screen.
fn handle_template_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_template(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => app.next_template(),
        KeyCode::Enter => app.confirm_template_selection(),
        KeyCode::Char('d') => app.request_delete_template(),
        _ => {}
    }
}

/// Handle keys in the delete template confirmation screen.
fn handle_delete_template_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter | KeyCode::Char('y') => app.delete_current_template(),
        KeyCode::Esc | KeyCode::Char('n') => {
            app.screen = Screen::SelectingTemplate;
        }
        _ => {}
    }
}

/// Handle keys in the course editing screen.
fn handle_edit_course_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_edit(),
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

/// Handle keys in the category editing screen.
fn handle_edit_category_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Tab => app.next_input_field(),
        KeyCode::Enter => app.confirm_category(),
        KeyCode::Backspace => {
            app.current_input_buffer().pop();
        }
        KeyCode::Char(c) => {
            app.current_input_buffer().push(c);
        }
        _ => {}
    }
}

/// Handle keys in the evaluation editing screen.
fn handle_edit_evaluation_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_edit(),
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

/// Handle keys in the delete confirmation screen.
fn handle_delete_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter | KeyCode::Char('y') => app.delete_current(),
        KeyCode::Esc | KeyCode::Char('n') => app.cancel_edit(),
        _ => {}
    }
}

/// Handle keys in the save template screen.
fn handle_save_template_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Tab => app.next_input_field(),
        KeyCode::Enter => app.confirm_save_template(),
        KeyCode::Backspace => {
            app.current_input_buffer().pop();
        }
        KeyCode::Char(c) => {
            app.current_input_buffer().push(c);
        }
        _ => {}
    }
}
