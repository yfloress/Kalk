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

//! AI import flow and its editable JSON buffer.

use super::{App, Screen};
use crate::persistence;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::{CursorMove, TextArea, WrapMode};

impl App {
    // =========================================================================
    // AI Import Wizard
    // =========================================================================

    /// Enter the AI import wizard at step 1 (prompt).  Called from the
    /// template selector when the synthetic AI entry is confirmed.
    pub fn start_ai_import(&mut self) {
        if self.refuse_while_simulating() {
            self.screen = Screen::Main;
            return;
        }
        self.screen = Screen::ImportPrompt;
        self.import_copied = false;
        self.import_prompt_fullscreen = false;
        self.import_prompt_scroll = 0;
        self.import_paste_error = None;
        self.import_editor = json_editor("");
        self.import_parsed = None;
        self.import_renamed_from = None;
        self.import_total_weight = 0.0;
        self.import_total_evals = 0;
    }

    /// Toggle the full-screen prompt view in step 1.  Used as a universal
    /// copy-fallback for terminals that don't honour OSC 52: the user can
    /// select with the mouse and copy with the terminal's own shortcut.
    pub fn import_toggle_fullscreen(&mut self) {
        self.import_prompt_fullscreen = !self.import_prompt_fullscreen;
    }

    /// Scroll the step-1 prompt view by `delta` source lines (positive scrolls
    /// down, negative up).  Clamped between 0 and the prompt's source-line
    /// count — wrapped lines may extend further, so this is a soft upper
    /// bound rather than the exact end of the rendered text.
    pub fn import_scroll_prompt(&mut self, delta: i32) {
        let max = self.messages().import_prompt.lines().count() as i32;
        let new_scroll = (self.import_prompt_scroll as i32 + delta).clamp(0, max);
        self.import_prompt_scroll = new_scroll as u16;
    }

    /// Scroll within the wrapped preview's measured viewport limit.
    pub fn import_scroll_preview(&mut self, delta: i32, max_scroll: u16) {
        let current = i32::from(self.import_preview_scroll.min(max_scroll));
        let new = current
            .saturating_add(delta)
            .clamp(0, i32::from(max_scroll));
        self.import_preview_scroll = new as u16;
    }

    /// Jump the step-1 prompt view back to the top.
    pub fn import_scroll_prompt_top(&mut self) {
        self.import_prompt_scroll = 0;
    }

    /// Jump the step-1 prompt view to the bottom (last source line).
    pub fn import_scroll_prompt_bottom(&mut self) {
        let max = self.messages().import_prompt.lines().count() as u16;
        self.import_prompt_scroll = max.saturating_sub(1);
    }

    /// Copy the AI prompt to the system clipboard (OSC 52) and surface a
    /// confirmation in the UI.  Silently no-ops if the terminal doesn't
    /// support OSC 52 — the user can still select-and-copy from the popup.
    pub fn import_copy_prompt(&mut self) {
        let prompt = self.messages().import_prompt;
        if crate::clipboard::copy(prompt).is_ok() {
            self.import_copied = true;
        }
    }

    /// Advance from step 1 (prompt) to step 2 (paste).
    pub fn import_goto_paste(&mut self) {
        self.screen = Screen::ImportPaste;
        self.import_paste_error = None;
    }

    /// Step back from step 2 to step 1.  Keeps any prior copy-feedback state.
    pub fn import_back_to_prompt(&mut self) {
        self.screen = Screen::ImportPrompt;
        self.import_paste_error = None;
    }

    /// Step back from step 3 to step 2 (paste).  Clears the cached parse so
    /// the user can supply a different blob.
    pub fn import_back_to_paste(&mut self) {
        self.screen = Screen::ImportPaste;
        self.import_parsed = None;
        self.import_renamed_from = None;
    }

    /// Process a clipboard paste received during step 2.  On success
    /// transitions to step 3 (preview); on failure leaves the user on step 2
    /// with `import_paste_error` populated for display.
    pub fn import_handle_paste(&mut self, raw: String) {
        let raw = crate::app::share::read_if_path(&raw).unwrap_or(raw);
        // A complete course replaces the buffer; anything else is an edit.
        if crate::app::import::parse(&raw).is_ok() {
            self.import_editor = json_editor(&raw);
        } else {
            self.import_editor.insert_str(raw);
        }
        self.import_validate_editor();
    }

    pub fn import_edit_json(&mut self, key: KeyEvent) {
        let changed = match key.code {
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.import_editor.undo()
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.import_editor.redo()
            }
            _ => self.import_editor.input(key),
        };
        if changed {
            self.import_paste_error = None;
        }
    }

    pub fn import_validate_editor(&mut self) {
        let raw = self.import_editor.lines().join("\n");
        let m = self.messages();
        match crate::app::import::parse(&raw) {
            Ok(schema) => {
                let existing: Vec<&str> = self.courses().iter().map(|c| c.name.as_str()).collect();
                let course =
                    crate::app::import::to_course(&schema, &existing, m.import_copy_suffix);
                let renamed_from = if course.name != schema.name.trim() {
                    Some(schema.name.trim().to_string())
                } else {
                    None
                };
                self.import_total_weight = crate::app::import::total_weight(&schema);
                self.import_total_evals = crate::app::import::total_evaluations(&schema);
                self.import_parsed = Some(course);
                self.import_preview_scroll = 0;
                self.import_renamed_from = renamed_from;
                self.import_paste_error = None;
                self.screen = Screen::ImportPreview;
            }
            Err(e) => {
                self.import_paste_error = Some(e.user_message(m));
            }
        }
    }

    /// Confirm step 3: insert the parsed course into the working set and
    /// return to Main, selecting the new course.
    pub fn import_confirm(&mut self) {
        let Some(course) = self.import_parsed.take() else {
            self.screen = Screen::Main;
            return;
        };

        self.push_undo();
        self.courses_mut().push(course);
        let new_idx = self.courses().len() - 1;
        self.selected_course = Some(new_idx);
        self.reset_category_selection();
        self.persist();

        let msg = self.messages().import_imported_ok.to_string();
        self.set_status(msg);
        self.screen = Screen::Main;
    }

    /// Save the imported course's structure as a user template, without
    /// leaving the preview. The course itself stays pending for Enter.
    pub fn import_preview_save_template(&mut self) {
        let Some(course) = &self.import_parsed else {
            return;
        };
        let template = course.to_template(course.name.clone(), String::new());
        self.user_templates.push(template);

        if persistence::save_user_templates(&self.user_templates).is_err() {
            let msg = self.messages().save_template_error.to_string();
            self.set_error(msg);
            return;
        }

        let msg = self.messages().import_saved_template.to_string();
        self.set_status(msg);
    }

    /// Cancel the wizard from any step.  Clears all wizard state and returns
    /// to Main.
    pub fn import_cancel(&mut self) {
        self.import_editor = json_editor("");
        self.import_copied = false;
        self.import_prompt_fullscreen = false;
        self.import_paste_error = None;
        self.import_parsed = None;
        self.import_renamed_from = None;
        self.import_total_weight = 0.0;
        self.import_total_evals = 0;
        self.screen = Screen::Main;
    }
}

fn json_editor(text: &str) -> TextArea<'static> {
    let mut editor = TextArea::from(text.lines().map(str::to_string));
    editor.set_wrap_mode(WrapMode::Word);
    editor.set_tab_length(2);
    editor.set_max_histories(128);
    editor.move_cursor(CursorMove::Bottom);
    editor.move_cursor(CursorMove::End);
    editor
}
