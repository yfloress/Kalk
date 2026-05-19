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

//! AI import: schema definition, tolerant JSON parser, and conversion to the
//! internal course model.
//!
//! The user pastes a JSON blob produced by an AI from a syllabus PDF/image.
//! The blob is sanitised (markdown code fences stripped, whitespace trimmed),
//! validated against [`IMPORT_SCHEMA_VERSION`], and converted to a [`Course`]
//! with freshly generated UUIDs.  Name collisions are resolved by appending
//! a localisable "(copia)" / "(copy)" suffix.

use serde::Deserialize;

use crate::i18n::Messages;
use crate::model::{
    AveragingMethod, Category, CategoryRules, Course, Evaluation, GlobalEligibility,
    GlobalExamPolicy, MinimumNotMetAction,
};

/// Version of the import schema this build understands.  Bump when making
/// breaking changes to the JSON shape so older blobs are rejected explicitly
/// instead of silently miscompiling into a corrupt course.
pub const IMPORT_SCHEMA_VERSION: u32 = 1;

// =============================================================================
// Schema
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ImportSchema {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub name: String,
    #[serde(default = "default_passing_grade")]
    pub passing_grade: f64,
    #[serde(default)]
    pub global_exam: Option<ImportGlobal>,
    #[serde(default)]
    pub categories: Vec<ImportCategory>,
}

fn default_schema_version() -> u32 {
    IMPORT_SCHEMA_VERSION
}
fn default_passing_grade() -> f64 {
    55.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportGlobal {
    /// One of `"none"`, `"weighted"`, `"replaces_worst"`.
    pub policy: String,
    #[serde(default)]
    pub semester_weight: Option<f64>,
    #[serde(default)]
    pub global_weight: Option<f64>,
    #[serde(default)]
    pub min_grade: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportCategory {
    pub name: String,
    pub weight: f64,
    #[serde(default)]
    pub drop_lowest: Option<usize>,
    #[serde(default)]
    pub minimum_average: Option<f64>,
    #[serde(default)]
    pub minimum_per_evaluation: Option<f64>,
    #[serde(default)]
    pub weighted_evaluations: Option<bool>,
    /// One of `"arithmetic"`, `"geometric"`.
    #[serde(default)]
    pub averaging_method: Option<String>,
    #[serde(default)]
    pub round_before_weighting: Option<bool>,
    #[serde(default)]
    pub evaluations: Vec<ImportEvaluation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportEvaluation {
    pub name: String,
    #[serde(default)]
    pub grade: Option<f64>,
    #[serde(default)]
    pub weight: Option<f64>,
}

// =============================================================================
// Errors
// =============================================================================

#[derive(Debug)]
pub enum ImportError {
    InvalidJson(serde_json::Error),
    UnsupportedSchema(u32),
    EmptyName,
    NoCategories,
}

impl ImportError {
    /// Render the error as a human-readable, translatable string.
    pub fn user_message(&self, m: &Messages) -> String {
        match self {
            ImportError::InvalidJson(e) => {
                format!("{}: {}", m.import_err_invalid_json, e)
            }
            ImportError::UnsupportedSchema(v) => {
                format!("{} {} (expected {})", m.import_err_schema_version, v, IMPORT_SCHEMA_VERSION)
            }
            ImportError::EmptyName => m.import_err_empty_name.to_string(),
            ImportError::NoCategories => m.import_err_no_categories.to_string(),
        }
    }
}

// =============================================================================
// Parsing
// =============================================================================

/// Strip markdown code fences (```json ... ``` or ``` ... ```) and trim.
///
/// AIs frequently wrap JSON in code fences even when explicitly told not to.
/// This makes the parser tolerant of that.
pub fn sanitize_json_input(raw: &str) -> String {
    let trimmed = raw.trim();

    // Strip opening fence in either form.
    let no_open = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim_start()
    } else if let Some(rest) = trimmed.strip_prefix("```JSON") {
        rest.trim_start()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim_start()
    } else {
        trimmed
    };

    // Strip closing fence.
    let no_close = no_open
        .trim_end()
        .strip_suffix("```")
        .unwrap_or(no_open)
        .trim();

    no_close.to_string()
}

/// Parse raw clipboard text into a validated [`ImportSchema`].
pub fn parse(raw: &str) -> Result<ImportSchema, ImportError> {
    let cleaned = sanitize_json_input(raw);
    let schema: ImportSchema =
        serde_json::from_str(&cleaned).map_err(ImportError::InvalidJson)?;

    if schema.schema_version != IMPORT_SCHEMA_VERSION {
        return Err(ImportError::UnsupportedSchema(schema.schema_version));
    }
    if schema.name.trim().is_empty() {
        return Err(ImportError::EmptyName);
    }
    if schema.categories.is_empty() {
        return Err(ImportError::NoCategories);
    }

    Ok(schema)
}

// =============================================================================
// Conversion to internal model
// =============================================================================

/// Convert a validated [`ImportSchema`] into a fresh [`Course`].
///
/// `existing_names` is consulted to ensure the new course gets a unique name.
/// `copy_suffix` is the localised word ("copia" / "copy") used when a
/// collision is detected.
pub fn to_course(schema: &ImportSchema, existing_names: &[&str], copy_suffix: &str) -> Course {
    let final_name = unique_name(schema.name.trim(), existing_names, copy_suffix);

    let mut course = Course::new(final_name, schema.passing_grade);

    if let Some(g) = &schema.global_exam {
        course.global_policy = match g.policy.as_str() {
            "weighted" => GlobalExamPolicy::Weighted {
                semester_weight: g.semester_weight.unwrap_or(0.7),
                global_weight: g.global_weight.unwrap_or(0.3),
            },
            "replaces_worst" | "replaces_worst_grade" => GlobalExamPolicy::ReplacesWorstGrade,
            _ => GlobalExamPolicy::None,
        };
        course.global_eligibility = GlobalEligibility {
            min_grade: g.min_grade,
        };
    }

    course.categories = schema
        .categories
        .iter()
        .map(|ic| {
            let evaluations: Vec<Evaluation> = ic
                .evaluations
                .iter()
                .map(|ie| {
                    let mut e = Evaluation::new(ie.name.clone());
                    e.grade = ie.grade;
                    e.weight = ie.weight;
                    e
                })
                .collect();

            let averaging_method = match ic.averaging_method.as_deref() {
                Some("geometric") => AveragingMethod::Geometric,
                _ => AveragingMethod::Arithmetic,
            };

            let rules = CategoryRules {
                minimum_average: ic.minimum_average,
                on_minimum_not_met: MinimumNotMetAction::default(),
                drop_lowest: ic.drop_lowest.unwrap_or(0),
                averaging_method,
                minimum_per_evaluation: ic.minimum_per_evaluation,
                on_min_per_eval_not_met: MinimumNotMetAction::default(),
                minimum_one_eval: None,
                on_min_one_eval_not_met: MinimumNotMetAction::default(),
                round_before_weighting: ic.round_before_weighting.unwrap_or(false),
                weighted_evaluations: ic.weighted_evaluations.unwrap_or(false),
            };

            Category::with_rules(ic.name.clone(), ic.weight, evaluations, rules)
        })
        .collect();

    course
}

/// Compute the sum of category weights, used by the preview to warn the user
/// when the import does not add up to 100%.
pub fn total_weight(schema: &ImportSchema) -> f64 {
    schema.categories.iter().map(|c| c.weight).sum()
}

/// Total number of evaluations across all categories.
pub fn total_evaluations(schema: &ImportSchema) -> usize {
    schema.categories.iter().map(|c| c.evaluations.len()).sum()
}

fn unique_name(name: &str, existing: &[&str], copy_suffix: &str) -> String {
    if !existing.iter().any(|n| *n == name) {
        return name.to_string();
    }
    let first = format!("{} ({})", name, copy_suffix);
    if !existing.iter().any(|n| *n == first) {
        return first;
    }
    let mut i = 2;
    loop {
        let candidate = format!("{} ({} {})", name, copy_suffix, i);
        if !existing.iter().any(|n| *n == candidate) {
            return candidate;
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_json_fence() {
        let raw = "```json\n{\"name\":\"x\"}\n```";
        assert_eq!(sanitize_json_input(raw), "{\"name\":\"x\"}");
    }

    #[test]
    fn sanitize_strips_bare_fence() {
        let raw = "```\n{\"name\":\"x\"}\n```";
        assert_eq!(sanitize_json_input(raw), "{\"name\":\"x\"}");
    }

    #[test]
    fn sanitize_handles_no_fence() {
        assert_eq!(sanitize_json_input("  {\"x\":1}  "), "{\"x\":1}");
    }

    #[test]
    fn parse_rejects_wrong_schema_version() {
        let raw = r#"{"schema_version":99,"name":"x","categories":[{"name":"c","weight":100}]}"#;
        assert!(matches!(parse(raw), Err(ImportError::UnsupportedSchema(99))));
    }

    #[test]
    fn parse_rejects_empty_name() {
        let raw = r#"{"name":"  ","categories":[{"name":"c","weight":100}]}"#;
        assert!(matches!(parse(raw), Err(ImportError::EmptyName)));
    }

    #[test]
    fn parse_rejects_no_categories() {
        let raw = r#"{"name":"x","categories":[]}"#;
        assert!(matches!(parse(raw), Err(ImportError::NoCategories)));
    }

    #[test]
    fn unique_name_appends_copy_when_taken() {
        let existing = vec!["Algebra"];
        assert_eq!(unique_name("Algebra", &existing, "copia"), "Algebra (copia)");
    }

    #[test]
    fn unique_name_keeps_original_when_free() {
        let existing: Vec<&str> = vec![];
        assert_eq!(unique_name("Algebra", &existing, "copia"), "Algebra");
    }

    #[test]
    fn unique_name_increments_when_copy_taken_too() {
        let existing = vec!["Algebra", "Algebra (copia)"];
        assert_eq!(
            unique_name("Algebra", &existing, "copia"),
            "Algebra (copia 2)"
        );
    }

    #[test]
    fn to_course_produces_categories_with_rules() {
        let raw = r#"{
            "schema_version": 1,
            "name": "Test",
            "passing_grade": 55,
            "categories": [
                {
                    "name": "Controles",
                    "weight": 40,
                    "drop_lowest": 1,
                    "evaluations": [
                        {"name": "C1"},
                        {"name": "C2"}
                    ]
                }
            ]
        }"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copia");
        assert_eq!(course.name, "Test");
        assert_eq!(course.categories.len(), 1);
        assert_eq!(course.categories[0].rules.drop_lowest, 1);
        assert_eq!(course.categories[0].evaluations.len(), 2);
    }
}
