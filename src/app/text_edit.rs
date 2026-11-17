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

//! Cursor-aware editing without changing the strings saved by forms.

use super::{App, InputField, Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_input::{Input, InputRequest};

#[derive(Debug, Default)]
pub struct TextEditor {
    target: Option<(Screen, InputField)>,
    input: Input,
}

impl App {
    pub fn is_text_entry(&self) -> bool {
        matches!(
            self.screen,
            Screen::EditingCourse { .. }
                | Screen::EditingCategory { .. }
                | Screen::EditingEvaluation { .. }
                | Screen::EditingAttendance
                | Screen::SavingTemplate
                | Screen::BulkAddEvaluations
                | Screen::EnteringGlobalGrade
                | Screen::EditingSemester { .. }
                | Screen::ImportBackup
        )
    }

    fn text_buffer(&mut self) -> Option<&mut String> {
        match self.screen {
            Screen::BulkAddEvaluations if self.input_field != InputField::Name => {
                Some(&mut self.edit_bulk_count)
            }
            Screen::EnteringGlobalGrade => Some(&mut self.edit_global_grade),
            Screen::EditingSemester { .. } => Some(&mut self.edit_name),
            Screen::ImportBackup => Some(&mut self.backup_path),
            _ if self.is_text_entry() => self.current_input_buffer(),
            _ => None,
        }
    }

    fn allows_text_char(&self, c: char) -> bool {
        if c.is_control() {
            return false;
        }
        match self.screen {
            Screen::BulkAddEvaluations if self.input_field != InputField::Name => {
                c.is_ascii_digit()
            }
            Screen::EnteringGlobalGrade => c.is_ascii_digit() || c == '.' || c == ',',
            Screen::EditingSemester { .. } | Screen::ImportBackup => true,
            _ => !self.input_field.is_numeric() || c.is_ascii_digit() || c == '.' || c == ',',
        }
    }

    fn sync_editor(&mut self) -> bool {
        let target = (self.screen.clone(), self.input_field);
        let Some(value) = self.text_buffer().cloned() else {
            return false;
        };
        if self.text_editor.target.as_ref() != Some(&target)
            || self.text_editor.input.value() != value
        {
            self.text_editor.input = Input::new(value);
            self.text_editor.target = Some(target);
        }
        true
    }

    fn store_editor(&mut self) {
        let value = self.text_editor.input.value().to_string();
        if let Some(buffer) = self.text_buffer() {
            let changed = *buffer != value;
            *buffer = value;
            if changed && self.screen == Screen::ImportBackup {
                self.backup_preview = None;
                self.backup_error = None;
            }
        }
    }

    pub fn edit_focused_text(&mut self, key: KeyEvent) {
        let Some(request) = key_request(key) else {
            return;
        };
        if let InputRequest::InsertChar(c) = request
            && !self.allows_text_char(c)
        {
            return;
        }
        if self.sync_editor() {
            self.text_editor.input.handle(request);
            self.store_editor();
        }
    }

    pub fn paste_into_focused_field(&mut self, text: &str) {
        let first = text.lines().next().unwrap_or("");
        let clean: Vec<char> = first
            .chars()
            .filter(|c| self.allows_text_char(*c))
            .collect();
        if self.sync_editor() {
            for c in clean {
                self.text_editor.input.handle(InputRequest::InsertChar(c));
            }
            self.store_editor();
        }
    }

    /// An unsynchronized/newly focused field starts at the end of its value.
    pub(crate) fn form_input(&self, value: &str) -> Input {
        if self.text_editor.target.as_ref() == Some(&(self.screen.clone(), self.input_field))
            && self.text_editor.input.value() == value
        {
            self.text_editor.input.clone()
        } else {
            Input::new(value.to_string())
        }
    }

    pub fn previous_input_field(&mut self) {
        let original = self.input_field;
        for _ in 0..40 {
            let candidate = self.input_field;
            self.next_input_field();
            if self.input_field == original {
                self.input_field = candidate;
                return;
            }
        }
        self.input_field = original;
    }
}

pub(crate) fn key_request(key: KeyEvent) -> Option<InputRequest> {
    use InputRequest::*;
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    Some(match key.code {
        KeyCode::Left if ctrl || alt => GoToPrevWord,
        KeyCode::Right if ctrl || alt => GoToNextWord,
        KeyCode::Left => GoToPrevChar,
        KeyCode::Right => GoToNextChar,
        KeyCode::Home => GoToStart,
        KeyCode::End => GoToEnd,
        KeyCode::Backspace if ctrl || alt => DeletePrevWord,
        KeyCode::Delete if ctrl || alt => DeleteNextWord,
        KeyCode::Backspace => DeletePrevChar,
        KeyCode::Delete => DeleteNextChar,
        KeyCode::Char('a') if ctrl => GoToStart,
        KeyCode::Char('e') if ctrl => GoToEnd,
        KeyCode::Char('u') if ctrl => DeleteFromStart,
        KeyCode::Char('k') if ctrl => DeleteTillEnd,
        KeyCode::Char('w') if ctrl => DeletePrevWord,
        KeyCode::Char('b') if alt => GoToPrevWord,
        KeyCode::Char('f') if alt => GoToNextWord,
        KeyCode::Char('d') if alt => DeleteNextWord,
        KeyCode::Char(c) if !ctrl && !alt && !c.is_control() => InsertChar(c),
        _ => return None,
    })
}
