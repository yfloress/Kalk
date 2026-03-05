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

//! Event handling for keyboard input.
//!
//! This module handles all keyboard events and dispatches them
//! to the appropriate handlers based on the current screen.

use crate::app::{App, Focus, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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

        // Global: Ctrl+L for language selection (works from main screen)
        if key.code == KeyCode::Char('l')
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && app.screen == Screen::Main
        {
            app.show_language_popup();
            return Ok(app.should_quit);
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
            Screen::SelectingLanguage => handle_language_keys(app, key.code),
        }
    }
    Ok(app.should_quit)
}

/// Shared form input handler (Esc, Tab, Enter, Backspace, Char).
fn handle_form_keys(app: &mut App, key: KeyCode, confirm: fn(&mut App), cancel: fn(&mut App)) {
    match key {
        KeyCode::Esc => cancel(app),
        KeyCode::Tab => app.next_input_field(),
        KeyCode::Enter => confirm(app),
        KeyCode::Backspace => {
            app.current_input_buffer().pop();
        }
        KeyCode::Char(c) => {
            app.current_input_buffer().push(c);
        }
        _ => {}
    }
}

/// Handle keys in the main screen.
fn handle_main_keys(app: &mut App, key: KeyCode) {
    // Clear any status message on the next user action
    app.clear_status();

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
    handle_form_keys(app, key, App::confirm_course, App::cancel_edit);
}

/// Handle keys in the category editing screen.
fn handle_edit_category_keys(app: &mut App, key: KeyCode) {
    handle_form_keys(app, key, App::confirm_category, App::cancel_edit);
}

/// Handle keys in the evaluation editing screen.
fn handle_edit_evaluation_keys(app: &mut App, key: KeyCode) {
    handle_form_keys(app, key, App::confirm_evaluation, App::cancel_edit);
}

/// Handle keys in the delete confirmation screen.
fn handle_delete_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter | KeyCode::Char('y') => app.delete_current(),
        KeyCode::Esc | KeyCode::Char('n') => app.cancel_edit(),
        _ => {}
    }
}

/// Handle keys in the language selection screen.
fn handle_language_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_language_selection(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_language(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => app.next_language(),
        KeyCode::Enter => app.confirm_language_selection(),
        _ => {}
    }
}

/// Handle keys in the save template screen.
fn handle_save_template_keys(app: &mut App, key: KeyCode) {
    handle_form_keys(app, key, App::confirm_save_template, App::cancel_edit);
}
