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

use std::time::{Duration, Instant};

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

impl StatusSeverity {
    /// How long a message survives keypresses. Errors stay longest: they
    /// usually carry something to act on, like the path of a copy.
    fn min_display(self) -> Duration {
        match self {
            StatusSeverity::Info => Duration::from_secs(3),
            StatusSeverity::Warning => Duration::from_secs(6),
            StatusSeverity::Error => Duration::from_secs(12),
        }
    }

    /// Whether a message shown for `elapsed` may be cleared by a keypress.
    fn may_clear_after(self, elapsed: Duration) -> bool {
        elapsed >= self.min_display()
    }
}

impl App {
    /// Clear the status message before a keypress is handled, unless it has
    /// not been on screen long enough to be read.
    pub fn clear_status(&mut self) {
        if let Some(since) = self.status_since
            && !self.status_severity.may_clear_after(since.elapsed())
        {
            return;
        }
        self.dismiss_status();
    }

    /// Clear the status message now, read or not: for an action that has
    /// just resolved what the message was about, like a fixed form.
    pub fn dismiss_status(&mut self) {
        self.status_message = None;
        self.status_severity = StatusSeverity::Info;
        self.status_since = None;
    }

    fn show_status(&mut self, msg: String, severity: StatusSeverity) {
        self.status_message = Some(msg);
        self.status_severity = severity;
        self.status_since = Some(Instant::now());
    }

    /// Set an informational status message (neutral colour).
    pub fn set_status(&mut self, msg: String) {
        self.show_status(msg, StatusSeverity::Info);
    }

    /// Set a warning status message (yellow/peach colour).
    pub fn set_warning(&mut self, msg: String) {
        self.show_status(msg, StatusSeverity::Warning);
    }

    /// Set an error status message (red colour).
    pub fn set_error(&mut self, msg: String) {
        self.show_status(msg, StatusSeverity::Error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_outlast_warnings_which_outlast_info() {
        let after = Duration::from_secs(4);
        assert!(StatusSeverity::Info.may_clear_after(after));
        assert!(!StatusSeverity::Warning.may_clear_after(after));
        assert!(!StatusSeverity::Error.may_clear_after(Duration::from_secs(8)));
        assert!(StatusSeverity::Error.may_clear_after(Duration::from_secs(12)));
    }
}
