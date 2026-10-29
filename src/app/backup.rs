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

//! Export and import of the whole library, driven from Home.

use crate::backup::{self, Backup};
use crate::model::CourseTemplate;
use crate::persistence;

use super::{App, Screen};

impl App {
    /// Write every semester and user template to today's backup file.
    pub fn export_backup(&mut self) {
        let m = self.messages();
        let Some(path) = backup::default_export_path() else {
            self.set_error(m.backup_err_no_home.to_string());
            return;
        };

        let data = Backup::new(&self.semesters, &self.user_templates);
        match backup::export(&path, &data) {
            Ok(()) => self.set_status(format!(
                "{} {}",
                m.backup_exported_to,
                backup::display_path(&path)
            )),
            Err(_) => self.set_error(m.backup_err_write.to_string()),
        }
    }

    /// Open the import popup, prefilled with the newest export in the home
    /// folder: after copying the file over, that is usually the one.
    pub fn start_import_backup(&mut self) {
        self.backup_path = backup::latest_in_home()
            .map(|path| backup::display_path(&path))
            .unwrap_or_else(|| "~/".to_string());
        self.backup_preview = None;
        self.backup_error = None;
        self.screen = Screen::ImportBackup;
    }

    /// First Enter reads the file and shows what it holds; the second one
    /// replaces the library with it.
    pub fn confirm_import_backup(&mut self) {
        if self.backup_preview.is_some() {
            self.apply_backup();
            return;
        }
        match backup::read(&backup::expand_path(&self.backup_path)) {
            Ok(data) => {
                self.backup_preview = Some(data);
                self.backup_error = None;
            }
            Err(err) => self.backup_error = Some(err.user_message(self.messages())),
        }
    }

    pub fn import_backup_type(&mut self, c: char) {
        self.backup_path.push(c);
        self.backup_path_changed();
    }

    pub fn import_backup_backspace(&mut self) {
        self.backup_path.pop();
        self.backup_path_changed();
    }

    /// A pasted or dragged-in path replaces whatever was typed.
    pub fn import_backup_paste(&mut self, text: String) {
        self.backup_path = text.trim().to_string();
        self.backup_path_changed();
    }

    pub fn cancel_import_backup(&mut self) {
        self.backup_preview = None;
        self.backup_error = None;
        self.screen = Screen::Home;
    }

    /// The preview belongs to the path it was read from.
    fn backup_path_changed(&mut self) {
        self.backup_preview = None;
        self.backup_error = None;
    }

    /// Replace every semester with the backup's, after copying the current
    /// `data.json` aside. Undo also brings the old semesters back.
    fn apply_backup(&mut self) {
        let Some(data) = self.backup_preview.take() else {
            return;
        };
        let m = self.messages();

        let safety_copy = match persistence::back_up_before_import() {
            Ok(copy) => copy,
            Err(_) => {
                self.backup_error = Some(m.backup_err_safety_copy.to_string());
                self.backup_preview = Some(data);
                return;
            }
        };
        // Whatever data.json held is in the copy now, so saving is safe again.
        self.save_blocked = false;

        let course_count = data.course_count();
        self.push_undo();
        let mut semesters = data.semesters;
        semesters.sort_by_key(|s| s.order);
        self.semesters = semesters;
        self.selected_semester = self.semesters.len() - 1;
        self.reset_course_selection();
        self.merge_user_templates(data.user_templates);
        self.persist();

        let mut msg = format!(
            "{} {} {} \u{b7} {} {}",
            m.backup_imported,
            self.semesters.len(),
            m.backup_semesters,
            course_count,
            m.course_plural
        );
        if let Some(path) = safety_copy {
            msg = format!(
                "{}. {} {}",
                msg,
                m.backup_previous_kept_in,
                backup::display_path(&path)
            );
        }
        // persist() reports its own failure; keep that one on screen.
        if self.status_message.is_none() {
            self.set_status(msg);
        }
        self.screen = Screen::Home;
    }

    /// Add the backup's templates that are not already here, by name.
    /// Templates are kept rather than replaced: they are cheap to have twice
    /// and expensive to lose.
    fn merge_user_templates(&mut self, incoming: Vec<CourseTemplate>) {
        let before = self.user_templates.len();
        for template in incoming {
            if !self.user_templates.iter().any(|t| t.name == template.name) {
                self.user_templates.push(template);
            }
        }
        if self.user_templates.len() != before
            && persistence::save_user_templates(&self.user_templates).is_err()
        {
            let msg = self.messages().save_template_error.to_string();
            self.set_error(msg);
        }
    }
}
