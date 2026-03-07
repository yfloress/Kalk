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

//! Global exam logic for courses.
//!
//! This module contains all methods on `Course` related to the global exam:
//! computing the post-global grade, checking eligibility, calculating the
//! needed global grade, resolving the target category, and simulating grade
//! replacement. Extracted from `model/mod.rs` to keep file sizes manageable.

use super::{
    AveragingMethod, Course, GlobalExamPolicy, MAX_GRADE, MIN_GRADE, MinimumNotMetAction,
    NeededGrade,
};

impl Course {
    // =========================================================================
    // Global Exam — Post-Global Grade
    // =========================================================================

    /// Compute the final grade after applying the global exam.
    ///
    /// Returns `None` if no global exam has been taken or the course has no
    /// global policy. Returns `Some(grade)` with the recalculated grade.
    pub(crate) fn compute_grade_after_global(&self, semester_grade: f64) -> Option<f64> {
        let global_grade = self.global_exam_grade?;

        match &self.global_policy {
            GlobalExamPolicy::None => None,
            GlobalExamPolicy::Weighted {
                semester_weight,
                global_weight,
            } => {
                let final_grade = semester_grade * semester_weight + global_grade * global_weight;
                Some(final_grade)
            }
            GlobalExamPolicy::ReplacesWorstGrade => {
                // Find the target category (explicit or auto-detected)
                let target_idx = self.resolve_global_target_category()?;
                let target_cat = self.categories.get(target_idx)?;

                // Find the worst graded evaluation in the target category
                let worst_idx = target_cat
                    .evaluations
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.grade.is_some())
                    .min_by(|(_, a), (_, b)| {
                        a.grade
                            .unwrap()
                            .partial_cmp(&b.grade.unwrap())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)?;

                // Simulate replacing the worst grade with the global grade
                // and recalculate the course grade from scratch
                self.recalculate_with_replaced_grade(target_idx, worst_idx, global_grade)
            }
        }
    }

    // =========================================================================
    // Global Exam — Target Category Resolution
    // =========================================================================

    /// Resolve which category the global exam targets.
    ///
    /// If `global_target_category` is set explicitly, use it. Otherwise,
    /// auto-detect by finding the first category whose `RequiresGlobal` rule
    /// was triggered (failed its minimum).
    pub(crate) fn resolve_global_target_category(&self) -> Option<usize> {
        if let Some(idx) = self.global_target_category
            && idx < self.categories.len()
        {
            return Some(idx);
        }

        // Auto-detect: find the first category with RequiresGlobal that fails
        for (idx, cat) in self.categories.iter().enumerate() {
            if cat.rules.on_minimum_not_met == MinimumNotMetAction::RequiresGlobal {
                // Check if this category actually fails its minimum
                let fails_avg = if let Some(min_avg) = cat.rules.minimum_average
                    && let Some(avg) = cat.average_grade()
                {
                    avg < min_avg
                } else {
                    false
                };

                let fails_per_eval = if let Some(failing) = cat.evals_below_minimum() {
                    !failing.is_empty()
                } else {
                    false
                };

                if fails_avg || fails_per_eval {
                    return Some(idx);
                }
            }
        }

        // If no category explicitly triggers RequiresGlobal but there IS a
        // global policy, default to the first category (common for
        // ReplacesWorstGrade where the user can take the global voluntarily).
        if !self.categories.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    // =========================================================================
    // Global Exam — Grade Replacement Simulation
    // =========================================================================

    /// Recalculate the course grade with one evaluation replaced.
    ///
    /// Creates a temporary copy of the category with the specified evaluation's
    /// grade replaced, then recomputes the weighted sum across all categories.
    fn recalculate_with_replaced_grade(
        &self,
        cat_idx: usize,
        eval_idx: usize,
        new_grade: f64,
    ) -> Option<f64> {
        let mut total = 0.0;

        for (ci, cat) in self.categories.iter().enumerate() {
            if ci == cat_idx {
                // Build a temporary category with the replaced grade
                let mut temp_cat = cat.clone();
                if let Some(eval) = temp_cat.evaluations.get_mut(eval_idx) {
                    eval.grade = Some(new_grade.clamp(MIN_GRADE, MAX_GRADE));
                }
                total += temp_cat.weighted_contribution().unwrap_or(0.0);
            } else {
                total += cat.weighted_contribution().unwrap_or(0.0);
            }
        }

        Some(total)
    }

    // =========================================================================
    // Global Exam — Eligibility
    // =========================================================================

    /// Check if the student is eligible to take the global exam.
    ///
    /// Returns `false` when the course has no global policy. When eligibility
    /// restrictions are configured, the semester grade must fall within the
    /// allowed range.
    pub fn is_eligible_for_global(&self) -> bool {
        if self.global_policy == GlobalExamPolicy::None {
            return false;
        }

        let semester_grade = self.compute_grade().grade;
        let elig = &self.global_eligibility;

        if let Some(min) = elig.min_grade
            && semester_grade < min
        {
            return false;
        }
        if let Some(max) = elig.max_grade
            && semester_grade > max
        {
            return false;
        }

        true
    }

    // =========================================================================
    // Global Exam — Needed Grade Calculation
    // =========================================================================

    /// Calculate the grade needed in the global exam to pass the course.
    pub fn needed_global_grade(&self) -> NeededGrade {
        if self.global_policy == GlobalExamPolicy::None {
            return NeededGrade::info();
        }

        let semester_grade = self.compute_grade().grade;

        // If already took the global, report result
        if let Some(global_grade) = self.global_exam_grade {
            let after = self.compute_grade().grade_after_global.unwrap_or(0.0);
            return if self.is_passing_grade(after) {
                NeededGrade::success(global_grade)
            } else {
                NeededGrade::failure(Some(global_grade))
            };
        }

        // Check eligibility
        if !self.is_eligible_for_global() {
            return NeededGrade::failure(None);
        }

        match &self.global_policy {
            GlobalExamPolicy::None => NeededGrade::info(),
            GlobalExamPolicy::Weighted {
                semester_weight,
                global_weight,
            } => {
                if global_weight.abs() < f64::EPSILON {
                    return NeededGrade::failure(None);
                }
                // semester_grade * sw + X * gw >= passing - 0.5
                let effective_passing = self.passing_grade - 0.5;
                let needed = (effective_passing - semester_grade * semester_weight) / global_weight;

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
            GlobalExamPolicy::ReplacesWorstGrade => self.needed_global_replaces_worst(),
        }
    }

    /// Calculate needed global grade for the `ReplacesWorstGrade` policy.
    ///
    /// Solves algebraically for the grade X that, when replacing the worst
    /// evaluation in the target category, produces a passing course grade.
    fn needed_global_replaces_worst(&self) -> NeededGrade {
        let target_idx = match self.resolve_global_target_category() {
            Some(idx) => idx,
            None => return NeededGrade::failure(None),
        };
        let target_cat = match self.categories.get(target_idx) {
            Some(cat) => cat,
            None => return NeededGrade::failure(None),
        };

        // Find the worst graded evaluation
        let worst_idx = match target_cat
            .evaluations
            .iter()
            .enumerate()
            .filter(|(_, e)| e.grade.is_some())
            .min_by(|(_, a), (_, b)| {
                a.grade
                    .unwrap()
                    .partial_cmp(&b.grade.unwrap())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        {
            Some(i) => i,
            None => return NeededGrade::failure(None),
        };

        // For arithmetic mean: solve algebraically
        // The target category has effective_count evals after drop_lowest.
        // Replacing worst with X changes the category average, which changes
        // the weighted course grade.
        if target_cat.rules.averaging_method == AveragingMethod::Geometric {
            return NeededGrade::info();
        }

        let grades = target_cat.effective_grades();
        if grades.is_empty() {
            return NeededGrade::info();
        }

        let effective_count = grades.len();
        let worst_grade = target_cat.evaluations[worst_idx].grade.unwrap_or(0.0);

        // Sum of all other categories' contributions (unchanged)
        let mut other_contribution = 0.0;
        for (ci, cat) in self.categories.iter().enumerate() {
            if ci != target_idx {
                other_contribution += cat.weighted_contribution().unwrap_or(0.0);
            }
        }

        // In the target category, replacing worst with X:
        // new_avg = (sum_of_grades - worst + X) / effective_count
        // new_contribution = new_avg * weight / 100
        // total = other_contribution + new_contribution >= passing - 0.5
        let sum_of_grades: f64 = grades.iter().sum();
        let cat_weight = target_cat.weight;
        let effective_passing = self.passing_grade - 0.5;

        // other_contribution + ((sum - worst + X) / count) * weight/100 >= eff_passing
        // ((sum - worst + X) / count) * weight/100 >= eff_passing - other_contribution
        // (sum - worst + X) / count >= (eff_passing - other_contribution) * 100 / weight
        // sum - worst + X >= count * (eff_passing - other_contribution) * 100 / weight
        // X >= count * (eff_passing - other_contribution) * 100 / weight - sum + worst
        if cat_weight.abs() < f64::EPSILON {
            return NeededGrade::failure(None);
        }

        let needed = effective_count as f64 * (effective_passing - other_contribution) * 100.0
            / cat_weight
            - sum_of_grades
            + worst_grade;

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
