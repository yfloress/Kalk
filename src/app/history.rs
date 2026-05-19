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

//! Undo / redo history for mutating user actions.
//!
//! Snapshots capture the user's course data and the current selection
//! (course/category/evaluation/focus) so that undo can also restore the
//! cursor to where the user was when the action was performed.
//!
//! Snapshots are taken *before* a mutation happens. The redo stack is
//! cleared whenever a new mutation is pushed (standard linear history).

use super::{App, Focus};
use crate::model::Course;

/// Maximum number of snapshots retained on the undo stack.
/// Each snapshot deep-clones `Vec<Course>` — 50 is plenty for interactive
/// work without meaningfully growing memory usage.
const MAX_HISTORY: usize = 50;

/// A point-in-time snapshot of the user's data and selection.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub courses: Vec<Course>,
    pub selected_course: Option<usize>,
    pub selected_category: Option<usize>,
    pub selected_evaluation: Option<usize>,
    pub focus: Focus,
}

impl App {
    /// Capture the current state into a snapshot.
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            courses: self.courses.clone(),
            selected_course: self.selected_course,
            selected_category: self.selected_category,
            selected_evaluation: self.selected_evaluation,
            focus: self.focus,
        }
    }

    /// Restore courses and selection from a snapshot.
    fn restore(&mut self, snap: Snapshot) {
        self.courses = snap.courses;
        self.selected_course = snap.selected_course;
        self.selected_category = snap.selected_category;
        self.selected_evaluation = snap.selected_evaluation;
        self.focus = snap.focus;
    }

    /// Push the current state onto the undo stack and clear redo.
    /// Call this *before* applying a mutating action so the previous state
    /// can be recovered.
    pub(crate) fn push_undo(&mut self) {
        let snap = self.snapshot();
        self.undo_stack.push(snap);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo the most recent mutating action.
    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            let current = self.snapshot();
            self.restore(snap);
            self.redo_stack.push(current);
            let msg = self.messages().undo_done.to_string();
            self.set_status(msg);
            self.persist();
        } else {
            let msg = self.messages().nothing_to_undo.to_string();
            self.set_warning(msg);
        }
    }

    /// Redo the most recently undone action.
    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            let current = self.snapshot();
            self.restore(snap);
            self.undo_stack.push(current);
            let msg = self.messages().redo_done.to_string();
            self.set_status(msg);
            self.persist();
        } else {
            let msg = self.messages().nothing_to_redo.to_string();
            self.set_warning(msg);
        }
    }
}
