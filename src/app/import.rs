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
    Attendance, AttendanceAction, AveragingMethod, Category, CategoryRules, CategoryThreshold,
    Course, Evaluation, GlobalEligibility, GlobalExamPolicy, GlobalOutcome, MinimumNotMetAction,
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
    pub credits: Option<u32>,
    #[serde(default)]
    pub attendance: Option<ImportAttendance>,
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
    /// Only eligible while this category averages below `below_average`.
    #[serde(default)]
    pub only_if_category_below: Option<ImportCategoryThreshold>,
    #[serde(default)]
    pub cap_if_passed: Option<f64>,
    #[serde(default)]
    pub cap_if_failed: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportAttendance {
    #[serde(default)]
    pub required_percent: Option<f64>,
    /// `"warn_only"` or `"fails_course"`.
    #[serde(default)]
    pub if_not_met: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportCategoryThreshold {
    pub category: String,
    pub below_average: f64,
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
    pub minimum_one_eval: Option<f64>,
    /// Each one of `"final_equals_average"`, `"requires_global"`, `"fail_course"`.
    #[serde(default)]
    pub on_minimum_not_met: Option<String>,
    #[serde(default)]
    pub on_min_per_eval_not_met: Option<String>,
    #[serde(default)]
    pub on_min_one_eval_not_met: Option<String>,
    #[serde(default)]
    pub weighted_evaluations: Option<bool>,
    /// One of `"arithmetic"`, `"geometric"`.
    #[serde(default)]
    pub averaging_method: Option<String>,
    #[serde(default)]
    pub round_before_weighting: Option<bool>,
    /// Ceiling used when an action is `cap_final_grade`.
    #[serde(default)]
    pub cap_final_grade: Option<f64>,
    /// Names of categories that must be completed before this one.
    #[serde(default)]
    pub requires_categories: Vec<String>,
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
                format!(
                    "{} {} (expected {})",
                    m.import_err_schema_version, v, IMPORT_SCHEMA_VERSION
                )
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
    let schema: ImportSchema = serde_json::from_str(&cleaned).map_err(ImportError::InvalidJson)?;

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
    course.credits = schema.credits.filter(|c| *c > 0);

    if let Some(a) = &schema.attendance
        && let Some(required) = a.required_percent
    {
        course.attendance = Attendance {
            total_classes: None,
            missed: 0,
            required_percent: Some(required.clamp(0.0, 100.0)),
            action: match a.if_not_met.as_deref() {
                Some("fails_course") => AttendanceAction::FailCourse,
                _ => AttendanceAction::WarnOnly,
            },
        };
    }

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
            only_if_category_below: g.only_if_category_below.as_ref().and_then(|c| {
                schema
                    .categories
                    .iter()
                    .position(|ic| ic.name == c.category)
                    .map(|idx| CategoryThreshold {
                        category_idx: idx,
                        average: c.below_average,
                    })
            }),
        };
        course.global_outcome = GlobalOutcome {
            cap_if_passed: g.cap_if_passed,
            cap_if_failed: g.cap_if_failed,
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
                on_minimum_not_met: parse_action(ic.on_minimum_not_met.as_deref()),
                drop_lowest: ic.drop_lowest.unwrap_or(0),
                averaging_method,
                minimum_per_evaluation: ic.minimum_per_evaluation,
                on_min_per_eval_not_met: parse_action(ic.on_min_per_eval_not_met.as_deref()),
                minimum_one_eval: ic.minimum_one_eval,
                on_min_one_eval_not_met: parse_action(ic.on_min_one_eval_not_met.as_deref()),
                round_before_weighting: ic.round_before_weighting.unwrap_or(false),
                weighted_evaluations: ic.weighted_evaluations.unwrap_or(false),
                cap_final_grade: ic.cap_final_grade,
                requires_categories: ic
                    .requires_categories
                    .iter()
                    .filter_map(|name| schema.categories.iter().position(|c| &c.name == name))
                    .collect(),
            };

            Category::with_rules(ic.name.clone(), ic.weight, evaluations, rules)
        })
        .collect();

    course
}

/// Map a minimum-not-met action name onto the enum. Anything unrecognised
/// falls back to the default rather than failing the whole import.
fn parse_action(raw: Option<&str>) -> MinimumNotMetAction {
    match raw {
        Some("requires_global") => MinimumNotMetAction::RequiresGlobal,
        Some("fail_course") => MinimumNotMetAction::FailCourse,
        Some("cap_final_grade") => MinimumNotMetAction::CapFinalGrade,
        _ => MinimumNotMetAction::FinalEqualsAverage,
    }
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
    if !existing.contains(&name) {
        return name.to_string();
    }
    let first = format!("{} ({})", name, copy_suffix);
    if !existing.contains(&first.as_str()) {
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
        assert!(matches!(
            parse(raw),
            Err(ImportError::UnsupportedSchema(99))
        ));
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
        assert_eq!(
            unique_name("Algebra", &existing, "copia"),
            "Algebra (copia)"
        );
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
    fn to_course_reads_credits() {
        let raw = r#"{"schema_version":1,"name":"Math","credits":10,
            "categories":[{"name":"C","weight":100}]}"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copy");
        assert_eq!(course.credits, Some(10));
    }

    #[test]
    fn to_course_treats_missing_or_zero_credits_as_none() {
        for raw in [
            r#"{"schema_version":1,"name":"Math","categories":[{"name":"C","weight":100}]}"#,
            r#"{"schema_version":1,"name":"Math","credits":0,"categories":[{"name":"C","weight":100}]}"#,
        ] {
            let schema = parse(raw).unwrap();
            assert!(to_course(&schema, &[], "copy").credits.is_none());
        }
    }

    #[test]
    fn to_course_reads_minimum_rules_and_actions() {
        let raw = r#"{"schema_version":1,"name":"Math","categories":[{
            "name":"C","weight":100,
            "minimum_average":50,"on_minimum_not_met":"requires_global",
            "minimum_per_evaluation":30,"on_min_per_eval_not_met":"fail_course",
            "minimum_one_eval":60,"on_min_one_eval_not_met":"final_equals_average"
        }]}"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copy");
        let rules = &course.categories[0].rules;

        assert_eq!(rules.minimum_average, Some(50.0));
        assert_eq!(
            rules.on_minimum_not_met,
            MinimumNotMetAction::RequiresGlobal
        );
        assert_eq!(rules.minimum_per_evaluation, Some(30.0));
        assert_eq!(
            rules.on_min_per_eval_not_met,
            MinimumNotMetAction::FailCourse
        );
        assert_eq!(rules.minimum_one_eval, Some(60.0));
        assert_eq!(
            rules.on_min_one_eval_not_met,
            MinimumNotMetAction::FinalEqualsAverage
        );
    }

    #[test]
    fn unknown_action_falls_back_instead_of_failing_the_import() {
        let raw = r#"{"schema_version":1,"name":"Math","categories":[{
            "name":"C","weight":100,"minimum_average":50,
            "on_minimum_not_met":"explode"}]}"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copy");
        assert_eq!(
            course.categories[0].rules.on_minimum_not_met,
            MinimumNotMetAction::FinalEqualsAverage
        );
    }

    #[test]
    fn to_course_reads_caps_attendance_and_prerequisites() {
        let raw = r#"{"schema_version":1,"name":"LabCom",
            "attendance":{"required_percent":85,"if_not_met":"fails_course"},
            "global_exam":{"policy":"weighted","semester_weight":0.7,"global_weight":0.3,
                "only_if_category_below":{"category":"Informes","below_average":60},
                "cap_if_passed":55,"cap_if_failed":54},
            "categories":[
              {"name":"Informes","weight":60},
              {"name":"Controles","weight":40,"minimum_average":51,
               "on_minimum_not_met":"cap_final_grade","cap_final_grade":54,
               "requires_categories":["Informes"]}
            ]}"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copy");

        assert_eq!(course.attendance.required_percent, Some(85.0));
        assert_eq!(course.attendance.action, AttendanceAction::FailCourse);

        assert_eq!(course.global_outcome.cap_if_passed, Some(55.0));
        assert_eq!(course.global_outcome.cap_if_failed, Some(54.0));
        let threshold = course.global_eligibility.only_if_category_below.unwrap();
        assert_eq!(threshold.category_idx, 0, "resolved by category name");
        assert!((threshold.average - 60.0).abs() < 0.01);

        let controls = &course.categories[1];
        assert_eq!(
            controls.rules.on_minimum_not_met,
            MinimumNotMetAction::CapFinalGrade
        );
        assert_eq!(controls.rules.cap_final_grade, Some(54.0));
        assert_eq!(controls.rules.requires_categories, vec![0]);
    }

    #[test]
    fn unknown_category_names_in_prerequisites_are_dropped() {
        let raw = r#"{"schema_version":1,"name":"X","categories":[
            {"name":"A","weight":100,"requires_categories":["Ghost"]}]}"#;
        let schema = parse(raw).unwrap();
        let course = to_course(&schema, &[], "copy");
        assert!(course.categories[0].rules.requires_categories.is_empty());
    }

    #[test]
    fn prompt_documents_every_importable_field() {
        // The prompt is the only thing that makes the AI emit these, so a new
        // schema field that never reaches the prompt is a silent dead end.
        for prompt in [crate::i18n::EN.import_prompt, crate::i18n::ES.import_prompt] {
            for field in [
                "credits",
                "passing_grade",
                "minimum_average",
                "minimum_per_evaluation",
                "minimum_one_eval",
                "on_minimum_not_met",
                "on_min_per_eval_not_met",
                "on_min_one_eval_not_met",
                "drop_lowest",
                "averaging_method",
                "round_before_weighting",
                "weighted_evaluations",
                "semester_weight",
                "global_weight",
                "min_grade",
                "cap_final_grade",
                "requires_categories",
                "only_if_category_below",
                "cap_if_passed",
                "cap_if_failed",
                "required_percent",
            ] {
                assert!(prompt.contains(field), "prompt is missing {field}");
            }
        }
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
