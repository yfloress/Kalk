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
//! Where a course is headed: the ceiling it can still reach, how much room is
//! left, and how far it sits from passing.
//!
//! The grade Kalk shows is already the floor — ungraded evaluations count as
//! zero — so these answer the other half: the best case, and what is still in
//! play to get there.

use super::{Course, GlobalExamPolicy, MAX_GRADE};

impl Course {
    /// The highest final grade still reachable: full marks on everything
    /// ungraded, including the global exam when one is configured.
    ///
    /// `None` when the course has no evaluations to judge.
    pub fn best_case_grade(&self) -> Option<f64> {
        if !self.has_evaluations() {
            return None;
        }

        // Recomputed on a filled-in copy so every category rule (drop lowest,
        // minimums, global policy) applies to the best case exactly as it
        // does to the current standing.
        let mut ideal = self.clone();
        for category in &mut ideal.categories {
            for evaluation in &mut category.evaluations {
                if evaluation.grade.is_none() {
                    evaluation.grade = Some(MAX_GRADE);
                }
            }
        }
        if ideal.global_policy != GlobalExamPolicy::None && ideal.global_exam_grade.is_none() {
            ideal.global_exam_grade = Some(MAX_GRADE);
        }

        ideal.final_grade()
    }

    /// True when even full marks on everything left cannot reach the passing
    /// grade — the course is arithmetically over, not merely going badly.
    pub fn is_unrecoverable(&self) -> bool {
        self.best_case_grade()
            .is_some_and(|best| !self.is_passing_grade(best))
    }

    /// Points between the current standing and the passing grade: positive is
    /// margin to spare, negative is the gap left to close.
    ///
    /// Uses the rounded grade, so it agrees with the pass/fail verdict.
    pub fn margin(&self) -> Option<f64> {
        self.final_grade()
            .map(|grade| Self::round_grade(grade) - self.passing_grade)
    }

    /// Share of the final grade (0-100) that no evaluation has settled yet.
    ///
    /// A category with no evaluations counts as entirely pending: nothing in
    /// it has been decided. Categories that drop their lowest grades make this
    /// an upper bound rather than an exact figure.
    pub fn pending_weight(&self) -> f64 {
        self.categories
            .iter()
            .map(|category| {
                if category.evaluations.is_empty() {
                    return category.weight;
                }

                let pending_share = if category.rules.weighted_evaluations {
                    let total = category.total_eval_weight();
                    if total <= 0.0 {
                        1.0
                    } else {
                        let pending: f64 = category
                            .evaluations
                            .iter()
                            .filter(|e| e.grade.is_none())
                            .map(|e| e.weight.unwrap_or(0.0))
                            .sum();
                        pending / total
                    }
                } else {
                    let pending = category.evaluations.len() - category.graded_count();
                    pending as f64 / category.evaluations.len() as f64
                };

                category.weight * pending_share
            })
            .sum()
    }

    /// First evaluation still without a grade, as `(category, evaluation)`.
    /// This is the one a "what do I need" hint should talk about.
    pub fn next_ungraded(&self) -> Option<(usize, usize)> {
        self.categories
            .iter()
            .enumerate()
            .find_map(|(ci, category)| {
                category
                    .evaluations
                    .iter()
                    .position(|e| e.grade.is_none())
                    .map(|ei| (ci, ei))
            })
    }
}
