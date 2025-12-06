//! Built-in course templates.
//!
//! This file contains pre-defined templates for common course structures.
//! Templates are generated based on the current language setting.
//!
//! Each template defines:
//! - name: Display name shown in the template selector
//! - description: Brief explanation of the structure
//! - categories: List of category templates with name, weight%, and evaluation count

use crate::i18n::Language;
use crate::model::{CategoryTemplate, CourseTemplate};

/// Returns all built-in templates for the given language.
///
/// Templates are displayed in this order in the UI (after user templates).
/// "Custom (Empty)" is always first.
///
/// # Example
///
/// ```ignore
/// let templates = built_in_templates(Language::English);
/// ```
pub fn built_in_templates(lang: Language) -> Vec<CourseTemplate> {
    let m = lang.messages();

    vec![
        // =======================================================================
        // CUSTOM (Empty) - Always first
        // =======================================================================
        CourseTemplate::new(m.tpl_custom, m.tpl_custom_desc, vec![]),
        // =======================================================================
        // Standard course formats
        // =======================================================================
        CourseTemplate::new(
            m.tpl_exam_quiz_80_20,
            m.tpl_exam_quiz_80_20_desc,
            vec![
                CategoryTemplate::new(m.tpl_exam, 80.0, 3),
                CategoryTemplate::new(m.tpl_quiz, 20.0, 3),
            ],
        ),
        CourseTemplate::new(
            m.tpl_exam_quiz_70_30,
            m.tpl_exam_quiz_70_30_desc,
            vec![
                CategoryTemplate::new(m.tpl_exam, 70.0, 3),
                CategoryTemplate::new(m.tpl_quiz, 30.0, 3),
            ],
        ),
        CourseTemplate::new(
            m.tpl_exam_homework_quiz,
            m.tpl_exam_homework_quiz_desc,
            vec![
                CategoryTemplate::new(m.tpl_exam, 60.0, 3),
                CategoryTemplate::new(m.tpl_homework, 25.0, 4),
                CategoryTemplate::new(m.tpl_quiz, 15.0, 3),
            ],
        ),
        CourseTemplate::new(
            m.tpl_exam_labs,
            m.tpl_exam_labs_desc,
            vec![
                CategoryTemplate::new(m.tpl_exam, 70.0, 3),
                CategoryTemplate::new(m.tpl_lab, 30.0, 6),
            ],
        ),
        CourseTemplate::new(
            m.tpl_project_exams,
            m.tpl_project_exams_desc,
            vec![
                CategoryTemplate::new(m.tpl_project, 40.0, 1),
                CategoryTemplate::new(m.tpl_exam, 60.0, 2),
            ],
        ),
        CourseTemplate::new(
            m.tpl_exams_only,
            m.tpl_exams_only_desc,
            vec![CategoryTemplate::new(m.tpl_exam, 100.0, 3)],
        ),
        CourseTemplate::new(
            m.tpl_continuous,
            m.tpl_continuous_desc,
            vec![CategoryTemplate::new(m.tpl_assessment, 100.0, 10)],
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
        let templates = built_in_templates(Language::English);
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_custom_template_is_first() {
        let templates = built_in_templates(Language::English);
        assert!(templates[0].categories.is_empty());
    }

    #[test]
    fn test_all_templates_have_valid_weights() {
        for lang in Language::all() {
            let templates = built_in_templates(*lang);
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

    #[test]
    fn test_templates_are_translated() {
        let en = built_in_templates(Language::English);
        let es = built_in_templates(Language::Spanish);

        // Same number of templates
        assert_eq!(en.len(), es.len());

        // But different names (first non-empty template)
        assert_ne!(en[1].name, es[1].name);
    }
}
