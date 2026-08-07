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

//! Status-message severity and the setters that drive the footer banner.

use super::App;

/// Severity of a transient status message shown in the footer.
/// Drives the colour of the banner so that informational, warning and
/// error states are distinguishable in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusSeverity {
    #[default]
    Info,
    Warning,
    Error,
}

impl App {
    /// Clear the status message (called before each user action).
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_severity = StatusSeverity::Info;
    }

    /// Set an informational status message (neutral colour).
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_severity = StatusSeverity::Info;
    }

    /// Set a warning status message (yellow/peach colour).
    pub fn set_warning(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_severity = StatusSeverity::Warning;
    }

    /// Set an error status message (red colour).
    pub fn set_error(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_severity = StatusSeverity::Error;
    }
}
