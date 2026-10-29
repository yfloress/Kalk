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

//! "What if" mode: change grades freely, see where the course would land, and
//! get everything back untouched on the way out.
//!
//! Every editing path keeps working as usual; `persist` simply writes nothing
//! while a simulation is running, and leaving it restores the snapshot taken
//! on the way in.

use crate::model::Semester;

use super::App;
use super::history::Snapshot;

/// What a simulation puts aside on the way in.
#[derive(Debug)]
pub struct Simulation {
    saved: Snapshot,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl App {
    pub fn is_simulating(&self) -> bool {
        self.simulation.is_some()
    }

    pub fn toggle_simulation(&mut self) {
        if self.is_simulating() {
            self.exit_simulation();
        } else {
            self.enter_simulation();
        }
    }

    /// Undo history starts empty inside the simulation, so undoing there can
    /// never step back into real, saved states.
    fn enter_simulation(&mut self) {
        self.simulation = Some(Simulation {
            saved: self.snapshot(),
            undo: std::mem::take(&mut self.undo_stack),
            redo: std::mem::take(&mut self.redo_stack),
        });
        let msg = self.messages().simulation_started.to_string();
        self.set_status(msg);
    }

    fn exit_simulation(&mut self) {
        let Some(simulation) = self.simulation.take() else {
            return;
        };
        self.restore(simulation.saved);
        self.undo_stack = simulation.undo;
        self.redo_stack = simulation.redo;
        let msg = self.messages().simulation_ended.to_string();
        self.set_status(msg);
    }

    /// The semesters as saved on disk: during a simulation, the ones put aside
    /// on the way in, so an export never carries made-up grades.
    pub fn real_semesters(&self) -> &[Semester] {
        match &self.simulation {
            Some(simulation) => &simulation.saved.semesters,
            None => &self.semesters,
        }
    }

    /// Refuse an action whose result would vanish when the simulation ends.
    /// Returns whether it was refused.
    pub(crate) fn refuse_while_simulating(&mut self) -> bool {
        if self.is_simulating() {
            let msg = self.messages().simulation_blocks.to_string();
            self.set_warning(msg);
        }
        self.is_simulating()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Course;

    #[test]
    fn leaving_a_simulation_puts_everything_back() {
        let mut app = App::default();
        app.courses_mut()
            .push(Course::new("Physics".to_string(), 55.0));

        app.toggle_simulation();
        app.courses_mut()[0].name = "Changed".to_string();
        app.courses_mut()
            .push(Course::new("Made up".to_string(), 55.0));
        assert_eq!(app.real_semesters()[0].courses.len(), 1);

        app.toggle_simulation();
        assert!(!app.is_simulating());
        assert_eq!(app.courses().len(), 1);
        assert_eq!(app.courses()[0].name, "Physics");
    }
}
