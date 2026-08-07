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

//! Form input fields: which field is focused, how focus advances,
//! and how toggle fields cycle.

use super::{App, Screen};
use crate::model::{AttendanceAction, AveragingMethod, GlobalExamPolicy};

/// Maximum value for the "drop lowest" toggle cycle (0..=MAX_DROP_LOWEST).
const MAX_DROP_LOWEST: usize = 5;

/// Input field being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Name,
    PassingGrade,
    Credits,
    Weight,
    Grade,
    Description,
    /// Evaluation weight within a weighted-evaluations category (text input).
    EvalWeight,
    // Category rule fields (text input)
    MinimumAverage,
    MinPerEval,
    MinOneEval,
    // Category rule toggle fields (cycled with Space/Enter, not typed)
    DropLowest,
    AvgMethod,
    OnMinNotMet,
    OnMinPerEvalNotMet,
    OnMinOneEvalNotMet,
    RoundBeforeWeight,
    /// Toggle: whether evaluations in this category have individual weights.
    WeightedEvals,
    // Attendance popup
    ClassesTotal,
    ClassesMissed,
    AttendanceRequired,
    AttendanceAction,
    // Global exam fields (course form)
    GlobalPolicy,
    GlobalSemesterWeight,
    GlobalExamWeight,
    GlobalMinGrade,
}

impl InputField {
    /// Returns true if this field is a toggle (cycled, not typed).
    pub fn is_toggle(&self) -> bool {
        matches!(
            self,
            InputField::DropLowest
                | InputField::AvgMethod
                | InputField::OnMinNotMet
                | InputField::OnMinPerEvalNotMet
                | InputField::OnMinOneEvalNotMet
                | InputField::RoundBeforeWeight
                | InputField::WeightedEvals
                | InputField::AttendanceAction
                | InputField::GlobalPolicy
        )
    }

    /// Returns true if this field only accepts numeric input (digits, dot, comma).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            InputField::PassingGrade
                | InputField::Credits
                | InputField::Weight
                | InputField::Grade
                | InputField::EvalWeight
                | InputField::MinimumAverage
                | InputField::MinPerEval
                | InputField::MinOneEval
                | InputField::ClassesTotal
                | InputField::ClassesMissed
                | InputField::AttendanceRequired
                | InputField::GlobalSemesterWeight
                | InputField::GlobalExamWeight
                | InputField::GlobalMinGrade
        )
    }
}

impl App {
    pub fn next_input_field(&mut self) {
        self.input_field = match (&self.screen, &self.input_field) {
            (Screen::EditingCourse { .. }, InputField::Name) => InputField::PassingGrade,
            (Screen::EditingCourse { .. }, InputField::PassingGrade) => InputField::Credits,
            (Screen::EditingCourse { .. }, InputField::Credits) => InputField::GlobalPolicy,
            (Screen::EditingCourse { .. }, InputField::GlobalPolicy) => {
                match self.edit_global_policy {
                    GlobalExamPolicy::None => InputField::Name,
                    GlobalExamPolicy::Weighted { .. } => InputField::GlobalSemesterWeight,
                    GlobalExamPolicy::ReplacesWorstGrade => InputField::GlobalMinGrade,
                }
            }
            (Screen::EditingCourse { .. }, InputField::GlobalSemesterWeight) => {
                InputField::GlobalExamWeight
            }
            (Screen::EditingCourse { .. }, InputField::GlobalExamWeight) => {
                InputField::GlobalMinGrade
            }
            (Screen::EditingCourse { .. }, InputField::GlobalMinGrade) => InputField::Name,
            // Category: Name → Weight → (advanced fields if expanded) → Name
            (Screen::EditingCategory { .. }, InputField::Name) => InputField::Weight,
            (Screen::EditingCategory { .. }, InputField::Weight) => {
                if self.show_advanced_rules {
                    InputField::DropLowest
                } else {
                    InputField::Name
                }
            }
            (Screen::EditingCategory { .. }, InputField::DropLowest) => InputField::AvgMethod,
            (Screen::EditingCategory { .. }, InputField::AvgMethod) => InputField::MinimumAverage,
            (Screen::EditingCategory { .. }, InputField::MinimumAverage) => {
                // Show OnMinNotMet only if a minimum average is set
                if !self.edit_min_average.trim().is_empty() {
                    InputField::OnMinNotMet
                } else {
                    InputField::MinPerEval
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinNotMet) => InputField::MinPerEval,
            (Screen::EditingCategory { .. }, InputField::MinPerEval) => {
                // Show OnMinPerEvalNotMet only if min per eval is set
                if !self.edit_min_per_eval.trim().is_empty() {
                    InputField::OnMinPerEvalNotMet
                } else {
                    InputField::MinOneEval
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinPerEvalNotMet) => {
                InputField::MinOneEval
            }
            (Screen::EditingCategory { .. }, InputField::MinOneEval) => {
                // Show OnMinOneEvalNotMet only if min one eval is set
                if !self.edit_min_one_eval.trim().is_empty() {
                    InputField::OnMinOneEvalNotMet
                } else {
                    InputField::RoundBeforeWeight
                }
            }
            (Screen::EditingCategory { .. }, InputField::OnMinOneEvalNotMet) => {
                InputField::RoundBeforeWeight
            }
            (Screen::EditingCategory { .. }, InputField::RoundBeforeWeight) => {
                InputField::WeightedEvals
            }
            (Screen::EditingCategory { .. }, InputField::WeightedEvals) => InputField::Name,
            (Screen::EditingEvaluation { .. }, InputField::Grade) => InputField::Name,
            (Screen::EditingEvaluation { .. }, InputField::Name) => {
                // Show weight field only when category has weighted evaluations
                if self.category_has_weighted_evals() {
                    InputField::EvalWeight
                } else {
                    InputField::Grade
                }
            }
            (Screen::EditingEvaluation { .. }, InputField::EvalWeight) => InputField::Grade,
            (Screen::EditingAttendance, InputField::ClassesTotal) => InputField::ClassesMissed,
            (Screen::EditingAttendance, InputField::ClassesMissed) => {
                InputField::AttendanceRequired
            }
            (Screen::EditingAttendance, InputField::AttendanceRequired) => {
                InputField::AttendanceAction
            }
            (Screen::EditingAttendance, InputField::AttendanceAction) => InputField::ClassesTotal,
            (Screen::SavingTemplate, InputField::Name) => InputField::Description,
            (Screen::SavingTemplate, InputField::Description) => InputField::Name,
            _ => self.input_field,
        };
    }

    /// The text buffer behind the focused field, or `None` when the field is a
    /// toggle and has no text to edit. Returning an `Option` keeps a caller
    /// from silently typing into an unrelated buffer.
    pub fn current_input_buffer(&mut self) -> Option<&mut String> {
        match self.input_field {
            InputField::Name => Some(&mut self.edit_name),
            InputField::PassingGrade => Some(&mut self.edit_passing_grade),
            InputField::Credits => Some(&mut self.edit_credits),
            InputField::ClassesTotal => Some(&mut self.edit_classes_total),
            InputField::ClassesMissed => Some(&mut self.edit_classes_missed),
            InputField::AttendanceRequired => Some(&mut self.edit_attendance_required),
            InputField::Weight => Some(&mut self.edit_weight),
            InputField::Grade => Some(&mut self.edit_grade),
            InputField::Description => Some(&mut self.edit_description),
            InputField::EvalWeight => Some(&mut self.edit_eval_weight),
            InputField::MinimumAverage => Some(&mut self.edit_min_average),
            InputField::MinPerEval => Some(&mut self.edit_min_per_eval),
            InputField::MinOneEval => Some(&mut self.edit_min_one_eval),
            InputField::GlobalSemesterWeight => Some(&mut self.edit_global_semester_weight),
            InputField::GlobalExamWeight => Some(&mut self.edit_global_exam_weight),
            InputField::GlobalMinGrade => Some(&mut self.edit_global_min_grade),
            // Toggle fields don't have text buffers — they are cycled, not typed into.
            // This branch should never be reached in practice.
            InputField::DropLowest
            | InputField::AvgMethod
            | InputField::OnMinNotMet
            | InputField::OnMinPerEvalNotMet
            | InputField::OnMinOneEvalNotMet
            | InputField::RoundBeforeWeight
            | InputField::WeightedEvals
            | InputField::AttendanceAction
            | InputField::GlobalPolicy => None,
        }
    }

    /// Cycle the current toggle field forward (→ / Space / Enter / l).
    pub fn cycle_toggle_field(&mut self) {
        match self.input_field {
            InputField::DropLowest => {
                self.edit_drop_lowest = (self.edit_drop_lowest + 1) % (MAX_DROP_LOWEST + 1);
            }
            InputField::AvgMethod => {
                self.edit_averaging_method = match self.edit_averaging_method {
                    AveragingMethod::Arithmetic => AveragingMethod::Geometric,
                    AveragingMethod::Geometric => AveragingMethod::Arithmetic,
                };
            }
            InputField::OnMinNotMet => {
                self.edit_on_min_not_met = self.edit_on_min_not_met.next();
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = self.edit_on_min_per_eval_not_met.next();
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = self.edit_on_min_one_eval_not_met.next();
            }
            InputField::RoundBeforeWeight => {
                self.edit_round_before_weighting = !self.edit_round_before_weighting;
            }
            InputField::WeightedEvals => {
                self.edit_weighted_evaluations = !self.edit_weighted_evaluations;
            }
            InputField::AttendanceAction => {
                self.edit_attendance_action = match self.edit_attendance_action {
                    AttendanceAction::WarnOnly => AttendanceAction::FailCourse,
                    AttendanceAction::FailCourse => AttendanceAction::WarnOnly,
                };
            }
            InputField::GlobalPolicy => {
                self.edit_global_policy = match self.edit_global_policy {
                    GlobalExamPolicy::None => GlobalExamPolicy::Weighted {
                        semester_weight: 0.7,
                        global_weight: 0.3,
                    },
                    GlobalExamPolicy::Weighted { .. } => GlobalExamPolicy::ReplacesWorstGrade,
                    GlobalExamPolicy::ReplacesWorstGrade => GlobalExamPolicy::None,
                };
            }
            _ => {}
        }
    }

    /// Cycle the current toggle field backward (← / h).
    pub fn cycle_toggle_field_reverse(&mut self) {
        match self.input_field {
            InputField::DropLowest => {
                self.edit_drop_lowest = if self.edit_drop_lowest == 0 {
                    MAX_DROP_LOWEST
                } else {
                    self.edit_drop_lowest - 1
                };
            }
            // Two-state toggles: reverse == forward
            InputField::AvgMethod | InputField::RoundBeforeWeight | InputField::WeightedEvals => {
                self.cycle_toggle_field()
            }
            // Three-state toggle: reverse cycle
            InputField::OnMinNotMet => {
                self.edit_on_min_not_met = self.edit_on_min_not_met.previous();
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = self.edit_on_min_per_eval_not_met.previous();
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = self.edit_on_min_one_eval_not_met.previous();
            }
            // Three-state toggle: reverse cycle
            InputField::GlobalPolicy => {
                self.edit_global_policy = match self.edit_global_policy {
                    GlobalExamPolicy::None => GlobalExamPolicy::ReplacesWorstGrade,
                    GlobalExamPolicy::ReplacesWorstGrade => GlobalExamPolicy::Weighted {
                        semester_weight: 0.7,
                        global_weight: 0.3,
                    },
                    GlobalExamPolicy::Weighted { .. } => GlobalExamPolicy::None,
                };
            }
            _ => {}
        }
    }
}
