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

//! Semester navigation and CRUD, driven from the Home screen.

use crate::model::Semester;
use crate::persistence;

use super::{App, Focus, InputField, Screen, semester_name};

impl App {
    // =========================================================================
    // Session
    // =========================================================================

    /// Current settings, ready to write to disk.
    pub(crate) fn config(&self) -> persistence::Config {
        persistence::Config {
            language: self.language,
            use_nerd_fonts: self.use_nerd_fonts,
            start_on_home: self.screen == Screen::Home,
            configured: self.configured,
            last_semester: self.semesters.get(self.selected_semester).map(|s| s.id),
        }
    }

    /// Remember where the user was, so the next run opens there.
    pub fn save_session(&self) {
        let _ = persistence::save_config(&self.config());
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    pub fn show_home(&mut self) {
        self.screen = Screen::Home;
    }

    /// Leave Home without changing which semester is open.
    pub fn close_home(&mut self) {
        self.screen = Screen::Main;
    }

    pub fn next_semester(&mut self) {
        if self.selected_semester + 1 < self.semesters.len() {
            self.selected_semester += 1;
            self.reset_course_selection();
        }
    }

    pub fn previous_semester(&mut self) {
        if self.selected_semester > 0 {
            self.selected_semester -= 1;
            self.reset_course_selection();
        }
    }

    /// Open the highlighted semester in the main three-panel view.
    pub fn enter_semester(&mut self) {
        self.focus = Focus::Courses;
        self.screen = Screen::Main;
    }

    /// Point the course/category/evaluation cursors at something valid for the
    /// semester now in view.
    fn reset_course_selection(&mut self) {
        self.selected_course = if self.courses().is_empty() {
            None
        } else {
            Some(0)
        };
        self.reset_category_selection();
        self.reset_evaluation_selection();
    }

    // =========================================================================
    // Create / rename
    // =========================================================================

    pub fn start_new_semester(&mut self) {
        self.screen = Screen::EditingSemester { is_new: true };
        self.input_field = InputField::Name;
        self.edit_name = semester_name(
            self.messages().semester_name_prefix,
            self.semesters.len() as u32,
        );
    }

    pub fn start_rename_semester(&mut self) {
        self.screen = Screen::EditingSemester { is_new: false };
        self.input_field = InputField::Name;
        self.edit_name = self.current_semester().name.clone();
    }

    pub fn confirm_semester(&mut self) {
        let name = self.edit_name.trim().to_string();
        if name.is_empty() {
            return;
        }

        let is_new = matches!(self.screen, Screen::EditingSemester { is_new: true });
        self.push_undo();

        if is_new {
            let order = self.semesters.last().map_or(0, |s| s.order + 1);
            self.semesters.push(Semester::new(name, order));
            self.selected_semester = self.semesters.len() - 1;
            self.reset_course_selection();
        } else {
            let idx = self.selected_semester.min(self.semesters.len() - 1);
            self.semesters[idx].name = name;
        }

        self.screen = Screen::Home;
        self.clear_status();
        self.persist();
    }

    // =========================================================================
    // Delete / reorder
    // =========================================================================

    pub fn start_delete_semester(&mut self) {
        // The app is built around always having a semester to show.
        if self.semesters.len() <= 1 {
            let msg = self.messages().semester_cannot_delete_last.to_string();
            self.set_warning(msg);
            return;
        }
        self.screen = Screen::ConfirmDeleteSemester;
    }

    pub fn confirm_delete_semester(&mut self) {
        if self.semesters.len() <= 1 {
            self.screen = Screen::Home;
            return;
        }

        self.push_undo();
        let idx = self.selected_semester.min(self.semesters.len() - 1);
        self.semesters.remove(idx);
        self.selected_semester = idx.min(self.semesters.len() - 1);
        self.reset_course_selection();

        self.screen = Screen::Home;
        self.persist();
    }

    pub fn move_semester_up(&mut self) {
        let idx = self.selected_semester;
        if idx == 0 || self.semesters.len() < 2 {
            return;
        }
        self.push_undo();
        self.semesters.swap(idx, idx - 1);
        self.renumber_semesters();
        self.selected_semester = idx - 1;
        self.persist();
    }

    pub fn move_semester_down(&mut self) {
        let idx = self.selected_semester;
        if idx + 1 >= self.semesters.len() {
            return;
        }
        self.push_undo();
        self.semesters.swap(idx, idx + 1);
        self.renumber_semesters();
        self.selected_semester = idx + 1;
        self.persist();
    }

    /// Rewrite `order` to match position, so the list survives a reload.
    fn renumber_semesters(&mut self) {
        for (i, semester) in self.semesters.iter_mut().enumerate() {
            semester.order = i as u32;
        }
    }
}
