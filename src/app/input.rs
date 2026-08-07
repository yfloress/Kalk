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
use crate::model::{AveragingMethod, GlobalExamPolicy, MinimumNotMetAction};

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
            (Screen::SavingTemplate, InputField::Name) => InputField::Description,
            (Screen::SavingTemplate, InputField::Description) => InputField::Name,
            _ => self.input_field,
        };
    }

    pub fn current_input_buffer(&mut self) -> &mut String {
        match self.input_field {
            InputField::Name => &mut self.edit_name,
            InputField::PassingGrade => &mut self.edit_passing_grade,
            InputField::Credits => &mut self.edit_credits,
            InputField::Weight => &mut self.edit_weight,
            InputField::Grade => &mut self.edit_grade,
            InputField::Description => &mut self.edit_description,
            InputField::EvalWeight => &mut self.edit_eval_weight,
            InputField::MinimumAverage => &mut self.edit_min_average,
            InputField::MinPerEval => &mut self.edit_min_per_eval,
            InputField::MinOneEval => &mut self.edit_min_one_eval,
            InputField::GlobalSemesterWeight => &mut self.edit_global_semester_weight,
            InputField::GlobalExamWeight => &mut self.edit_global_exam_weight,
            InputField::GlobalMinGrade => &mut self.edit_global_min_grade,
            // Toggle fields don't have text buffers — they are cycled, not typed into.
            // This branch should never be reached in practice.
            InputField::DropLowest
            | InputField::AvgMethod
            | InputField::OnMinNotMet
            | InputField::OnMinPerEvalNotMet
            | InputField::OnMinOneEvalNotMet
            | InputField::RoundBeforeWeight
            | InputField::WeightedEvals
            | InputField::GlobalPolicy => {
                debug_assert!(
                    false,
                    "current_input_buffer called on toggle field {:?}",
                    self.input_field
                );
                &mut self.edit_description
            }
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
                self.edit_on_min_not_met = match self.edit_on_min_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = match self.edit_on_min_per_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = match self.edit_on_min_one_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::RoundBeforeWeight => {
                self.edit_round_before_weighting = !self.edit_round_before_weighting;
            }
            InputField::WeightedEvals => {
                self.edit_weighted_evaluations = !self.edit_weighted_evaluations;
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
                self.edit_on_min_not_met = match self.edit_on_min_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinPerEvalNotMet => {
                self.edit_on_min_per_eval_not_met = match self.edit_on_min_per_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
            }
            InputField::OnMinOneEvalNotMet => {
                self.edit_on_min_one_eval_not_met = match self.edit_on_min_one_eval_not_met {
                    MinimumNotMetAction::FinalEqualsAverage => MinimumNotMetAction::FailCourse,
                    MinimumNotMetAction::FailCourse => MinimumNotMetAction::RequiresGlobal,
                    MinimumNotMetAction::RequiresGlobal => MinimumNotMetAction::FinalEqualsAverage,
                };
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
