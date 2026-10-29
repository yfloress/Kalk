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

use super::{Course, GlobalExamPolicy, MAX_GRADE, MIN_GRADE, NeededGrade};

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
        // Before the first grade the shown grade is 0, so the distance would
        // read -55 on every course in a fresh semester: arithmetically true,
        // and worth nothing. The gap only means something once something has
        // actually been graded.
        if !self.has_any_grade() {
            return None;
        }
        self.final_grade()
            .map(|grade| Self::round_grade(grade) - self.passing_grade)
    }

    /// Whether any evaluation in the course carries a grade yet.
    pub fn has_any_grade(&self) -> bool {
        self.categories
            .iter()
            .any(|c| c.evaluations.iter().any(|e| e.grade.is_some()))
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

    /// What evaluation `(category_idx, eval_idx)` asks for, counting every
    /// other evaluation still open.
    ///
    /// When it is the only one left, the exact single-evaluation solver
    /// answers. Otherwise the requirement is spread evenly over all of them:
    /// solving for one while the rest count as zero makes that one carry the
    /// whole course, which early in a semester asks for 200 or more.
    pub fn needed_grade_ahead(
        &self,
        category_idx: usize,
        eval_idx: usize,
        ignore_current_grade: bool,
    ) -> NeededGrade {
        let graded = self
            .categories
            .get(category_idx)
            .and_then(|c| c.evaluations.get(eval_idx))
            .is_some_and(|e| e.grade.is_some());
        if graded && !ignore_current_grade {
            return self.needed_grade_for_evaluation(category_idx, eval_idx, false);
        }

        if self.open_evaluations(Some((category_idx, eval_idx))).len() <= 1 {
            return self.needed_grade_for_evaluation(category_idx, eval_idx, ignore_current_grade);
        }
        self.needed_across_open(Some((category_idx, eval_idx)))
    }

    /// The even grade needed on every ungraded evaluation (plus `target`, if
    /// given) to pass without the global exam.
    pub fn needed_across_open(&self, target: Option<(usize, usize)>) -> NeededGrade {
        let open = self.open_evaluations(target);
        if open.is_empty() {
            return NeededGrade::info();
        }
        let passes_at = |grade: f64| self.passes_with_open_at(&open, grade);

        if passes_at(MIN_GRADE) {
            return NeededGrade::success(0.0).spread_over(open.len());
        }
        if !passes_at(MAX_GRADE) {
            // No grade passes the semester outright. Either the global can
            // still rescue it or nothing can.
            return if self.is_unrecoverable() {
                NeededGrade::failure(None)
            } else {
                NeededGrade::global_only()
            };
        }

        // Raising grades never lowers the final grade nor breaks a rule, so
        // the pass/fail boundary is a single point and bisection finds it.
        let (mut low, mut high) = (MIN_GRADE, MAX_GRADE);
        for _ in 0..30 {
            let mid = (low + high) / 2.0;
            if passes_at(mid) {
                high = mid;
            } else {
                low = mid;
            }
        }
        NeededGrade::warning(high).spread_over(open.len())
    }

    /// Every ungraded evaluation as `(category, evaluation)`, plus `target`
    /// when it exists and is not already among them.
    fn open_evaluations(&self, target: Option<(usize, usize)>) -> Vec<(usize, usize)> {
        let mut open: Vec<(usize, usize)> = self
            .categories
            .iter()
            .enumerate()
            .flat_map(|(ci, category)| {
                category
                    .evaluations
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.grade.is_none())
                    .map(move |(ei, _)| (ci, ei))
            })
            .collect();

        if let Some((ci, ei)) = target
            && self
                .categories
                .get(ci)
                .is_some_and(|c| ei < c.evaluations.len())
            && !open.contains(&(ci, ei))
        {
            open.push((ci, ei));
        }
        open
    }

    /// Whether the course passes without the global exam when every
    /// evaluation in `open` scores `grade` and the rest keep theirs.
    pub(super) fn passes_with_open_at(&self, open: &[(usize, usize)], grade: f64) -> bool {
        let mut probe = self.clone();
        for &(ci, ei) in open {
            if let Some(evaluation) = probe
                .categories
                .get_mut(ci)
                .and_then(|c| c.evaluations.get_mut(ei))
            {
                evaluation.grade = Some(grade);
            }
        }
        let result = probe.compute_grade();
        probe.is_passing_grade(result.grade) && !result.needs_global
    }
}
