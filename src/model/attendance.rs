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

//! Attendance requirements.
//!
//! Kalk derives everything else from grades, but attendance is a gate that no
//! grade can open: a course with a 90 average and 70% attendance is failed.
//! Tracked as classes held and classes missed, because that is what a student
//! actually keeps count of.

use serde::{Deserialize, Serialize};

/// What falling below the required attendance does to the course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttendanceAction {
    /// Show it, but let the grades decide the verdict.
    #[default]
    WarnOnly,
    /// The course is failed regardless of the grades.
    FailCourse,
}

/// Attendance record for one course. Inert until `required_percent` is set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attendance {
    /// Classes held so far. `None` means attendance is not being tracked.
    #[serde(default)]
    pub total_classes: Option<u32>,
    /// Classes missed out of `total_classes`.
    #[serde(default)]
    pub missed: u32,
    /// Minimum percentage required to pass the course.
    #[serde(default)]
    pub required_percent: Option<f64>,
    #[serde(default)]
    pub action: AttendanceAction,
}

impl Attendance {
    /// Percentage attended, `None` while no classes have been recorded.
    pub fn percent(&self) -> Option<f64> {
        let total = self.total_classes.filter(|t| *t > 0)?;
        let attended = total.saturating_sub(self.missed);
        Some(f64::from(attended) / f64::from(total) * 100.0)
    }

    /// True when a requirement exists and the current percentage is below it.
    pub fn is_below_requirement(&self) -> bool {
        match (self.required_percent, self.percent()) {
            (Some(required), Some(actual)) => actual < required,
            _ => false,
        }
    }

    /// True when the shortfall must fail the course outright.
    pub fn fails_course(&self) -> bool {
        self.action == AttendanceAction::FailCourse && self.is_below_requirement()
    }

    /// Classes that may still be missed while staying at or above the
    /// requirement, assuming `total_classes` is the final count.
    /// `None` when nothing is being tracked.
    pub fn misses_remaining(&self) -> Option<u32> {
        let total = self.total_classes.filter(|t| *t > 0)?;
        let required = self.required_percent?;
        let allowed = (f64::from(total) * (1.0 - required / 100.0)).floor() as u32;
        Some(allowed.saturating_sub(self.missed))
    }

    pub fn is_tracked(&self) -> bool {
        self.total_classes.is_some_and(|t| t > 0)
    }
}
