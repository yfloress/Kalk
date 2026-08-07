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

//! Category domain model and rules.
//!
//! A `Category` groups evaluations under a weighted section of a course
//! (e.g., "Certamenes 80%", "Controles 20%"). Each category can have
//! optional `CategoryRules` that modify how its grade is calculated
//! (drop lowest, geometric mean, minimum average, etc.).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Evaluation;

// =============================================================================
// Averaging Method
// =============================================================================

/// How to average evaluations within a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AveragingMethod {
    /// Standard arithmetic mean: sum / count
    #[default]
    Arithmetic,
    /// Geometric mean: (product)^(1/count) — a single 0 makes the result 0
    Geometric,
}

// =============================================================================
// Minimum-Not-Met Action
// =============================================================================

/// What happens when a category's minimum average requirement is not met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MinimumNotMetAction {
    /// Final grade = this category's average (other categories ignored)
    #[default]
    FinalEqualsAverage,
    /// Student must take Certamen Global (handled at course level)
    RequiresGlobal,
    /// Course is automatically failed (grade capped at 0)
    FailCourse,
    /// The final grade may not exceed `CategoryRules::cap_final_grade`.
    /// Models rules like "if the tests average 50 or less, the course caps
    /// at 54" — failing without pretending the other grades never happened.
    CapFinalGrade,
}

impl MinimumNotMetAction {
    /// Every action, in the order the form cycles through them.
    pub const ALL: [MinimumNotMetAction; 4] = [
        MinimumNotMetAction::FinalEqualsAverage,
        MinimumNotMetAction::RequiresGlobal,
        MinimumNotMetAction::FailCourse,
        MinimumNotMetAction::CapFinalGrade,
    ];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

// =============================================================================
// Category Rules
// =============================================================================

/// Rules that modify how a category's grade is calculated.
///
/// All fields use `#[serde(default)]` for backward compatibility —
/// existing JSON data without rules will deserialize with sensible defaults
/// that produce identical behavior to the old system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryRules {
    /// Minimum average required in this category (None = no requirement).
    /// If the average is below this, `on_minimum_not_met` determines the consequence.
    #[serde(default)]
    pub minimum_average: Option<f64>,

    /// What happens when the minimum average is not met.
    #[serde(default)]
    pub on_minimum_not_met: MinimumNotMetAction,

    /// Drop the N lowest grades before averaging (0 = don't drop any).
    /// If drop_lowest >= evaluation count, no grades are dropped.
    #[serde(default)]
    pub drop_lowest: usize,

    /// How to average evaluations in this category.
    #[serde(default)]
    pub averaging_method: AveragingMethod,

    /// Minimum grade required per individual evaluation (None = no requirement).
    /// If any evaluation is below this, the category is flagged.
    #[serde(default)]
    pub minimum_per_evaluation: Option<f64>,

    /// What happens when the per-evaluation minimum is not met.
    #[serde(default)]
    pub on_min_per_eval_not_met: MinimumNotMetAction,

    /// At least one graded evaluation must be >= this value (None = no requirement).
    /// If no graded evaluation meets this threshold, `on_min_one_eval_not_met` is triggered.
    #[serde(default)]
    pub minimum_one_eval: Option<f64>,

    /// What happens when no evaluation meets the minimum-one-eval threshold.
    #[serde(default)]
    pub on_min_one_eval_not_met: MinimumNotMetAction,

    /// Whether to round the category average to nearest integer before
    /// using it in the weighted course grade calculation.
    #[serde(default)]
    pub round_before_weighting: bool,

    /// Whether evaluations in this category have individual weights
    /// (must sum to 100%) instead of being averaged equally.
    #[serde(default)]
    pub weighted_evaluations: bool,

    /// Ceiling applied to the course when one of this category's rules uses
    /// `CapFinalGrade`. Shared by all three of them: no syllabus so far caps
    /// at different values per rule. `None` makes the action a no-op.
    #[serde(default)]
    pub cap_final_grade: Option<f64>,

    /// Categories that must be fully graded before this one may be sat.
    /// Purely informational — it changes no grade, it warns that the student
    /// is not entitled to the evaluation yet.
    #[serde(default)]
    pub requires_categories: Vec<usize>,
}

impl CategoryRules {
    /// Returns true if all rules are at their defaults (no special behavior).
    pub fn is_default(&self) -> bool {
        self.minimum_average.is_none()
            && self.on_minimum_not_met == MinimumNotMetAction::FinalEqualsAverage
            && self.drop_lowest == 0
            && self.averaging_method == AveragingMethod::Arithmetic
            && self.minimum_per_evaluation.is_none()
            && self.on_min_per_eval_not_met == MinimumNotMetAction::FinalEqualsAverage
            && self.minimum_one_eval.is_none()
            && self.on_min_one_eval_not_met == MinimumNotMetAction::FinalEqualsAverage
            && !self.round_before_weighting
            && !self.weighted_evaluations
            && self.cap_final_grade.is_none()
            && self.requires_categories.is_empty()
    }
}

// =============================================================================
// Category
// =============================================================================

/// Represents a category of evaluations (e.g., "Certamenes", "Controles").
/// Each category has a weight that contributes to the final course grade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    /// Weight of this category (0.0 - 100.0 percentage)
    pub weight: f64,
    pub evaluations: Vec<Evaluation>,
    /// Optional rules that modify grade calculation for this category.
    #[serde(default)]
    pub rules: CategoryRules,
}

impl Category {
    #[cfg(test)]
    pub fn new(name: String, weight: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            weight: weight.clamp(0.0, 100.0),
            evaluations: Vec::new(),
            rules: CategoryRules::default(),
        }
    }

    pub fn with_evaluations(name: String, weight: f64, evaluations: Vec<Evaluation>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            weight: weight.clamp(0.0, 100.0),
            evaluations,
            rules: CategoryRules::default(),
        }
    }

    pub fn with_rules(
        name: String,
        weight: f64,
        evaluations: Vec<Evaluation>,
        rules: CategoryRules,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            weight: weight.clamp(0.0, 100.0),
            evaluations,
            rules,
        }
    }

    // =========================================================================
    // Grade Calculation
    // =========================================================================

    /// Get the grades to use for averaging, after applying `drop_lowest`.
    /// Ungraded evaluations are treated as 0.
    /// Returns an empty Vec if there are no evaluations.
    pub(crate) fn effective_grades(&self) -> Vec<f64> {
        if self.evaluations.is_empty() {
            return Vec::new();
        }

        let mut grades: Vec<f64> = self
            .evaluations
            .iter()
            .map(|e| e.grade.unwrap_or(0.0))
            .collect();

        // Drop lowest is incompatible with weighted evaluations
        if !self.rules.weighted_evaluations
            && self.rules.drop_lowest > 0
            && self.rules.drop_lowest < grades.len()
        {
            grades.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            grades = grades[self.rules.drop_lowest..].to_vec();
        }

        grades
    }

    /// Get effective grades with their evaluation weights (for weighted mode).
    /// Returns pairs of (grade, eval_weight_fraction) where weight is 0.0-1.0.
    /// For equal-weight mode, each weight is 1/N.
    pub(crate) fn effective_grades_weighted(&self) -> Vec<(f64, f64)> {
        if self.evaluations.is_empty() {
            return Vec::new();
        }

        if self.rules.weighted_evaluations {
            self.evaluations
                .iter()
                .map(|e| {
                    let grade = e.grade.unwrap_or(0.0);
                    let w = e.weight.unwrap_or(0.0) / 100.0;
                    (grade, w)
                })
                .collect()
        } else {
            let grades = self.effective_grades();
            let n = grades.len() as f64;
            grades.into_iter().map(|g| (g, 1.0 / n)).collect()
        }
    }

    /// Grades of every evaluation EXCEPT `eval_idx`, after dropping the lowest
    /// `drop_lowest` of *those others*, paired with the effective evaluation
    /// count that INCLUDES a slot for `eval_idx` itself.
    ///
    /// Used by the needed-grade solvers, which assume the evaluation being
    /// solved for will survive `drop_lowest` — a grade high enough to pass is
    /// not the one discarded — so the dropped grades come from the other
    /// evaluations. The caller reconstructs the average as
    /// `(sum(returned) + X) / count`. Ungraded evaluations count as 0.
    pub(crate) fn effective_grades_excluding(&self, eval_idx: usize) -> (Vec<f64>, usize) {
        let mut others: Vec<f64> = self
            .evaluations
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != eval_idx)
            .map(|(_, e)| e.grade.unwrap_or(0.0))
            .collect();

        // Drop the lowest `drop_lowest` of the OTHER evaluations (incompatible
        // with weighted evaluations). Mirror `effective_grades`: only drop when
        // fewer than the total evaluation count would be dropped.
        if !self.rules.weighted_evaluations
            && self.rules.drop_lowest > 0
            && self.rules.drop_lowest < self.evaluations.len()
        {
            let n = self.rules.drop_lowest.min(others.len());
            others.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            others.drain(0..n);
        }

        let effective_count = others.len() + 1; // +1 for the target evaluation
        (others, effective_count)
    }

    /// Grade needed in evaluation `eval_idx` for this category's own average to
    /// reach `target`, mirroring the averaging mode (equal-weight or weighted
    /// evaluations) and `drop_lowest` handling used by [`Self::average_grade`].
    ///
    /// Other ungraded evaluations count as 0. Returns `None` when the
    /// evaluation cannot move the average (zero individual weight, or no
    /// effective evaluations remain). The result is not clamped — callers
    /// decide whether a value above [`MAX_GRADE`](super::MAX_GRADE) is
    /// unreachable.
    pub(crate) fn needed_in_eval_for_average(&self, eval_idx: usize, target: f64) -> Option<f64> {
        if self.rules.weighted_evaluations {
            let eval_w = self.evaluations.get(eval_idx)?.weight.unwrap_or(0.0) / 100.0;
            if eval_w.abs() < f64::EPSILON {
                return None;
            }
            // Contribution already locked in by every other evaluation.
            let other: f64 = self
                .evaluations
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != eval_idx)
                .map(|(_, e)| e.grade.unwrap_or(0.0) * (e.weight.unwrap_or(0.0) / 100.0))
                .sum();
            Some((target - other) / eval_w)
        } else {
            let (other_grades, effective_count) = self.effective_grades_excluding(eval_idx);
            if effective_count == 0 {
                return None;
            }
            // avg = (sum(others) + X) / count  >=  target
            let other_sum: f64 = other_grades.iter().sum();
            Some(target * effective_count as f64 - other_sum)
        }
    }

    /// Whether some evaluation *other than* `eval_idx` already has a graded
    /// value at or above `min`. Used to tell whether a `minimum_one_eval`
    /// requirement is already satisfied without this evaluation's help.
    pub(crate) fn other_eval_meets(&self, eval_idx: usize, min: f64) -> bool {
        self.evaluations
            .iter()
            .enumerate()
            .any(|(i, e)| i != eval_idx && matches!(e.grade, Some(g) if g >= min))
    }

    /// Calculate the average grade of all evaluations in this category,
    /// respecting rules (drop_lowest, averaging_method, round_before_weighting,
    /// weighted_evaluations).
    /// Ungraded evaluations are treated as 0, reflecting the student's
    /// real current standing (e.g., 90 + 0 + 0 = 30 average).
    /// Returns None only if there are no evaluations at all.
    pub fn average_grade(&self) -> Option<f64> {
        if self.evaluations.is_empty() {
            return None;
        }

        // Weighted evaluations mode: use individual eval weights
        if self.rules.weighted_evaluations {
            let pairs = self.effective_grades_weighted();
            if pairs.is_empty() {
                return None;
            }

            let avg = match self.rules.averaging_method {
                AveragingMethod::Arithmetic => pairs.iter().map(|(g, w)| g * w).sum::<f64>(),
                AveragingMethod::Geometric => {
                    // Weighted geometric mean: exp(sum(w_i * ln(g_i)))
                    if pairs.iter().any(|(g, _)| *g <= 0.0) {
                        0.0
                    } else {
                        let log_sum: f64 = pairs.iter().map(|(g, w)| w * g.ln()).sum();
                        log_sum.exp()
                    }
                }
            };

            return if self.rules.round_before_weighting {
                Some(avg.round())
            } else {
                Some(avg)
            };
        }

        // Equal-weight mode (original behavior)
        let grades = self.effective_grades();
        if grades.is_empty() {
            return None;
        }

        let avg = match self.rules.averaging_method {
            AveragingMethod::Arithmetic => {
                let sum: f64 = grades.iter().sum();
                sum / grades.len() as f64
            }
            AveragingMethod::Geometric => {
                // Geometric mean: (product)^(1/n)
                // If any grade is 0, the result is 0.
                if grades.iter().any(|g| *g <= 0.0) {
                    0.0
                } else {
                    let log_sum: f64 = grades.iter().map(|g| g.ln()).sum();
                    (log_sum / grades.len() as f64).exp()
                }
            }
        };

        if self.rules.round_before_weighting {
            Some(avg.round())
        } else {
            Some(avg)
        }
    }

    /// Calculate this category's weighted contribution to the final grade.
    /// Returns None only if there are no evaluations at all.
    /// Ungraded evaluations count as 0 in the average.
    pub fn weighted_contribution(&self) -> Option<f64> {
        self.average_grade().map(|avg| avg * self.weight / 100.0)
    }

    // =========================================================================
    // Query Helpers
    // =========================================================================

    /// Returns the number of graded evaluations.
    pub fn graded_count(&self) -> usize {
        self.evaluations
            .iter()
            .filter(|e| e.grade.is_some())
            .count()
    }

    /// Get total weight of all evaluation weights in this category.
    /// Only meaningful when `rules.weighted_evaluations` is true.
    pub fn total_eval_weight(&self) -> f64 {
        self.evaluations
            .iter()
            .map(|e| e.weight.unwrap_or(0.0))
            .sum()
    }

    /// Validate that evaluation weights sum to 100%.
    /// Only meaningful when `rules.weighted_evaluations` is true.
    pub fn validate_eval_weights(&self) -> super::WeightValidation {
        if self.evaluations.is_empty() {
            return super::WeightValidation::Empty;
        }

        let total = self.total_eval_weight();
        let tolerance = 0.01;

        if (total - 100.0).abs() < tolerance {
            super::WeightValidation::Valid
        } else if total < 100.0 {
            super::WeightValidation::Under(total)
        } else {
            super::WeightValidation::Over(total)
        }
    }

    /// Check if category average is passing, using the course's passing grade.
    pub fn is_passing(&self, passing_grade: f64) -> Option<bool> {
        self.average_grade().map(|avg| avg >= passing_grade)
    }

    /// Check if the minimum average requirement is met.
    /// Returns None if there's no minimum requirement or no evaluations.
    /// Returns Some(true) if met, Some(false) if not met.
    pub fn meets_minimum(&self) -> Option<bool> {
        let min = self.rules.minimum_average?;
        self.average_grade().map(|avg| avg >= min)
    }

    /// Check whether at least one graded evaluation meets the `minimum_one_eval`
    /// threshold.  Returns `None` if there is no such requirement.
    /// Returns `Some(true)` if at least one graded eval >= threshold,
    /// `Some(false)` if all graded evals are below.
    ///
    /// Returns `None` until at least one evaluation is graded: the requirement
    /// "at least one eval >= X" cannot be judged failed before any grade exists
    /// (the student simply hasn't had the chance to meet it yet).
    pub fn any_eval_meets_minimum(&self) -> Option<bool> {
        let min = self.rules.minimum_one_eval?;
        if self.graded_count() == 0 {
            return None;
        }
        let any_passes = self
            .evaluations
            .iter()
            .any(|e| matches!(e.grade, Some(g) if g >= min));
        Some(any_passes)
    }

    /// Check if all individual evaluations meet the per-evaluation minimum.
    /// Returns None if there's no per-evaluation requirement.
    /// Returns Some(list of failing eval indices) if there is a requirement.
    pub fn evals_below_minimum(&self) -> Option<Vec<usize>> {
        let min = self.rules.minimum_per_evaluation?;
        let failing: Vec<usize> = self
            .evaluations
            .iter()
            .enumerate()
            .filter(|(_, e)| match e.grade {
                Some(g) => g < min,
                None => false, // Ungraded evals are not flagged (they haven't been taken yet)
            })
            .map(|(i, _)| i)
            .collect();
        Some(failing)
    }

    /// Returns the number of effective evaluations (after dropping lowest).
    #[cfg(test)]
    pub fn effective_eval_count(&self) -> usize {
        self.effective_grades().len()
    }

    /// Returns the indices of evaluations that would be dropped.
    /// Drop-lowest is incompatible with weighted evaluations.
    pub fn dropped_indices(&self) -> Vec<usize> {
        if self.rules.weighted_evaluations
            || self.rules.drop_lowest == 0
            || self.evaluations.is_empty()
            || self.rules.drop_lowest >= self.evaluations.len()
        {
            return Vec::new();
        }

        let mut indexed: Vec<(usize, f64)> = self
            .evaluations
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.grade.unwrap_or(0.0)))
            .collect();

        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed
            .iter()
            .take(self.rules.drop_lowest)
            .map(|(i, _)| *i)
            .collect()
    }
}
