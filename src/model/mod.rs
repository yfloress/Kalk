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

//! Data models for the application.
//!
//! Grade calculation follows a hierarchical structure:
//! Course -> Categories -> Evaluations
//!
//! Each Category has a weight (percentage of final grade) and optional rules
//! (minimum average, drop lowest, averaging method, etc.).
//! Evaluations within a Category are averaged according to the category's rules.
//! Ungraded evaluations are treated as 0 in all calculations,
//! reflecting the real current standing of the student.
//!
//! Grade scale: 0-100, with 55 as passing grade by default.

mod category;
mod global;

#[cfg(test)]
mod tests;

pub use category::{AveragingMethod, Category, CategoryRules, MinimumNotMetAction};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default passing grade (0-100 scale)
pub const DEFAULT_PASSING_GRADE: f64 = 55.0;

/// Minimum valid grade
pub const MIN_GRADE: f64 = 0.0;

/// Maximum valid grade
pub const MAX_GRADE: f64 = 100.0;

// =============================================================================
// Evaluation
// =============================================================================

/// Represents a single evaluation within a category.
/// Evaluations are averaged equally within their category unless
/// the category has `weighted_evaluations` enabled, in which case
/// each evaluation carries its own weight (must sum to 100%).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: Uuid,
    pub name: String,
    /// Grade obtained (None if not yet graded). Scale: 0-100
    pub grade: Option<f64>,
    /// Individual weight within the category (0-100%).
    /// Only used when `CategoryRules::weighted_evaluations` is true.
    /// `None` means equal weight (default behavior).
    #[serde(default)]
    pub weight: Option<f64>,
}

impl Evaluation {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            grade: None,
            weight: None,
        }
    }

    #[cfg(test)]
    pub fn with_grade(name: String, grade: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            grade: Some(grade.clamp(MIN_GRADE, MAX_GRADE)),
            weight: None,
        }
    }

    #[cfg(test)]
    pub fn with_weight(name: String, grade: f64, weight: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            grade: Some(grade.clamp(MIN_GRADE, MAX_GRADE)),
            weight: Some(weight.clamp(0.0, MAX_GRADE)),
        }
    }
}

// =============================================================================
// Weight Validation
// =============================================================================

/// Validation status for course weights.
#[derive(Debug, Clone, PartialEq)]
pub enum WeightValidation {
    /// Weights sum to 100%
    Valid,
    /// Weights sum to less than 100%
    Under(f64),
    /// Weights sum to more than 100%
    Over(f64),
    /// No categories defined
    Empty,
}

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
}

/// Result of calculating the grade needed in a specific evaluation to pass.
#[derive(Debug, Clone)]
pub struct NeededGrade {
    /// The exact grade needed (may be negative, > 100, or NaN for edge cases)
    pub value: Option<f64>,
    pub status: NeededGradeStatus,
}

impl NeededGrade {
    fn success(value: f64) -> Self {
        Self {
            value: Some(value),
            status: NeededGradeStatus::Success,
        }
    }

    fn failure(value: Option<f64>) -> Self {
        Self {
            value,
            status: NeededGradeStatus::Failure,
        }
    }

    fn warning(value: f64) -> Self {
        Self {
            value: Some(value),
            status: NeededGradeStatus::Warning,
        }
    }

    fn info() -> Self {
        Self {
            value: None,
            status: NeededGradeStatus::Info,
        }
    }
}

// =============================================================================
// Global Exam
// =============================================================================

/// How the global exam affects the final grade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GlobalExamPolicy {
    /// No global exam available for this course.
    #[default]
    None,

    /// `nota_final = nota_semestral * semester_weight + nota_global * global_weight`
    ///
    /// `semester_weight + global_weight` should equal 1.0 (validated at input).
    Weighted {
        semester_weight: f64,
        global_weight: f64,
    },

    /// The global exam grade replaces the worst evaluation in the target
    /// category, then the entire course grade is recalculated from scratch.
    ReplacesWorstGrade,
}

/// Eligibility requirements to take the global exam.
///
/// The field defaults to `None` (no restriction — anyone can take it).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalEligibility {
    /// Minimum semester grade required to be eligible. `None` = no minimum.
    #[serde(default)]
    pub min_grade: Option<f64>,
}

// =============================================================================
// Course Grade Result
// =============================================================================

/// The result of computing a course's final grade, accounting for category rules.
#[derive(Debug, Clone)]
pub struct CourseGradeResult {
    /// The computed final grade (semester grade, before global)
    pub grade: f64,
    /// Whether the grade was overridden by a failed minimum requirement
    pub overridden_by: Option<String>,
    /// Whether the student needs a global exam
    pub needs_global: bool,
    /// Categories that failed their minimum average requirement
    pub failed_minimums: Vec<FailedMinimum>,
    /// Categories with individual evaluations below the per-eval minimum
    pub eval_violations: Vec<EvalViolation>,
    /// Grade after applying the global exam. `None` when no global was taken
    /// or the course has no global policy configured.
    pub grade_after_global: Option<f64>,
}

/// A category that failed its minimum average requirement.
#[derive(Debug, Clone)]
pub struct FailedMinimum {
    pub category_idx: usize,
    pub category_name: String,
    pub average: f64,
    pub required: f64,
    pub action: MinimumNotMetAction,
    /// True when this entry was created from a per-evaluation minimum
    /// violation rather than a category-level minimum average failure.
    pub from_per_eval: bool,
}

/// A category with individual evaluation violations.
#[derive(Debug, Clone)]
pub struct EvalViolation {
    pub category_idx: usize,
    pub category_name: String,
    pub failing_indices: Vec<usize>,
    pub required: f64,
}

// =============================================================================
// Course
// =============================================================================

/// Represents a course with its evaluation categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    /// Minimum grade required to pass (default: 55.0)
    pub passing_grade: f64,
    pub categories: Vec<Category>,

    /// Global exam policy for this course.
    #[serde(default)]
    pub global_policy: GlobalExamPolicy,
    /// Eligibility requirements to take the global exam.
    #[serde(default)]
    pub global_eligibility: GlobalEligibility,
    /// Grade obtained in the global exam. `None` = hasn't taken it yet.
    #[serde(default)]
    pub global_exam_grade: Option<f64>,
    /// For `ReplacesWorstGrade`: which category the global targets.
    /// `None` = auto-detect from the category that triggered `RequiresGlobal`.
    #[serde(default)]
    pub global_target_category: Option<usize>,
}

impl Course {
    pub fn new(name: String, passing_grade: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            passing_grade: passing_grade.clamp(MIN_GRADE, MAX_GRADE),
            categories: Vec::new(),
            global_policy: GlobalExamPolicy::None,
            global_eligibility: GlobalEligibility::default(),
            global_exam_grade: None,
            global_target_category: None,
        }
    }

    pub fn from_template(name: String, passing_grade: f64, template: &CourseTemplate) -> Self {
        let categories = template
            .categories
            .iter()
            .map(|ct| {
                let evaluations = (1..=ct.default_evaluation_count)
                    .map(|i| Evaluation::new(format!("{} {}", ct.name, i)))
                    .collect();
                let mut cat = Category::with_evaluations(ct.name.clone(), ct.weight, evaluations);
                cat.rules = ct.rules.clone();
                cat
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            name,
            passing_grade: passing_grade.clamp(MIN_GRADE, MAX_GRADE),
            categories,
            global_policy: template.global_policy.clone(),
            global_eligibility: template.global_eligibility.clone(),
            global_exam_grade: None,
            global_target_category: None,
        }
    }

    // =========================================================================
    // Grade Calculation
    // =========================================================================

    /// Calculate the current weighted grade based on all evaluations,
    /// applying category rules (minimums, drop lowest, etc.).
    /// Returns None if there are no categories with evaluations.
    pub fn current_grade(&self) -> Option<f64> {
        if !self.has_evaluations() {
            return None;
        }

        let result = self.compute_grade();
        Some(result.grade)
    }

    /// Compute the full grade result with all rule checks.
    pub fn compute_grade(&self) -> CourseGradeResult {
        let mut failed_minimums = Vec::new();
        let mut eval_violations = Vec::new();
        let mut needs_global = false;

        // No evaluations at all → nothing to check, grade is 0 with no flags.
        let has_evals = self.has_evaluations();

        // Check all category rules (only when there are evaluations)
        for (idx, cat) in self.categories.iter().enumerate() {
            if !has_evals {
                break;
            }
            // Check minimum average requirement
            if let Some(min_avg) = cat.rules.minimum_average
                && let Some(avg) = cat.average_grade()
                && avg < min_avg
            {
                failed_minimums.push(FailedMinimum {
                    category_idx: idx,
                    category_name: cat.name.clone(),
                    average: avg,
                    required: min_avg,
                    action: cat.rules.on_minimum_not_met,
                    from_per_eval: false,
                });
            }

            // Check per-evaluation minimum (independent action)
            if let Some(failing) = cat.evals_below_minimum()
                && !failing.is_empty()
            {
                eval_violations.push(EvalViolation {
                    category_idx: idx,
                    category_name: cat.name.clone(),
                    failing_indices: failing.clone(),
                    required: cat.rules.minimum_per_evaluation.unwrap_or(0.0),
                });

                let avg = cat.average_grade().unwrap_or(0.0);
                failed_minimums.push(FailedMinimum {
                    category_idx: idx,
                    category_name: cat.name.clone(),
                    average: avg,
                    required: cat.rules.minimum_per_evaluation.unwrap_or(0.0),
                    action: cat.rules.on_min_per_eval_not_met,
                    from_per_eval: true,
                });
            }

            // Check "at least one eval >= X" requirement (independent action)
            if let Some(false) = cat.any_eval_meets_minimum() {
                let avg = cat.average_grade().unwrap_or(0.0);
                failed_minimums.push(FailedMinimum {
                    category_idx: idx,
                    category_name: cat.name.clone(),
                    average: avg,
                    required: cat.rules.minimum_one_eval.unwrap_or(0.0),
                    action: cat.rules.on_min_one_eval_not_met,
                    from_per_eval: false,
                });
            }
        }

        // Calculate the normal weighted grade
        let normal_grade: f64 = self
            .categories
            .iter()
            .filter_map(|c| c.weighted_contribution())
            .sum();

        // Apply minimum-not-met consequences
        // FailCourse takes highest priority — grade is 0, course is auto-failed
        let fail_course_failures: Vec<&FailedMinimum> = failed_minimums
            .iter()
            .filter(|f| f.action == MinimumNotMetAction::FailCourse)
            .collect();

        if let Some(worst) = fail_course_failures.iter().min_by(|a, b| {
            a.average
                .partial_cmp(&b.average)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return CourseGradeResult {
                grade: 0.0,
                overridden_by: Some(worst.category_name.clone()),
                needs_global: false,
                failed_minimums,
                eval_violations,
                grade_after_global: None,
            };
        }

        // If any category with FinalEqualsAverage fails, use the worst one
        let final_equals_avg_failures: Vec<&FailedMinimum> = failed_minimums
            .iter()
            .filter(|f| f.action == MinimumNotMetAction::FinalEqualsAverage)
            .collect();

        if let Some(worst) = final_equals_avg_failures.iter().min_by(|a, b| {
            a.average
                .partial_cmp(&b.average)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return CourseGradeResult {
                grade: worst.average,
                overridden_by: Some(worst.category_name.clone()),
                needs_global: false,
                failed_minimums,
                eval_violations,
                grade_after_global: None,
            };
        }

        // Check for RequiresGlobal
        let requires_global_failures: Vec<&FailedMinimum> = failed_minimums
            .iter()
            .filter(|f| f.action == MinimumNotMetAction::RequiresGlobal)
            .collect();

        if !requires_global_failures.is_empty() {
            if self.global_policy == GlobalExamPolicy::None {
                // No global exam configured — treat RequiresGlobal as
                // FinalEqualsAverage so the course shows as FAILED with
                // the worst failing category's average as its grade.
                if let Some(worst) = requires_global_failures.iter().min_by(|a, b| {
                    a.average
                        .partial_cmp(&b.average)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    return CourseGradeResult {
                        grade: worst.average,
                        overridden_by: Some(worst.category_name.clone()),
                        needs_global: false,
                        failed_minimums,
                        eval_violations,
                        grade_after_global: None,
                    };
                }
            } else {
                needs_global = true;
            }
        }

        // If the semester grade is below passing and a global policy is
        // configured, the student needs the global exam even when no
        // individual category triggered RequiresGlobal.  This covers the
        // common rule "NP < passing_grade => must take global".
        if !needs_global
            && self.global_policy != GlobalExamPolicy::None
            && has_evals
            && !self.is_passing_grade(normal_grade)
        {
            needs_global = true;
        }

        // Compute post-global grade if a global exam grade has been entered
        let grade_after_global = self.compute_grade_after_global(normal_grade);

        CourseGradeResult {
            grade: normal_grade,
            overridden_by: None,
            needs_global,
            failed_minimums,
            eval_violations,
            grade_after_global,
        }
    }

    // =========================================================================
    // Weight Helpers
    // =========================================================================

    /// Get total weight of all categories.
    pub fn total_weight(&self) -> f64 {
        self.categories.iter().map(|c| c.weight).sum()
    }

    /// Get the remaining weight to reach 100%.
    /// Returns 0.0 if already at or above 100%.
    pub fn remaining_weight(&self) -> f64 {
        (100.0 - self.total_weight()).max(0.0)
    }

    /// Validate that category weights sum to 100%.
    pub fn validate_weights(&self) -> WeightValidation {
        if self.categories.is_empty() {
            return WeightValidation::Empty;
        }

        let total = self.total_weight();
        let tolerance = 0.01;

        if (total - 100.0).abs() < tolerance {
            WeightValidation::Valid
        } else if total < 100.0 {
            WeightValidation::Under(total)
        } else {
            WeightValidation::Over(total)
        }
    }

    /// Auto-balance weights: distribute equally among all categories.
    pub fn auto_balance_weights(&mut self) {
        if self.categories.is_empty() {
            return;
        }

        let count = self.categories.len() as f64;
        let equal_weight = 100.0 / count;

        for cat in &mut self.categories {
            cat.weight = (equal_weight * 10.0).round() / 10.0;
        }

        // Adjust last category to ensure exact 100%
        let total: f64 = self.categories.iter().map(|c| c.weight).sum();
        if let Some(last) = self.categories.last_mut() {
            last.weight += 100.0 - total;
        }
    }

    // =========================================================================
    // Query Helpers
    // =========================================================================

    /// Round grade according to university rules (0.5+ rounds up to nearest integer).
    pub fn round_grade(grade: f64) -> f64 {
        grade.round()
    }

    /// Check if a grade is passing (considering rounding: 54.5+ = 55 = pass).
    pub fn is_passing_grade(&self, grade: f64) -> bool {
        Self::round_grade(grade) >= self.passing_grade
    }

    /// Check if the course has any evaluations at all.
    pub fn has_evaluations(&self) -> bool {
        self.categories.iter().any(|c| !c.evaluations.is_empty())
    }

    // =========================================================================
    // Templates
    // =========================================================================

    /// Convert this course's structure to a reusable template.
    /// Preserves category structure (name, weight, evaluation count, rules)
    /// and global exam configuration, not the actual grades.
    pub fn to_template(&self, template_name: String, description: String) -> CourseTemplate {
        let categories = self
            .categories
            .iter()
            .map(|cat| {
                CategoryTemplate::with_rules(
                    &cat.name,
                    cat.weight,
                    cat.evaluations.len(),
                    cat.rules.clone(),
                )
            })
            .collect();

        CourseTemplate {
            name: template_name,
            description,
            categories,
            global_policy: self.global_policy.clone(),
            global_eligibility: self.global_eligibility.clone(),
        }
    }

    /// Generate a description string based on course structure.
    /// Format: "3x Certamen (80%) + 3x Control (20%)"
    /// Returns an empty string if there are no categories (caller handles i18n).
    pub fn generate_template_description(&self) -> String {
        if self.categories.is_empty() {
            return String::new();
        }

        self.categories
            .iter()
            .map(|c| format!("{}x {} ({:.0}%)", c.evaluations.len(), c.name, c.weight))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    // =========================================================================
    // Needed Grade
    // =========================================================================

    /// Calculate the grade needed in a specific evaluation to pass the course.
    ///
    /// When `ignore_current_grade` is true, the current eval's grade is treated
    /// as 0 (used when editing to show what's needed regardless of existing grade).
    pub fn needed_grade_for_evaluation(
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

        // Categories with zero weight cannot affect the course outcome
        if category.weight.abs() < f64::EPSILON {
            return NeededGrade::failure(None);
        }

        // For geometric mean, the needed-grade calculation is different and complex.
        // For now, only support arithmetic mean in needed-grade calculation.
        if category.rules.averaging_method == AveragingMethod::Geometric {
            return NeededGrade::info();
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

            if !needed.is_finite() {
                return NeededGrade::failure(None);
            }

            return if needed <= 0.0 {
                NeededGrade::success(0.0)
            } else if needed > MAX_GRADE {
                NeededGrade::failure(Some(needed))
            } else {
                NeededGrade::warning(needed)
            };
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

        if !needed.is_finite() {
            return NeededGrade::failure(None);
        }

        if needed <= 0.0 {
            NeededGrade::success(0.0)
        } else if needed > MAX_GRADE {
            NeededGrade::failure(Some(needed))
        } else {
            NeededGrade::warning(needed)
        }
    }
}

// =============================================================================
// Templates
// =============================================================================

/// Template for a category when creating a course from template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTemplate {
    pub name: String,
    pub weight: f64,
    pub default_evaluation_count: usize,
    /// Rules to apply to this category (uses default if not specified).
    #[serde(default)]
    pub rules: CategoryRules,
}

impl CategoryTemplate {
    pub fn new(name: &str, weight: f64, count: usize) -> Self {
        Self {
            name: name.to_string(),
            weight,
            default_evaluation_count: count,
            rules: CategoryRules::default(),
        }
    }

    pub fn with_rules(name: &str, weight: f64, count: usize, rules: CategoryRules) -> Self {
        Self {
            name: name.to_string(),
            weight,
            default_evaluation_count: count,
            rules,
        }
    }
}

/// Pre-defined course structure template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseTemplate {
    pub name: String,
    pub description: String,
    pub categories: Vec<CategoryTemplate>,
    /// Global exam policy saved with the template.
    #[serde(default)]
    pub global_policy: GlobalExamPolicy,
    /// Global eligibility requirements saved with the template.
    #[serde(default)]
    pub global_eligibility: GlobalEligibility,
}

impl CourseTemplate {
    pub fn new(name: &str, description: &str, categories: Vec<CategoryTemplate>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            categories,
            global_policy: GlobalExamPolicy::None,
            global_eligibility: GlobalEligibility::default(),
        }
    }
}
