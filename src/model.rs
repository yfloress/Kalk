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
//! Each Category has a weight (percentage of final grade).
//! Evaluations within a Category are averaged equally.
//! Ungraded evaluations are treated as 0 in all calculations,
//! reflecting the real current standing of the student.
//!
//! Grade scale: 0-100, with 55 as passing grade by default.

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
/// Evaluations are averaged equally within their category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: Uuid,
    pub name: String,
    /// Grade obtained (None if not yet graded). Scale: 0-100
    pub grade: Option<f64>,
}

impl Evaluation {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            grade: None,
        }
    }

    #[cfg(test)]
    pub fn with_grade(name: String, grade: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            grade: Some(grade.clamp(MIN_GRADE, MAX_GRADE)),
        }
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
}

impl Category {
    pub fn new(name: String, weight: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            weight: weight.clamp(0.0, 100.0),
            evaluations: Vec::new(),
        }
    }

    pub fn with_evaluations(name: String, weight: f64, evaluations: Vec<Evaluation>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            weight: weight.clamp(0.0, 100.0),
            evaluations,
        }
    }

    /// Calculate the average grade of all evaluations in this category.
    /// Ungraded evaluations are treated as 0, reflecting the student's
    /// real current standing (e.g., 90 + 0 + 0 = 30 average).
    /// Returns None only if there are no evaluations at all.
    pub fn average_grade(&self) -> Option<f64> {
        if self.evaluations.is_empty() {
            None
        } else {
            let total: f64 = self
                .evaluations
                .iter()
                .map(|e| e.grade.unwrap_or(0.0))
                .sum();
            Some(total / self.evaluations.len() as f64)
        }
    }

    /// Calculate this category's weighted contribution to the final grade.
    /// Returns None only if there are no evaluations at all.
    /// Ungraded evaluations count as 0 in the average.
    pub fn weighted_contribution(&self) -> Option<f64> {
        self.average_grade().map(|avg| avg * self.weight / 100.0)
    }

    /// Returns the number of graded evaluations.
    pub fn graded_count(&self) -> usize {
        self.evaluations
            .iter()
            .filter(|e| e.grade.is_some())
            .count()
    }

    /// Check if category average is passing, using the course's passing grade.
    pub fn is_passing(&self, passing_grade: f64) -> Option<bool> {
        self.average_grade().map(|avg| avg >= passing_grade)
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
}

impl Course {
    pub fn new(name: String, passing_grade: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            passing_grade: passing_grade.clamp(MIN_GRADE, MAX_GRADE),
            categories: Vec::new(),
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
                Category::with_evaluations(ct.name.clone(), ct.weight, evaluations)
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            name,
            passing_grade: passing_grade.clamp(MIN_GRADE, MAX_GRADE),
            categories,
        }
    }

    /// Calculate the current weighted grade based on all evaluations.
    /// Ungraded evaluations are treated as 0.
    /// Returns None if there are no categories with evaluations.
    pub fn current_grade(&self) -> Option<f64> {
        let categories_with_evals: Vec<&Category> = self
            .categories
            .iter()
            .filter(|c| !c.evaluations.is_empty())
            .collect();

        if categories_with_evals.is_empty() {
            None
        } else {
            let total: f64 = categories_with_evals
                .iter()
                .filter_map(|c| c.weighted_contribution())
                .sum();
            Some(total)
        }
    }

    /// Get total weight of all categories.
    pub fn total_weight(&self) -> f64 {
        self.categories.iter().map(|c| c.weight).sum()
    }

    /// Get the remaining weight to reach 100%.
    /// Returns 0.0 if already at or above 100%.
    pub fn remaining_weight(&self) -> f64 {
        (100.0 - self.total_weight()).max(0.0)
    }

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
            cat.weight = (equal_weight * 10.0).round() / 10.0; // Round to 1 decimal
        }

        // Adjust last category to ensure exact 100%
        let total: f64 = self.categories.iter().map(|c| c.weight).sum();
        if let Some(last) = self.categories.last_mut() {
            last.weight += 100.0 - total;
        }
    }

    /// Convert this course's structure to a reusable template.
    /// Only preserves the category structure (name, weight, evaluation count),
    /// not the actual grades.
    pub fn to_template(&self, template_name: String, description: String) -> CourseTemplate {
        let categories = self
            .categories
            .iter()
            .map(|cat| CategoryTemplate::new(&cat.name, cat.weight, cat.evaluations.len()))
            .collect();

        CourseTemplate::new(&template_name, &description, categories)
    }

    /// Generate a description string based on course structure.
    /// Format: "3x Certamen (80%) + 3x Control (20%)"
    pub fn generate_template_description(&self) -> String {
        if self.categories.is_empty() {
            return "Empty template".to_string();
        }

        self.categories
            .iter()
            .map(|c| format!("{}x {} ({:.0}%)", c.evaluations.len(), c.name, c.weight))
            .collect::<Vec<_>>()
            .join(" + ")
    }

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

        let eval_count = category.evaluations.len() as f64;
        if eval_count == 0.0 {
            return NeededGrade::info();
        }

        // Calculate total contribution from all categories,
        // treating ungraded evals as 0, excluding current eval
        let mut total_contribution = 0.0;

        for (ci, cat) in self.categories.iter().enumerate() {
            if ci == category_idx {
                // For the target category, sum all evals except the one we're solving for
                let other_sum: f64 = cat
                    .evaluations
                    .iter()
                    .enumerate()
                    .map(|(ei, e)| {
                        if ei == eval_idx {
                            0.0
                        } else {
                            e.grade.unwrap_or(0.0)
                        }
                    })
                    .sum();
                total_contribution +=
                    other_sum * cat.weight / (100.0 * cat.evaluations.len() as f64);
            } else if !cat.evaluations.is_empty() {
                let sum: f64 = cat.evaluations.iter().map(|e| e.grade.unwrap_or(0.0)).sum();
                let avg = sum / cat.evaluations.len() as f64;
                total_contribution += avg * cat.weight / 100.0;
            }
        }

        // Weight of this single evaluation in the final grade
        let eval_weight = category.weight / (100.0 * eval_count);

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
}

impl CategoryTemplate {
    pub fn new(name: &str, weight: f64, count: usize) -> Self {
        Self {
            name: name.to_string(),
            weight,
            default_evaluation_count: count,
        }
    }
}

/// Pre-defined course structure template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseTemplate {
    pub name: String,
    pub description: String,
    pub categories: Vec<CategoryTemplate>,
}

impl CourseTemplate {
    pub fn new(name: &str, description: &str, categories: Vec<CategoryTemplate>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            categories,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_average_with_ungraded_as_zero() {
        let mut cat = Category::new("Tests".to_string(), 60.0);
        cat.evaluations
            .push(Evaluation::with_grade("Test 1".to_string(), 90.0));
        cat.evaluations.push(Evaluation::new("Test 2".to_string()));
        cat.evaluations.push(Evaluation::new("Test 3".to_string()));

        // (90 + 0 + 0) / 3 = 30 — reflects real standing
        assert!((cat.average_grade().unwrap() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_category_average_all_graded() {
        let mut cat = Category::new("Tests".to_string(), 60.0);
        cat.evaluations
            .push(Evaluation::with_grade("Test 1".to_string(), 70.0));
        cat.evaluations
            .push(Evaluation::with_grade("Test 2".to_string(), 80.0));

        assert!((cat.average_grade().unwrap() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_category_average_empty() {
        let cat = Category::new("Tests".to_string(), 60.0);
        assert!(cat.average_grade().is_none());
    }

    #[test]
    fn test_category_weighted_contribution() {
        let mut cat = Category::new("Tests".to_string(), 80.0);
        cat.evaluations
            .push(Evaluation::with_grade("Test 1".to_string(), 50.0));

        // 50 * 80 / 100 = 40
        assert!((cat.weighted_contribution().unwrap() - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_is_passing_uses_custom_grade() {
        let mut cat = Category::new("Tests".to_string(), 60.0);
        cat.evaluations
            .push(Evaluation::with_grade("Test 1".to_string(), 57.0));

        // With default 55: should pass
        assert_eq!(cat.is_passing(55.0), Some(true));
        // With 60: should fail
        assert_eq!(cat.is_passing(60.0), Some(false));
    }

    #[test]
    fn test_course_current_grade() {
        let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

        let mut certs = Category::new("Certamenes".to_string(), 80.0);
        certs
            .evaluations
            .push(Evaluation::with_grade("C1".to_string(), 60.0));

        let mut controls = Category::new("Controles".to_string(), 20.0);
        controls
            .evaluations
            .push(Evaluation::with_grade("Co1".to_string(), 80.0));

        course.categories.push(certs);
        course.categories.push(controls);

        // (60 * 80 / 100) + (80 * 20 / 100) = 48 + 16 = 64
        assert!((course.current_grade().unwrap() - 64.0).abs() < 0.01);
    }

    #[test]
    fn test_remaining_weight() {
        let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
        course.categories.push(Category::new("A".to_string(), 60.0));

        assert!((course.remaining_weight() - 40.0).abs() < 0.01);

        course.categories.push(Category::new("B".to_string(), 40.0));
        assert!((course.remaining_weight() - 0.0).abs() < 0.01);

        // Over 100% should return 0
        course.categories.push(Category::new("C".to_string(), 10.0));
        assert!((course.remaining_weight() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_weight_validation() {
        let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);

        assert_eq!(course.validate_weights(), WeightValidation::Empty);

        course.categories.push(Category::new("A".to_string(), 60.0));
        assert!(matches!(
            course.validate_weights(),
            WeightValidation::Under(_)
        ));

        course.categories.push(Category::new("B".to_string(), 40.0));
        assert_eq!(course.validate_weights(), WeightValidation::Valid);

        course.categories.push(Category::new("C".to_string(), 10.0));
        assert!(matches!(
            course.validate_weights(),
            WeightValidation::Over(_)
        ));
    }

    #[test]
    fn test_auto_balance_weights() {
        let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
        course.categories.push(Category::new("A".to_string(), 60.0));
        course.categories.push(Category::new("B".to_string(), 60.0));

        course.auto_balance_weights();

        assert!((course.total_weight() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_from_template() {
        let template = CourseTemplate::new(
            "Test Template",
            "3x Certamen (80%) + 3x Control (20%)",
            vec![
                CategoryTemplate::new("Certamen", 80.0, 3),
                CategoryTemplate::new("Control", 20.0, 3),
            ],
        );

        let course =
            Course::from_template("Matematicas".to_string(), DEFAULT_PASSING_GRADE, &template);

        assert_eq!(course.categories.len(), 2);

        assert_eq!(course.categories[0].name, "Certamen");
        assert_eq!(course.categories[0].weight, 80.0);
        assert_eq!(course.categories[0].evaluations.len(), 3);
        assert_eq!(course.categories[0].evaluations[0].name, "Certamen 1");

        assert_eq!(course.categories[1].name, "Control");
        assert_eq!(course.categories[1].weight, 20.0);
        assert_eq!(course.categories[1].evaluations.len(), 3);
    }

    #[test]
    fn test_generate_template_description() {
        let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
        course.categories.push(Category::with_evaluations(
            "Certamen".to_string(),
            80.0,
            vec![
                Evaluation::new("C1".to_string()),
                Evaluation::new("C2".to_string()),
                Evaluation::new("C3".to_string()),
            ],
        ));
        course.categories.push(Category::with_evaluations(
            "Control".to_string(),
            20.0,
            vec![
                Evaluation::new("Co1".to_string()),
                Evaluation::new("Co2".to_string()),
            ],
        ));

        let desc = course.generate_template_description();
        assert_eq!(desc, "3x Certamen (80%) + 2x Control (20%)");
    }

    #[test]
    fn test_needed_grade_already_passing() {
        let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
        let mut cat = Category::new("Tests".to_string(), 100.0);
        cat.evaluations
            .push(Evaluation::with_grade("T1".to_string(), 70.0));
        course.categories.push(cat);

        let result = course.needed_grade_for_evaluation(0, 0, false);
        assert_eq!(result.status, NeededGradeStatus::Success);
    }

    #[test]
    fn test_needed_grade_impossible() {
        let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
        let mut cat = Category::new("Tests".to_string(), 100.0);
        // 3 evals, first two are 0
        cat.evaluations
            .push(Evaluation::with_grade("T1".to_string(), 0.0));
        cat.evaluations
            .push(Evaluation::with_grade("T2".to_string(), 0.0));
        cat.evaluations.push(Evaluation::new("T3".to_string()));
        course.categories.push(cat);

        // Need (54.5 - 0) / (100/300) = 54.5 / 0.333 = 163.5 → impossible
        let result = course.needed_grade_for_evaluation(0, 2, true);
        assert_eq!(result.status, NeededGradeStatus::Failure);
        assert!(result.value.unwrap() > MAX_GRADE);
    }

    #[test]
    fn test_needed_grade_achievable() {
        let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
        let mut cat = Category::new("Tests".to_string(), 100.0);
        cat.evaluations
            .push(Evaluation::with_grade("T1".to_string(), 60.0));
        cat.evaluations.push(Evaluation::new("T2".to_string()));
        course.categories.push(cat);

        let result = course.needed_grade_for_evaluation(0, 1, true);
        assert_eq!(result.status, NeededGradeStatus::Warning);
        assert!(result.value.is_some());
    }

    #[test]
    fn test_has_evaluations() {
        let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
        assert!(!course.has_evaluations());

        course
            .categories
            .push(Category::new("A".to_string(), 100.0));
        assert!(!course.has_evaluations());

        course.categories[0]
            .evaluations
            .push(Evaluation::new("E1".to_string()));
        assert!(course.has_evaluations());
    }
}
