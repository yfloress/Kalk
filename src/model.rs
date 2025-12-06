//! Data models for the application.
//!
//! Grade calculation follows a hierarchical structure:
//! Course -> Categories -> Evaluations
//!
//! Each Category has a weight (percentage of final grade).
//! Evaluations within a Category are averaged equally.
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

    /// Calculate the average grade of all graded evaluations in this category.
    /// Returns None if no evaluations have been graded.
    pub fn average_grade(&self) -> Option<f64> {
        let graded: Vec<f64> = self.evaluations.iter().filter_map(|e| e.grade).collect();

        if graded.is_empty() {
            None
        } else {
            Some(graded.iter().sum::<f64>() / graded.len() as f64)
        }
    }

    /// Calculate this category's contribution to the final grade.
    /// Returns None if no evaluations have been graded.
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

    /// Returns true if all evaluations in this category have been graded.
    pub fn is_complete(&self) -> bool {
        !self.evaluations.is_empty() && self.graded_count() == self.evaluations.len()
    }

    /// Check if category average is passing
    pub fn is_passing(&self) -> Option<bool> {
        self.average_grade().map(|avg| avg >= DEFAULT_PASSING_GRADE)
    }
}

/// Validation status for course weights
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

    /// Calculate the current weighted grade based on graded evaluations.
    /// Only considers categories that have at least one graded evaluation.
    pub fn current_grade(&self) -> Option<f64> {
        let contributions: Vec<f64> = self
            .categories
            .iter()
            .filter_map(Category::weighted_contribution)
            .collect();

        if contributions.is_empty() {
            None
        } else {
            Some(contributions.iter().sum())
        }
    }

    /// Calculate the total weight of categories that have been graded.
    pub fn graded_weight(&self) -> f64 {
        self.categories
            .iter()
            .filter(|c| c.average_grade().is_some())
            .map(|c| c.weight)
            .sum()
    }

    /// Calculate the remaining weight percentage not yet graded.
    pub fn remaining_weight(&self) -> f64 {
        100.0 - self.graded_weight()
    }

    /// Get total weight of all categories.
    pub fn total_weight(&self) -> f64 {
        self.categories.iter().map(|c| c.weight).sum()
    }

    /// Round grade according to university rules (0.5+ rounds up to nearest integer)
    pub fn round_grade(grade: f64) -> f64 {
        grade.round()
    }

    /// Check if a grade is passing (considering rounding: 54.5+ = 55 = pass)
    pub fn is_passing_grade(&self, grade: f64) -> bool {
        Self::round_grade(grade) >= self.passing_grade
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

    /// Auto-balance weights: distribute remaining weight proportionally
    /// or set equal weights for all categories.
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
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_average() {
        let mut cat = Category::new("Tests".to_string(), 60.0);
        cat.evaluations
            .push(Evaluation::with_grade("Test 1".to_string(), 70.0));
        cat.evaluations
            .push(Evaluation::with_grade("Test 2".to_string(), 80.0));
        cat.evaluations.push(Evaluation::new("Test 3".to_string())); // Not graded

        // Average of graded evaluations: (70 + 80) / 2 = 75
        assert!((cat.average_grade().unwrap() - 75.0).abs() < 0.01);
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
    fn test_weight_validation() {
        let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);

        // Empty course
        assert_eq!(course.validate_weights(), WeightValidation::Empty);

        // Under 100%
        course.categories.push(Category::new("A".to_string(), 60.0));
        assert!(matches!(
            course.validate_weights(),
            WeightValidation::Under(_)
        ));

        // Exactly 100%
        course.categories.push(Category::new("B".to_string(), 40.0));
        assert_eq!(course.validate_weights(), WeightValidation::Valid);

        // Over 100%
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
        // Create a test template directly
        let template = CourseTemplate::new(
            "Test Template",
            "3 Certamenes (80%) + 3 Controles (20%)",
            vec![
                CategoryTemplate::new("Certamen", 80.0, 3),
                CategoryTemplate::new("Control", 20.0, 3),
            ],
        );

        let course =
            Course::from_template("Matematicas".to_string(), DEFAULT_PASSING_GRADE, &template);

        // Should have 2 categories
        assert_eq!(course.categories.len(), 2);

        // First category: Certamen with 3 evaluations
        assert_eq!(course.categories[0].name, "Certamen");
        assert_eq!(course.categories[0].weight, 80.0);
        assert_eq!(course.categories[0].evaluations.len(), 3);
        assert_eq!(course.categories[0].evaluations[0].name, "Certamen 1");
        assert_eq!(course.categories[0].evaluations[1].name, "Certamen 2");
        assert_eq!(course.categories[0].evaluations[2].name, "Certamen 3");

        // Second category: Control with 3 evaluations
        assert_eq!(course.categories[1].name, "Control");
        assert_eq!(course.categories[1].weight, 20.0);
        assert_eq!(course.categories[1].evaluations.len(), 3);
        assert_eq!(course.categories[1].evaluations[0].name, "Control 1");
        assert_eq!(course.categories[1].evaluations[1].name, "Control 2");
        assert_eq!(course.categories[1].evaluations[2].name, "Control 3");
    }
}
