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

//! What a course still asks of its open evaluations: the exact solver for a
//! single evaluation, and the verdict types both it and the spread solver in
//! `outlook` return.

use super::{AveragingMethod, Category, Course, GlobalExamPolicy, MAX_GRADE, MinimumNotMetAction};

// =============================================================================
// Needed Grade Calculation
// =============================================================================

/// Result status for a needed-grade calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeededGradeStatus {
    /// Already passing or needs 0+
    Success,
    /// Cannot pass (needs > 100 or impossible)
    Failure,
    /// Needs a specific achievable grade
    Warning,
    /// Informational (no evaluations, etc.)
    Info,
    /// No grade passes the semester outright, but the global exam still can.
    GlobalOnly,
}

/// Result of calculating the grade needed in a specific evaluation to pass.
#[derive(Debug, Clone)]
pub struct NeededGrade {
    /// The exact grade needed (may be negative, > 100, or NaN for edge cases)
    pub value: Option<f64>,
    pub status: NeededGradeStatus,
    /// How many evaluations must each reach `value`: 1 for a single
    /// evaluation, more when the requirement is spread over all that remain.
    pub spread_over: usize,
}

impl NeededGrade {
    pub(super) fn success(value: f64) -> Self {
        Self {
            value: Some(value),
            status: NeededGradeStatus::Success,
            spread_over: 1,
        }
    }

    pub(super) fn failure(value: Option<f64>) -> Self {
        Self {
            value,
            status: NeededGradeStatus::Failure,
            spread_over: 1,
        }
    }

    pub(super) fn warning(value: f64) -> Self {
        Self {
            value: Some(value),
            status: NeededGradeStatus::Warning,
            spread_over: 1,
        }
    }

    pub(super) fn info() -> Self {
        Self {
            value: None,
            status: NeededGradeStatus::Info,
            spread_over: 1,
        }
    }

    pub(super) fn global_only() -> Self {
        Self {
            value: None,
            status: NeededGradeStatus::GlobalOnly,
            spread_over: 1,
        }
    }

    pub(super) fn spread_over(mut self, count: usize) -> Self {
        self.spread_over = count;
        self
    }
}

impl Course {
    // =========================================================================
    // Needed Grade
    // =========================================================================

    /// The category's `minimum_average` requirement, but only when failing it
    /// actually blocks passing the course — in which case the needed-grade
    /// solver must also satisfy it, not just the weighted course total.
    ///
    /// Binding cases (mirroring [`Self::compute_grade`]):
    /// - `FailCourse`: breaking it caps the course grade at 0. Always binding.
    /// - `FinalEqualsAverage`: the final grade becomes the category average, so
    ///   it only blocks passing when the minimum sits at or below the passing
    ///   grade (otherwise the capped average could itself still pass).
    /// - `RequiresGlobal`: only binding when no global exam is configured, since
    ///   `compute_grade` then degrades it to the `FinalEqualsAverage` failure.
    fn binding_category_minimum(&self, category: &Category) -> Option<f64> {
        let min = category.rules.minimum_average?;
        let binds = match category.rules.on_minimum_not_met {
            MinimumNotMetAction::FailCourse => true,
            MinimumNotMetAction::FinalEqualsAverage => min <= self.passing_grade,
            MinimumNotMetAction::RequiresGlobal => {
                self.global_policy == GlobalExamPolicy::None && min <= self.passing_grade
            }
            // Only binding when the ceiling itself sits below passing: a cap
            // above the passing grade still leaves room to pass.
            MinimumNotMetAction::CapFinalGrade => category
                .rules
                .cap_final_grade
                .is_some_and(|cap| cap < self.passing_grade),
        };
        binds.then_some(min)
    }

    /// Whether the course could still pass if this evaluation scored full
    /// marks and everything else went perfectly, global exam included.
    fn can_still_pass_with_perfect(&self, category_idx: usize, eval_idx: usize) -> bool {
        let mut ideal = self.clone();
        let Some(evaluation) = ideal
            .categories
            .get_mut(category_idx)
            .and_then(|c| c.evaluations.get_mut(eval_idx))
        else {
            // Bad indices are the caller's problem; the normal path reports it.
            return true;
        };
        evaluation.grade = Some(MAX_GRADE);

        ideal
            .best_case_grade()
            .is_none_or(|best| ideal.is_passing_grade(best))
    }

    /// The lowest grade this evaluation must reach so that no category rule
    /// whose failure blocks passing is broken. Returns the highest such floor,
    /// or `None` when no rule constrains this evaluation.
    ///
    /// Covered rules:
    /// - `minimum_average`: via [`Self::binding_category_minimum`] (all three
    ///   not-met actions, with the appropriate gating).
    /// - `minimum_per_evaluation` with `FailCourse`: every evaluation must reach
    ///   the threshold, so this one must too.
    /// - `minimum_one_eval` with `FailCourse`: at least one evaluation must reach
    ///   the threshold — binding here only when no *other* evaluation already
    ///   does.
    ///
    /// The per-eval and one-eval rules are only floored for `FailCourse`, where
    /// the consequence is unconditional. The `FinalEqualsAverage` /
    /// `RequiresGlobal` actions cap the grade at the (grade-dependent) category
    /// average, which is not soundly expressible as a fixed per-eval floor, so
    /// they are intentionally left out rather than risk a wrong estimate.
    fn pass_blocking_floor(&self, category: &Category, eval_idx: usize) -> Option<f64> {
        let mut floors: Vec<f64> = Vec::new();

        if let Some(min) = self.binding_category_minimum(category)
            && let Some(need) = category.needed_in_eval_for_average(eval_idx, min)
        {
            floors.push(need);
        }

        if category.rules.on_min_per_eval_not_met == MinimumNotMetAction::FailCourse
            && let Some(min) = category.rules.minimum_per_evaluation
        {
            floors.push(min);
        }

        if category.rules.on_min_one_eval_not_met == MinimumNotMetAction::FailCourse
            && let Some(min) = category.rules.minimum_one_eval
            && !category.other_eval_meets(eval_idx, min)
        {
            floors.push(min);
        }

        floors
            .into_iter()
            .fold(None, |acc, v| Some(acc.map_or(v, |a: f64| a.max(v))))
    }

    /// Whether the course passes when evaluation `(cat_idx, eval_idx)` is set to
    /// `grade`, leaving every other grade untouched. Used by the needed-grade
    /// solver to detect when nothing is required of this evaluation — for
    /// instance a `drop_lowest` category where it would be discarded anyway, so
    /// the course already passes on the strength of the others.
    fn passes_with_eval(&self, cat_idx: usize, eval_idx: usize, grade: f64) -> bool {
        self.passes_with_open_at(&[(cat_idx, eval_idx)], grade)
    }

    /// Calculate the grade needed in a specific evaluation to pass the course.
    ///
    /// When `ignore_current_grade` is true, the current eval's grade is treated
    /// as 0 (used when editing to show what's needed regardless of existing grade).
    ///
    /// The result accounts for both the weighted course total *and* any
    /// category rule whose failure would block passing (see
    /// [`Self::pass_blocking_floor`]): the needed grade is the larger of the
    /// two, so a hurdle that would fail the course is never under-reported.
    pub fn needed_grade_for_evaluation(
        &self,
        category_idx: usize,
        eval_idx: usize,
        ignore_current_grade: bool,
    ) -> NeededGrade {
        let needed =
            self.needed_grade_ignoring_ceilings(category_idx, eval_idx, ignore_current_grade);

        // A ceiling set elsewhere — a capped category, a failed attendance
        // requirement — can put passing out of reach no matter what this
        // evaluation scores. Keep the number, but stop calling it achievable.
        if matches!(
            needed.status,
            NeededGradeStatus::Success | NeededGradeStatus::Warning
        ) && !self.can_still_pass_with_perfect(category_idx, eval_idx)
        {
            return NeededGrade::failure(needed.value);
        }

        needed
    }

    fn needed_grade_ignoring_ceilings(
        &self,
        category_idx: usize,
        eval_idx: usize,
        ignore_current_grade: bool,
    ) -> NeededGrade {
        let Some(category) = self.categories.get(category_idx) else {
            return NeededGrade::failure(None);
        };

        let Some(eval) = category.evaluations.get(eval_idx) else {
            return NeededGrade::failure(None);
        };

        if category.evaluations.is_empty() {
            return NeededGrade::info();
        }

        // If already graded and we're NOT ignoring current grade, report status
        if !ignore_current_grade && let Some(grade) = eval.grade {
            return if grade >= self.passing_grade {
                NeededGrade::success(grade)
            } else {
                NeededGrade::failure(Some(grade))
            };
        }

        // For geometric mean, the needed-grade calculation is different and complex.
        // For now, only support arithmetic mean in needed-grade calculation.
        if category.rules.averaging_method == AveragingMethod::Geometric {
            return NeededGrade::info();
        }

        // If the course already passes with this evaluation contributing 0
        // (e.g. a drop_lowest category where it would be discarded), nothing is
        // required here regardless of what the linear solve would suggest.
        if self.passes_with_eval(category_idx, eval_idx, 0.0) {
            return NeededGrade::success(0.0);
        }

        // Floors imposed by category rules whose failure blocks passing
        // (minimum_average, plus FailCourse per-eval / one-eval minimums).
        let floor = self.pass_blocking_floor(category, eval_idx);

        // Categories with zero weight don't move the weighted total, but a
        // pass-blocking rule can still force a needed grade (e.g. a 0%-weight
        // hurdle category that fails the course on its own).
        if category.weight.abs() < f64::EPSILON {
            return match floor {
                Some(f) => Self::classify_needed(f),
                None => NeededGrade::failure(None),
            };
        }

        // Weighted evaluations mode: each eval has its own weight fraction
        if category.rules.weighted_evaluations {
            let eval_w = eval.weight.unwrap_or(0.0);
            if eval_w.abs() < f64::EPSILON {
                return NeededGrade::failure(None);
            }

            // Sum of (grade * eval_weight/100) for all OTHER evaluations in this category
            let other_contribution: f64 = category
                .evaluations
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != eval_idx)
                .map(|(_, e)| {
                    let g = e.grade.unwrap_or(0.0);
                    let w = e.weight.unwrap_or(0.0) / 100.0;
                    g * w
                })
                .sum();

            // Contribution from all OTHER categories
            let mut total_contribution = other_contribution * category.weight / 100.0;
            for (ci, cat) in self.categories.iter().enumerate() {
                if ci != category_idx
                    && !cat.evaluations.is_empty()
                    && let Some(contribution) = cat.weighted_contribution()
                {
                    total_contribution += contribution;
                }
            }

            // Weight of this single evaluation in the final grade:
            // (eval_w / 100) * (category.weight / 100)
            let final_eval_weight = (eval_w / 100.0) * (category.weight / 100.0);
            if final_eval_weight.abs() < f64::EPSILON {
                return NeededGrade::failure(None);
            }

            let effective_passing = self.passing_grade - 0.5;
            let needed = (effective_passing - total_contribution) / final_eval_weight;
            let needed = Self::floored(needed, floor);
            return Self::classify_needed(needed);
        }

        // Equal-weight mode (original behavior)
        let (other_grades, effective_count) = category.effective_grades_excluding(eval_idx);
        if effective_count == 0 {
            return NeededGrade::info();
        }

        // Calculate total contribution from all OTHER categories
        let mut total_contribution = 0.0;

        for (ci, cat) in self.categories.iter().enumerate() {
            if ci == category_idx {
                // For the target category, use the pre-computed grades excluding target eval
                let other_sum: f64 = other_grades.iter().sum();
                total_contribution += other_sum * cat.weight / (100.0 * effective_count as f64);
            } else if !cat.evaluations.is_empty()
                && let Some(contribution) = cat.weighted_contribution()
            {
                total_contribution += contribution;
            }
        }

        // Weight of this single evaluation in the final grade
        let eval_weight = category.weight / (100.0 * effective_count as f64);

        if eval_weight.abs() < f64::EPSILON {
            return NeededGrade::failure(None);
        }

        // We need: total_contribution + (needed_grade * eval_weight) >= passing - 0.5
        // Because 0.5+ rounds up (54.5 rounds to 55)
        let effective_passing = self.passing_grade - 0.5;
        let needed = (effective_passing - total_contribution) / eval_weight;
        let needed = Self::floored(needed, floor);
        Self::classify_needed(needed)
    }

    /// Raise `needed` to `floor` when a pass-blocking category rule requires a
    /// higher grade than the weighted total alone. Returns `needed` unchanged
    /// when there is no such floor.
    fn floored(needed: f64, floor: Option<f64>) -> f64 {
        match floor {
            Some(f) => needed.max(f),
            None => needed,
        }
    }

    /// Turn a raw "needed grade" value into a [`NeededGrade`] verdict:
    /// non-finite or unreachable (> [`MAX_GRADE`]) → failure, already satisfied
    /// (<= 0) → success, otherwise a warning carrying the value.
    fn classify_needed(needed: f64) -> NeededGrade {
        if !needed.is_finite() {
            NeededGrade::failure(None)
        } else if needed <= 0.0 {
            NeededGrade::success(0.0)
        } else if needed > MAX_GRADE {
            NeededGrade::failure(Some(needed))
        } else {
            NeededGrade::warning(needed)
        }
    }
}
