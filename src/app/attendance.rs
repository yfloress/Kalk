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

//! The attendance popup: classes held, classes missed, and what falling short
//! of the requirement does to the course.

use crate::model::Attendance;

use super::{App, InputField, Screen};

impl App {
    pub fn start_edit_attendance(&mut self) {
        let Some(course) = self.current_course() else {
            return;
        };
        let a = course.attendance.clone();

        self.edit_classes_total = a.total_classes.map(|t| t.to_string()).unwrap_or_default();
        self.edit_classes_missed = if a.total_classes.is_some() {
            a.missed.to_string()
        } else {
            String::new()
        };
        self.edit_attendance_required = a
            .required_percent
            .map(|p| format!("{:.0}", p))
            .unwrap_or_default();
        self.edit_attendance_action = a.action;

        self.input_field = InputField::ClassesTotal;
        self.screen = Screen::EditingAttendance;
    }

    pub fn confirm_attendance(&mut self) {
        let attendance = Attendance {
            total_classes: self
                .edit_classes_total
                .trim()
                .parse()
                .ok()
                .filter(|t| *t > 0),
            missed: self.edit_classes_missed.trim().parse().unwrap_or(0),
            required_percent: self
                .edit_attendance_required
                .trim()
                .replace(',', ".")
                .parse::<f64>()
                .ok()
                .map(|p| p.clamp(0.0, 100.0)),
            action: self.edit_attendance_action,
        };

        self.push_undo();
        if let Some(idx) = self.selected_course
            && let Some(course) = self.courses_mut().get_mut(idx)
        {
            course.attendance = attendance;
        }

        self.screen = Screen::Main;
        self.clear_status();
        self.persist();
    }

    /// Attendance as it would stand with the values currently typed, so the
    /// popup can show the percentage before anything is saved.
    pub fn previewed_attendance(&self) -> Attendance {
        Attendance {
            total_classes: self
                .edit_classes_total
                .trim()
                .parse()
                .ok()
                .filter(|t| *t > 0),
            missed: self.edit_classes_missed.trim().parse().unwrap_or(0),
            required_percent: self
                .edit_attendance_required
                .trim()
                .replace(',', ".")
                .parse::<f64>()
                .ok(),
            action: self.edit_attendance_action,
        }
    }
}
