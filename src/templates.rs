//! Built-in course templates.
//!
//! This file contains pre-defined templates for common course structures.
//! As a developer, you can easily add, modify, or remove templates here.
//!
//! Each template defines:
//! - name: Display name shown in the template selector
//! - description: Brief explanation of the structure
//! - categories: List of category templates with name, weight%, and evaluation count

use crate::model::{CategoryTemplate, CourseTemplate};

/// Returns all built-in templates.
///
/// Templates are displayed in this order in the UI (after "Custom" and user templates).
/// To add a new template, simply add a new entry to this vector.
///
/// # Example
///
/// ```ignore
/// CourseTemplate::new(
///     "Template Name",
///     "Description of the template",
///     vec![
///         CategoryTemplate::new("Category1", 60.0, 3),  // 60%, 3 evaluations
///         CategoryTemplate::new("Category2", 40.0, 2),  // 40%, 2 evaluations
///     ],
/// ),
/// ```
pub fn built_in_templates() -> Vec<CourseTemplate> {
    vec![
        // =======================================================================
        // CUSTOM (Empty) - Always first in the final list
        // =======================================================================
        CourseTemplate::new(
            "Custom (Empty)",
            "Start with no categories - add your own",
            vec![],
        ),
        // =======================================================================
        // Chilean university standard formats
        // =======================================================================
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
        // =======================================================================
        // Add your custom built-in templates below
        // =======================================================================
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_templates_not_empty() {
        let templates = built_in_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_custom_template_is_first() {
        let templates = built_in_templates();
        assert!(templates[0].name.contains("Custom"));
        assert!(templates[0].categories.is_empty());
    }

    #[test]
    fn test_all_templates_have_valid_weights() {
        let templates = built_in_templates();
        for template in templates {
            let total_weight: f64 = template.categories.iter().map(|c| c.weight).sum();
            // Either empty (Custom) or weights sum to 100%
            assert!(
                template.categories.is_empty() || (total_weight - 100.0).abs() < 0.01,
                "Template '{}' has invalid weight sum: {}",
                template.name,
                total_weight
            );
        }
    }
}
