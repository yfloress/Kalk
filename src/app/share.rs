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

//! Sharing a course with classmates: its structure and rules, never the
//! grades, in the same JSON the import wizard reads. Whoever receives it
//! pastes it, or its file path, in step 2 of the import.

use crate::backup;
use crate::model::{
    AttendanceAction, AveragingMethod, Course, GlobalExamPolicy, MinimumNotMetAction,
};

use super::App;
use super::import::{
    IMPORT_SCHEMA_VERSION, ImportAttendance, ImportCategory, ImportCategoryThreshold,
    ImportEvaluation, ImportGlobal, ImportSchema,
};

/// The import schema for `course`, without grades or attendance taken.
pub fn to_schema(course: &Course) -> ImportSchema {
    let category_name = |idx: usize| course.categories.get(idx).map(|c| c.name.clone());

    let attendance = course
        .attendance
        .required_percent
        .map(|required| ImportAttendance {
            required_percent: Some(required),
            if_not_met: Some(
                match course.attendance.action {
                    AttendanceAction::FailCourse => "fails_course",
                    AttendanceAction::WarnOnly => "warn_only",
                }
                .to_string(),
            ),
        });

    let global_exam = match &course.global_policy {
        GlobalExamPolicy::None => None,
        policy => {
            let (name, semester_weight, global_weight) = match policy {
                GlobalExamPolicy::Weighted {
                    semester_weight,
                    global_weight,
                } => ("weighted", Some(*semester_weight), Some(*global_weight)),
                _ => ("replaces_worst", None, None),
            };
            Some(ImportGlobal {
                policy: name.to_string(),
                semester_weight,
                global_weight,
                min_grade: course.global_eligibility.min_grade,
                only_if_category_below: course
                    .global_eligibility
                    .only_if_category_below
                    .as_ref()
                    .and_then(|t| {
                        category_name(t.category_idx).map(|category| ImportCategoryThreshold {
                            category,
                            below_average: t.average,
                        })
                    }),
                cap_if_passed: course.global_outcome.cap_if_passed,
                cap_if_failed: course.global_outcome.cap_if_failed,
            })
        }
    };

    let categories = course
        .categories
        .iter()
        .map(|c| {
            let r = &c.rules;
            ImportCategory {
                name: c.name.clone(),
                weight: c.weight,
                drop_lowest: Some(r.drop_lowest),
                minimum_average: r.minimum_average,
                minimum_per_evaluation: r.minimum_per_evaluation,
                minimum_one_eval: r.minimum_one_eval,
                on_minimum_not_met: Some(action_name(r.on_minimum_not_met)),
                on_min_per_eval_not_met: Some(action_name(r.on_min_per_eval_not_met)),
                on_min_one_eval_not_met: Some(action_name(r.on_min_one_eval_not_met)),
                weighted_evaluations: Some(r.weighted_evaluations),
                averaging_method: Some(
                    match r.averaging_method {
                        AveragingMethod::Arithmetic => "arithmetic",
                        AveragingMethod::Geometric => "geometric",
                    }
                    .to_string(),
                ),
                round_before_weighting: Some(r.round_before_weighting),
                cap_final_grade: r.cap_final_grade,
                requires_categories: r
                    .requires_categories
                    .iter()
                    .filter_map(|&idx| category_name(idx))
                    .collect(),
                evaluations: c
                    .evaluations
                    .iter()
                    .map(|e| ImportEvaluation {
                        name: e.name.clone(),
                        grade: None,
                        weight: e.weight,
                    })
                    .collect(),
            }
        })
        .collect();

    ImportSchema {
        schema_version: IMPORT_SCHEMA_VERSION,
        name: course.name.clone(),
        passing_grade: course.passing_grade,
        credits: course.credits,
        attendance,
        global_exam,
        categories,
    }
}

fn action_name(action: MinimumNotMetAction) -> String {
    match action {
        MinimumNotMetAction::FinalEqualsAverage => "final_equals_average",
        MinimumNotMetAction::RequiresGlobal => "requires_global",
        MinimumNotMetAction::FailCourse => "fail_course",
        MinimumNotMetAction::CapFinalGrade => "cap_final_grade",
    }
    .to_string()
}

/// The contents of the file `text` names, when the paste is a path rather
/// than JSON: dragging a shared course file into the terminal pastes its path.
pub fn read_if_path(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.starts_with('{') || trimmed.starts_with('`') {
        return None;
    }
    let path = backup::expand_path(trimmed);
    if !path.is_file() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

impl App {
    /// Copy the selected course's structure to the clipboard and to a file in
    /// the home folder, whichever is easier to send.
    pub fn share_course(&mut self) {
        let m = self.messages();
        let Some(course) = self.current_course() else {
            return;
        };
        let path = backup::course_share_path(&course.name);
        let Ok(json) = serde_json::to_string_pretty(&to_schema(course)) else {
            self.set_error(m.share_err_write.to_string());
            return;
        };

        let copied = crate::clipboard::copy(&json).is_ok();
        let saved = path.filter(|path| std::fs::write(path, &json).is_ok());

        match (saved, copied) {
            (Some(path), _) => self.set_status(format!(
                "{} {}",
                m.share_saved_to,
                backup::display_path(&path)
            )),
            (None, true) => self.set_status(m.share_copied.to_string()),
            (None, false) => self.set_error(m.share_err_write.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::import::to_course;
    use crate::model::{Category, CategoryRules, Evaluation};

    #[test]
    fn json_is_never_taken_for_a_path() {
        assert_eq!(read_if_path(r#"{"name":"x"}"#), None);
        assert_eq!(read_if_path("```json"), None);
        assert_eq!(read_if_path("/no/such/kalk-course.json"), None);
    }

    #[test]
    fn a_shared_course_reimports_with_its_rules_and_without_grades() {
        let mut course = Course::new("Lab".to_string(), 55.0);
        course.credits = Some(3);
        course.global_policy = GlobalExamPolicy::Weighted {
            semester_weight: 0.7,
            global_weight: 0.3,
        };
        course
            .categories
            .push(Category::new("Reports".to_string(), 60.0));
        let rules = CategoryRules {
            minimum_average: Some(51.0),
            on_minimum_not_met: MinimumNotMetAction::CapFinalGrade,
            cap_final_grade: Some(54.0),
            requires_categories: vec![0],
            ..CategoryRules::default()
        };
        course.categories.push(Category::with_rules(
            "Quizzes".to_string(),
            40.0,
            vec![Evaluation::with_grade("Q1".to_string(), 80.0)],
            rules,
        ));

        let json = serde_json::to_string(&to_schema(&course)).unwrap();
        let schema = super::super::import::parse(&json).unwrap();
        let back = to_course(&schema, &[], "copy");

        assert_eq!(back.name, "Lab");
        assert_eq!(back.credits, Some(3));
        assert_eq!(back.global_policy, course.global_policy);
        let quizzes = &back.categories[1];
        assert_eq!(
            quizzes.rules.on_minimum_not_met,
            MinimumNotMetAction::CapFinalGrade
        );
        assert_eq!(quizzes.rules.cap_final_grade, Some(54.0));
        assert_eq!(quizzes.rules.requires_categories, vec![0]);
        assert_eq!(quizzes.evaluations[0].name, "Q1");
        assert_eq!(quizzes.evaluations[0].grade, None, "grades stay private");
    }
}
