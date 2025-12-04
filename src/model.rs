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

impl WeightValidation {
    pub fn message(&self) -> String {
        match self {
            WeightValidation::Valid => "Weights OK (100%)".to_string(),
            WeightValidation::Under(total) => format!("Warning: Only {:.1}% assigned", total),
            WeightValidation::Over(total) => format!("Error: {:.1}% exceeds 100%", total),
            WeightValidation::Empty => "No categories".to_string(),
        }
    }
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
            .filter_map(|c| c.weighted_contribution())
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

    /// Get formatted current grade with pass/fail status
    pub fn current_grade_status(&self) -> String {
        match self.current_grade() {
            Some(grade) => {
                let rounded = Self::round_grade(grade);
                if self.is_passing_grade(grade) {
                    format!("Current: {:.1} -> {:.0} (PASSED)", grade, rounded)
                } else {
                    format!("Current: {:.1} -> {:.0} (FAILED)", grade, rounded)
                }
            }
            None => "No grades yet".to_string(),
        }
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

    /// Returns a descriptive string about the grade status.
    /// Tells the user what grade they need in the remaining categories to pass.
    pub fn calculate_required_grade(&self) -> String {
        let remaining_weight = self.remaining_weight();

        // Check if all categories are complete
        let all_complete = self.categories.iter().all(|c| c.is_complete());

        if all_complete || remaining_weight <= 0.0 {
            return match self.current_grade() {
                Some(grade) => {
                    let rounded = Self::round_grade(grade);
                    if self.is_passing_grade(grade) {
                        format!("Passed: {:.1} -> {:.0}", grade, rounded)
                    } else {
                        format!("Failed: {:.1} -> {:.0}", grade, rounded)
                    }
                }
                None => "No evaluations".to_string(),
            };
        }

        match self.current_grade() {
            Some(current) => {
                // Calculate required grade in remaining weight
                // current_contribution + (required * remaining_weight / 100) = passing_grade
                // required = (passing_grade - current_contribution) * 100 / remaining_weight
                let current_contribution = current;
                let required =
                    (self.passing_grade - current_contribution) * 100.0 / remaining_weight;

                if required <= 0.0 {
                    format!("Current: {:.1} | Already passing!", current)
                } else if required > MAX_GRADE {
                    format!(
                        "Current: {:.1} | Cannot pass (need {:.1})",
                        current, required
                    )
                } else {
                    format!(
                        "Current: {:.1} | Need {:.1} in {:.0}%",
                        current, required, remaining_weight
                    )
                }
            }
            None => {
                format!("No grades | Need {:.1} to pass", self.passing_grade)
            }
        }
    }

    /// Calculate what grade is needed in a specific evaluation to pass the course.
    /// Returns a descriptive string.
    pub fn calculate_needed_for_evaluation(&self, category_idx: usize, eval_idx: usize) -> String {
        let Some(category) = self.categories.get(category_idx) else {
            return "Invalid category".to_string();
        };

        let Some(eval) = category.evaluations.get(eval_idx) else {
            return "Invalid evaluation".to_string();
        };

        // If already graded, show that info
        if let Some(grade) = eval.grade {
            if grade >= self.passing_grade {
                return format!("{}: {:.0} (passing)", eval.name, grade);
            } else {
                return format!("{}: {:.0} (below passing)", eval.name, grade);
            }
        }

        // Calculate current contributions from all categories
        let mut total_contribution = 0.0;
        let mut total_weight_graded = 0.0;

        for (ci, cat) in self.categories.iter().enumerate() {
            if ci == category_idx {
                // For target category, calculate partial average excluding target eval
                let other_grades: Vec<f64> = cat
                    .evaluations
                    .iter()
                    .enumerate()
                    .filter(|(ei, e)| *ei != eval_idx && e.grade.is_some())
                    .filter_map(|(_, e)| e.grade)
                    .collect();

                if !other_grades.is_empty() {
                    let other_avg = other_grades.iter().sum::<f64>() / other_grades.len() as f64;
                    // Partial contribution (will be adjusted with new grade)
                    let graded_count = other_grades.len();
                    let total_evals = cat.evaluations.len();
                    // Weight of already graded evals in this category
                    let partial_weight = cat.weight * (graded_count as f64 / total_evals as f64);
                    total_contribution += other_avg * partial_weight / 100.0;
                    total_weight_graded += partial_weight;
                }
            } else if let Some(avg) = cat.average_grade() {
                // Other categories contribute normally
                total_contribution += avg * cat.weight / 100.0;
                total_weight_graded += cat.weight;
            }
        }

        // Calculate weight this single evaluation represents
        let eval_weight = category.weight / category.evaluations.len() as f64;
        let remaining_weight = 100.0 - total_weight_graded;

        if remaining_weight <= 0.0 {
            return "All evaluations graded".to_string();
        }

        // What grade do we need in this evaluation?
        // total_contribution + (needed * eval_weight / 100) + (other_remaining) = passing_grade
        // Simplified: what grade here to reach passing assuming other remaining are at passing level

        // Due to rounding rules (54.5 rounds to 55), we only need 54.5 to pass
        let effective_passing = self.passing_grade - 0.5;

        // Grade needed in this eval to reach passing (assuming remaining evals get passing grade)
        let other_remaining_weight = remaining_weight - eval_weight;
        let other_remaining_contribution = effective_passing * other_remaining_weight / 100.0;

        // needed * eval_weight / 100 = effective_passing - total_contribution - other_remaining_contribution
        let needed_contribution =
            effective_passing - total_contribution - other_remaining_contribution;
        let needed_grade = needed_contribution * 100.0 / eval_weight;

        if needed_grade <= 0.0 {
            "Need: 0+ (already passing with any grade)".to_string()
        } else if needed_grade > MAX_GRADE {
            format!("Need: {:.0}+ (impossible, max is 100)", needed_grade)
        } else {
            format!("Need {:.0}+ in this eval to pass", needed_grade)
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

    /// Returns the list of built-in templates.
    pub fn built_in_templates() -> Vec<CourseTemplate> {
        vec![
            // Chilean university standard formats
            CourseTemplate::new(
                "Certamenes + Controles (80/20)",
                "3 Certamenes (80%) + 3 Controles (20%)",
                vec![
                    CategoryTemplate::new("Certamen", 80.0, 3),
                    CategoryTemplate::new("Control", 20.0, 3),
                ],
            ),
            CourseTemplate::new(
                "Certamenes + Controles (70/30)",
                "3 Certamenes (70%) + 3 Controles (30%)",
                vec![
                    CategoryTemplate::new("Certamen", 70.0, 3),
                    CategoryTemplate::new("Control", 30.0, 3),
                ],
            ),
            CourseTemplate::new(
                "Certamenes + Tareas + Controles",
                "3 Certamenes (60%) + 4 Tareas (25%) + 3 Controles (15%)",
                vec![
                    CategoryTemplate::new("Certamen", 60.0, 3),
                    CategoryTemplate::new("Tarea", 25.0, 4),
                    CategoryTemplate::new("Control", 15.0, 3),
                ],
            ),
            CourseTemplate::new(
                "Certamenes + Labs",
                "3 Certamenes (70%) + 6 Labs (30%)",
                vec![
                    CategoryTemplate::new("Certamen", 70.0, 3),
                    CategoryTemplate::new("Lab", 30.0, 6),
                ],
            ),
            CourseTemplate::new(
                "Proyecto + Certamenes",
                "Proyecto (40%) + 2 Certamenes (60%)",
                vec![
                    CategoryTemplate::new("Proyecto", 40.0, 1),
                    CategoryTemplate::new("Certamen", 60.0, 2),
                ],
            ),
            CourseTemplate::new(
                "Solo Certamenes",
                "3 Certamenes (100%)",
                vec![CategoryTemplate::new("Certamen", 100.0, 3)],
            ),
            CourseTemplate::new(
                "Evaluacion Continua",
                "10 Evaluaciones semanales (100%)",
                vec![CategoryTemplate::new("Evaluacion", 100.0, 10)],
            ),
            CourseTemplate::new(
                "Custom (Empty)",
                "Start with no categories - add your own",
                vec![],
            ),
        ]
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
        let templates = CourseTemplate::built_in_templates();
        let template = &templates[0]; // Certamenes + Controles (80/20)

        let course =
            Course::from_template("Matematicas".to_string(), DEFAULT_PASSING_GRADE, template);

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
