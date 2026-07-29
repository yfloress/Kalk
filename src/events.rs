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
    if !event::poll(Duration::from_millis(100))? {
        return Ok(app.should_quit);
    }

    match event::read()? {
        Event::Paste(s) => {
            // Only step 2 of the import wizard consumes paste events.  Other
            // screens silently ignore them so a stray paste doesn't clobber
            // form fields.
            if app.screen == Screen::ImportPaste {
                app.import_handle_paste(s);
            }
            return Ok(app.should_quit);
        }
        Event::Key(key) => {
            // Only handle key press events, not release
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            handle_key_event(app, key)
        }
        _ => Ok(app.should_quit),
    }
}

/// Dispatch a key-press event to the correct handler based on the current
/// screen.  Returns `Ok(true)` when the application should quit.
fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> color_eyre::Result<bool> {
    {

        // Global: Ctrl+Z / Ctrl+Y for undo / redo (Main screen only — popups
        // and forms have their own Esc-based cancel semantics).
        if app.screen == Screen::Main && key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('z') => {
                    app.undo();
                    return Ok(app.should_quit);
                }
                KeyCode::Char('y') => {
                    app.redo();
                    return Ok(app.should_quit);
                }
                _ => {}
            }
        }

        // Global: Shift+L for language selection (works from main screen)
        if key.code == KeyCode::Char('L')
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && app.screen == Screen::Main
        {
            app.show_language_popup();
            return Ok(app.should_quit);
        }

        // Global: Shift+S for settings (works from main screen)
        if key.code == KeyCode::Char('S')
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && app.screen == Screen::Main
        {
            app.show_settings();
            return Ok(app.should_quit);
        }

        // Ctrl+R toggles advanced rules in category edit popup
        if key.code == KeyCode::Char('r')
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(app.screen, Screen::EditingCategory { .. })
        {
            app.toggle_advanced_rules();
            return Ok(app.should_quit);
        }

        // ? toggles field help panel in category/course edit popup
        if key.code == KeyCode::Char('?')
            && matches!(
                app.screen,
                Screen::EditingCategory { .. } | Screen::EditingCourse { .. }
            )
        {
            app.toggle_field_help();
            return Ok(app.should_quit);
        }

        // ? from the main screen opens the global help overlay; pressing it
        // again (or Esc) closes it.
        if key.code == KeyCode::Char('?') {
            if app.screen == Screen::Main {
                app.show_help();
                return Ok(app.should_quit);
            } else if app.screen == Screen::Help {
                app.close_help();
                return Ok(app.should_quit);
            }
        }

        match &app.screen {
            Screen::Main => handle_main_keys(app, key.code, key.modifiers),
            Screen::SelectingTemplate => handle_template_keys(app, key.code),
            Screen::EditingCourse { .. } => handle_edit_course_keys(app, key.code),
            Screen::EditingCategory { .. } => handle_edit_category_keys(app, key.code),
            Screen::EditingEvaluation { .. } => handle_edit_evaluation_keys(app, key.code),
            Screen::ConfirmDelete => handle_delete_keys(app, key.code),
            Screen::ConfirmDeleteTemplate => handle_delete_template_keys(app, key.code),
            Screen::SavingTemplate => handle_save_template_keys(app, key.code),
            Screen::SelectingLanguage => handle_language_keys(app, key.code),
            Screen::Settings => handle_settings_keys(app, key.code),
            Screen::BulkAddEvaluations => handle_bulk_add_keys(app, key.code),
            Screen::EnteringGlobalGrade => handle_global_grade_keys(app, key.code),
            Screen::Help => handle_help_keys(app, key.code),
            Screen::ImportPrompt => handle_import_prompt_keys(app, key.code),
            Screen::ImportPaste => handle_import_paste_keys(app, key.code),
            Screen::ImportPreview => handle_import_preview_keys(app, key.code),
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
            if app.input_field.is_numeric() {
                if c.is_ascii_digit() || c == '.' || c == ',' {
                    app.current_input_buffer().push(c);
                }
            } else {
                app.current_input_buffer().push(c);
            }
        }
        _ => {}
    }
}

/// Handle keys in the main screen.
fn handle_main_keys(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    // Clear any status message on the next user action
    app.clear_status();

    // Ctrl+N: bulk-add evaluations (only when focused on evaluations or categories)
    if key == KeyCode::Char('n') && modifiers.contains(KeyModifiers::CONTROL) {
        if matches!(app.focus, Focus::Evaluations | Focus::Categories)
            && !app.is_on_virtual_global()
        {
            app.start_bulk_add_evaluations();
        }
        return;
    }

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

        // Jump to first / last item in current focus
        KeyCode::Home => app.goto_first(),
        KeyCode::End | KeyCode::Char('G') => app.goto_last(),

        // Create new item
        KeyCode::Char('n') => match app.focus {
            Focus::Courses => app.start_new_course(),
            Focus::Categories => {
                if app.is_on_virtual_global() {
                    app.start_global_grade_entry();
                } else {
                    app.start_new_category();
                }
            }
            Focus::Evaluations => {
                if app.is_on_virtual_global() {
                    app.start_global_grade_entry();
                } else {
                    app.start_new_evaluation();
                }
            }
        },

        // Edit selected item
        KeyCode::Enter => match app.focus {
            Focus::Courses => app.start_edit_course(),
            Focus::Categories => {
                if app.is_on_virtual_global() {
                    app.start_global_grade_entry();
                } else {
                    app.start_edit_category();
                }
            }
            Focus::Evaluations => {
                if app.is_on_virtual_global() {
                    app.start_global_grade_entry();
                } else {
                    app.start_edit_evaluation();
                }
            }
        },

        // Delete selected item (no-op on virtual global category)
        KeyCode::Char('d') => {
            if !app.is_on_virtual_global() {
                app.request_delete();
            }
        }

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

        // Enter global exam grade
        KeyCode::Char('g') => {
            if app.focus == Focus::Courses && app.current_course().is_some() {
                app.start_global_grade_entry();
            }
        }

        // Yank (copy) the current evaluation
        KeyCode::Char('y') => {
            if app.focus == Focus::Evaluations && !app.is_on_virtual_global() {
                app.yank_evaluation();
            }
        }

        // Paste the yanked evaluation into the current category
        KeyCode::Char('p') => {
            if app.focus == Focus::Evaluations && !app.is_on_virtual_global() {
                app.paste_evaluation();
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
/// Toggle fields (GlobalPolicy) cycle on Space/Left/Right; Enter always confirms.
fn handle_edit_course_keys(app: &mut App, key: KeyCode) {
    if app.input_field.is_toggle() {
        match key {
            KeyCode::Esc => app.cancel_edit(),
            KeyCode::Tab => app.next_input_field(),
            KeyCode::Enter => app.confirm_course(),
            KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => app.cycle_toggle_field(),
            KeyCode::Left | KeyCode::Char('h') => app.cycle_toggle_field_reverse(),
            _ => {}
        }
    } else {
        handle_form_keys(app, key, App::confirm_course, App::cancel_edit);
    }
}

/// Handle keys in the category editing screen.
/// Toggle fields (AvgMethod, OnMinNotMet, RoundBeforeWeight) cycle on Space/Left/Right;
/// Enter always confirms.
fn handle_edit_category_keys(app: &mut App, key: KeyCode) {
    if app.input_field.is_toggle() {
        match key {
            KeyCode::Esc => app.cancel_edit(),
            KeyCode::Tab => app.next_input_field(),
            KeyCode::Enter => app.confirm_category(),
            KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => app.cycle_toggle_field(),
            KeyCode::Left | KeyCode::Char('h') => app.cycle_toggle_field_reverse(),
            _ => {}
        }
    } else {
        handle_form_keys(app, key, App::confirm_category, App::cancel_edit);
    }
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

/// Handle keys in the settings screen.
fn handle_settings_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_settings(),
        KeyCode::Enter => app.confirm_settings(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_setting(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => app.next_setting(),
        KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => app.toggle_current_setting(),
        KeyCode::Left | KeyCode::Char('h') => app.toggle_current_setting_reverse(),
        _ => {}
    }
}

/// Handle keys in the bulk-add evaluations popup.
/// Two fields: Name (base name) and Count (number of evaluations).
fn handle_bulk_add_keys(app: &mut App, key: KeyCode) {
    use crate::app::InputField;
    let on_name = app.input_field == InputField::Name;

    match key {
        KeyCode::Esc => app.cancel_bulk_add(),
        KeyCode::Enter => app.confirm_bulk_add(),
        KeyCode::Tab => {
            app.input_field = if on_name {
                InputField::Grade // reuse Grade as the "count" field
            } else {
                InputField::Name
            };
        }
        KeyCode::Backspace => {
            if on_name {
                app.edit_name.pop();
            } else {
                app.edit_bulk_count.pop();
            }
        }
        KeyCode::Char(c) => {
            if on_name {
                app.edit_name.push(c);
            } else if c.is_ascii_digit() {
                app.edit_bulk_count.push(c);
            }
        }
        _ => {}
    }
}

/// Handle keys in the global help overlay.
/// Any of Esc / q / Enter closes the overlay — pressing `?` again is handled
/// by the global toggle above.
fn handle_help_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => app.close_help(),
        _ => {}
    }
}

/// Handle keys in the global grade entry popup.
fn handle_global_grade_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.cancel_global_grade(),
        KeyCode::Enter => app.confirm_global_grade(),
        KeyCode::Backspace => {
            app.edit_global_grade.pop();
        }
        KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == ',' => {
            app.edit_global_grade.push(c);
        }
        _ => {}
    }
}


/// Step 1 of the AI import wizard — copy the prompt to the clipboard, scroll
/// it, enter the full-screen view, or advance to the paste step.
fn handle_import_prompt_keys(app: &mut App, key: KeyCode) {
    // In full-screen mode the scrolling and exit shortcuts are the only
    // active keys so the terminal's selection isn't interrupted.
    if app.import_prompt_fullscreen {
        match key {
            KeyCode::Esc
            | KeyCode::Char('q')
            | KeyCode::Char('f')
            | KeyCode::Char('F') => app.import_toggle_fullscreen(),
            KeyCode::Down | KeyCode::Char('j') => app.import_scroll_prompt(1),
            KeyCode::Up | KeyCode::Char('k') => app.import_scroll_prompt(-1),
            KeyCode::PageDown | KeyCode::Char(' ') => app.import_scroll_prompt(10),
            KeyCode::PageUp => app.import_scroll_prompt(-10),
            KeyCode::Home | KeyCode::Char('g') => app.import_scroll_prompt_top(),
            KeyCode::End | KeyCode::Char('G') => app.import_scroll_prompt_bottom(),
            _ => {}
        }
        return;
    }

    match key {
        KeyCode::Esc => app.import_cancel(),
        KeyCode::Char('c') | KeyCode::Char('y') => app.import_copy_prompt(),
        KeyCode::Char('f') | KeyCode::Char('F') => app.import_toggle_fullscreen(),
        KeyCode::Down | KeyCode::Char('j') => app.import_scroll_prompt(1),
        KeyCode::Up | KeyCode::Char('k') => app.import_scroll_prompt(-1),
        KeyCode::PageDown => app.import_scroll_prompt(10),
        KeyCode::PageUp => app.import_scroll_prompt(-10),
        KeyCode::Home => app.import_scroll_prompt_top(),
        KeyCode::End => app.import_scroll_prompt_bottom(),
        KeyCode::Enter => app.import_goto_paste(),
        _ => {}
    }
}

/// Step 2 of the AI import wizard — waits for a paste event.  Pure key events
/// only support going back or cancelling; the actual paste content is fed in
/// via [`App::import_handle_paste`] from the [`Event::Paste`] branch.
fn handle_import_paste_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.import_cancel(),
        KeyCode::Char('b') => app.import_back_to_prompt(),
        _ => {}
    }
}

/// Step 3 of the AI import wizard — preview and confirm/back/cancel.
fn handle_import_preview_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.import_cancel(),
        KeyCode::Char('b') => app.import_back_to_paste(),
        KeyCode::Enter => app.import_confirm(),
        _ => {}
    }
}
