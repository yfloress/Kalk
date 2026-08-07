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

use super::*;

// =========================================================================
// Category averaging tests
// =========================================================================

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

    assert_eq!(cat.is_passing(55.0), Some(true));
    assert_eq!(cat.is_passing(60.0), Some(false));
}

// =========================================================================
// Drop lowest tests
// =========================================================================

#[test]
fn test_drop_lowest_basic() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 70.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 80.0));

    // Drop 30, average of (70 + 80) / 2 = 75
    assert!((cat.average_grade().unwrap() - 75.0).abs() < 0.01);
}

#[test]
fn test_drop_lowest_more_than_evals() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 5; // More than we have
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 70.0));

    // Should not drop anything (would leave 0 evals)
    assert!((cat.average_grade().unwrap() - 50.0).abs() < 0.01);
}

#[test]
fn test_drop_lowest_single_eval() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 50.0));

    // Can't drop the only eval
    assert!((cat.average_grade().unwrap() - 50.0).abs() < 0.01);
}

#[test]
fn test_dropped_indices() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0)); // lowest
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 70.0));

    let dropped = cat.dropped_indices();
    assert_eq!(dropped, vec![1]); // T2 at index 1 is the lowest
}

// =========================================================================
// Geometric mean tests
// =========================================================================

#[test]
fn test_geometric_mean() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.averaging_method = AveragingMethod::Geometric;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 64.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 36.0));

    // Geometric mean of 64 and 36 = sqrt(64*36) = sqrt(2304) = 48
    assert!((cat.average_grade().unwrap() - 48.0).abs() < 0.1);
}

#[test]
fn test_geometric_mean_with_zero() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.averaging_method = AveragingMethod::Geometric;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 0.0));

    // Any zero makes geometric mean = 0
    assert!((cat.average_grade().unwrap() - 0.0).abs() < 0.01);
}

// =========================================================================
// Round before weighting tests
// =========================================================================

#[test]
fn test_round_before_weighting() {
    let mut cat = Category::new("Tests".to_string(), 70.0);
    cat.rules.round_before_weighting = true;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 66.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 67.0));

    // Arithmetic avg = 66.5, rounded = 67
    // Weighted contribution = 67 * 70 / 100 = 46.9
    assert!((cat.average_grade().unwrap() - 67.0).abs() < 0.01);
    assert!((cat.weighted_contribution().unwrap() - 46.9).abs() < 0.01);
}

// =========================================================================
// Minimum average tests
// =========================================================================

#[test]
fn test_meets_minimum_passing() {
    let mut cat = Category::new("Tests".to_string(), 70.0);
    cat.rules.minimum_average = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 60.0));

    assert_eq!(cat.meets_minimum(), Some(true));
}

#[test]
fn test_meets_minimum_failing() {
    let mut cat = Category::new("Tests".to_string(), 70.0);
    cat.rules.minimum_average = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 40.0));

    assert_eq!(cat.meets_minimum(), Some(false));
}

#[test]
fn test_meets_minimum_no_requirement() {
    let mut cat = Category::new("Tests".to_string(), 70.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 40.0));

    assert_eq!(cat.meets_minimum(), None);
}

// =========================================================================
// Per-evaluation minimum tests
// =========================================================================

#[test]
fn test_evals_below_minimum() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(30.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 50.0)); // OK
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 20.0)); // Fails
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 35.0)); // OK

    let failing = cat.evals_below_minimum().unwrap();
    assert_eq!(failing, vec![1]);
}

#[test]
fn test_evals_below_minimum_ungraded_not_flagged() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(30.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 50.0));
    cat.evaluations.push(Evaluation::new("T2".to_string())); // Ungraded

    let failing = cat.evals_below_minimum().unwrap();
    assert!(failing.is_empty());
}

// =========================================================================
// Course grade with rules tests
// =========================================================================

#[test]
fn test_course_grade_normal() {
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
fn test_course_grade_with_minimum_failure() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut certs = Category::new("Controles".to_string(), 70.0);
    certs.rules.minimum_average = Some(50.0);
    certs.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 40.0));
    certs
        .evaluations
        .push(Evaluation::with_grade("C2".to_string(), 30.0));

    let mut tareas = Category::new("Tareas".to_string(), 30.0);
    tareas
        .evaluations
        .push(Evaluation::with_grade("T1".to_string(), 90.0));

    course.categories.push(certs);
    course.categories.push(tareas);

    let result = course.compute_grade();
    // Controles average = 35, which is < 50
    // So final grade = 35 (overridden by Controles)
    assert!((result.grade - 35.0).abs() < 0.01);
    assert!(result.overridden_by.is_some());
    assert_eq!(result.overridden_by.unwrap(), "Controles");
}

#[test]
fn test_course_grade_with_requires_global() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut certs = Category::new("Certamenes".to_string(), 85.0);
    certs.rules.minimum_average = Some(55.0);
    certs.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 40.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 15.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 70.0));

    course.categories.push(certs);
    course.categories.push(quizzes);

    let result = course.compute_grade();
    // Certamenes avg = 40 < 55 → RequiresGlobal, but no global policy
    // configured, so it falls back to FinalEqualsAverage behavior.
    assert!(!result.needs_global);
    assert_eq!(result.overridden_by, Some("Certamenes".to_string()));
    assert!((result.grade - 40.0).abs() < 0.01);
}

#[test]
fn test_course_grade_with_requires_global_and_global_policy() {
    // When a global policy IS configured, RequiresGlobal should set
    // needs_global = true (not fall back to FinalEqualsAverage).
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let mut certs = Category::new("Certamenes".to_string(), 85.0);
    certs.rules.minimum_average = Some(55.0);
    certs.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 40.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 15.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 70.0));

    course.categories.push(certs);
    course.categories.push(quizzes);

    let result = course.compute_grade();
    // Certamenes avg = 40 < 55 → RequiresGlobal, and course HAS a global
    // policy, so needs_global should be true.
    assert!(result.needs_global);
    assert!(result.overridden_by.is_none());
    // Normal weighted grade: (40*85/100) + (70*15/100) = 34 + 10.5 = 44.5
    assert!((result.grade - 44.5).abs() < 0.01);
}

// =========================================================================
// Weight validation tests
// =========================================================================

#[test]
fn test_remaining_weight() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    course.categories.push(Category::new("A".to_string(), 60.0));

    assert!((course.remaining_weight() - 40.0).abs() < 0.01);

    course.categories.push(Category::new("B".to_string(), 40.0));
    assert!((course.remaining_weight() - 0.0).abs() < 0.01);

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

// =========================================================================
// Template tests
// =========================================================================

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

    let course = Course::from_template("Matematicas".to_string(), DEFAULT_PASSING_GRADE, &template);

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
fn test_from_template_with_rules() {
    let rules = CategoryRules {
        minimum_average: Some(50.0),
        on_minimum_not_met: MinimumNotMetAction::FinalEqualsAverage,
        drop_lowest: 1,
        ..Default::default()
    };

    let template = CourseTemplate::new(
        "With Rules",
        "test",
        vec![CategoryTemplate::with_rules("Controles", 70.0, 5, rules)],
    );

    let course = Course::from_template("Math".to_string(), DEFAULT_PASSING_GRADE, &template);

    assert_eq!(course.categories[0].rules.minimum_average, Some(50.0));
    assert_eq!(course.categories[0].rules.drop_lowest, 1);
    assert_eq!(
        course.categories[0].rules.on_minimum_not_met,
        MinimumNotMetAction::FinalEqualsAverage
    );
}

#[test]
fn test_generate_template_description_empty() {
    let course = Course::new("Empty".to_string(), DEFAULT_PASSING_GRADE);
    assert_eq!(course.generate_template_description(), "");
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

// =========================================================================
// Needed grade tests
// =========================================================================

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
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 0.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 0.0));
    cat.evaluations.push(Evaluation::new("T3".to_string()));
    course.categories.push(cat);

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
fn test_needed_grade_with_drop_lowest() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    // 3 evals: 80, 20 (will be dropped), and one we're solving for
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 20.0));
    cat.evaluations.push(Evaluation::new("T3".to_string()));
    course.categories.push(cat);

    let result = course.needed_grade_for_evaluation(0, 2, true);
    // The solved eval survives drop_lowest (a passing grade isn't the discarded
    // one), so the 20 is dropped: avg of (80 + X) / 2 >= 54.5  =>  X >= 29.
    // (A grade below ~29 would itself be dropped and never lift the course.)
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let value = result.value.unwrap();
    assert!(
        (28.0..=30.0).contains(&value),
        "expected needed grade around 29, got {value}"
    );
}

#[test]
fn test_needed_grade_drop_lowest_already_passing() {
    // Two strong grades already pass once the target is dropped, so the target
    // is not needed at all — needed grade is 0 (success), not a positive value.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 90.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 90.0));
    cat.evaluations.push(Evaluation::new("T3".to_string()));
    course.categories.push(cat);

    // With T3 = 0 it gets dropped, leaving 90 & 90 -> course already passes.
    let result = course.needed_grade_for_evaluation(0, 2, true);
    assert_eq!(result.status, NeededGradeStatus::Success);
}

#[test]
fn test_minimum_one_eval_not_judged_before_any_grade() {
    // "At least one eval >= 80" must NOT fail the course while every evaluation
    // is still ungraded — the student hasn't had the chance to meet it yet.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_one_eval = Some(80.0);
    cat.rules.on_min_one_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations.push(Evaluation::new("T1".to_string()));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    assert_eq!(course.categories[0].any_eval_meets_minimum(), None);
    let result = course.compute_grade();
    assert!(
        result.overridden_by.is_none(),
        "one-eval rule should not override the grade before any eval is graded"
    );

    // Once a grade exists and none meets the threshold, the rule does apply.
    course.categories[0].evaluations[0].grade = Some(40.0);
    assert_eq!(course.categories[0].any_eval_meets_minimum(), Some(false));
}

#[test]
fn test_needed_grade_respects_binding_category_minimum() {
    // Scenario from the field: a "Tipo 1" category whose average must reach 50
    // or the whole course is failed (FailCourse), while a high-scoring "Tipo 2"
    // already covers the weighted total. The needed grade must reflect the
    // category minimum, not just the (already satisfied) weighted total.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut tipo1 = Category::new("Tipo 1".to_string(), 50.0);
    tipo1.rules.minimum_average = Some(50.0);
    tipo1.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    tipo1
        .evaluations
        .push(Evaluation::with_grade("E1".to_string(), 40.0));
    tipo1
        .evaluations
        .push(Evaluation::with_grade("E2".to_string(), 45.0));
    tipo1.evaluations.push(Evaluation::new("E3".to_string()));
    course.categories.push(tipo1);

    let mut tipo2 = Category::new("Tipo 2".to_string(), 50.0);
    tipo2
        .evaluations
        .push(Evaluation::with_grade("G1".to_string(), 100.0));
    tipo2
        .evaluations
        .push(Evaluation::with_grade("G2".to_string(), 100.0));
    course.categories.push(tipo2);

    // The weighted total alone is already satisfied, but the category minimum
    // requires (40 + 45 + X) / 3 >= 50  =>  X >= 65.
    let result = course.needed_grade_for_evaluation(0, 2, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let value = result.value.unwrap();
    assert!(
        (64.0..=66.0).contains(&value),
        "expected needed grade around 65, got {value}"
    );
}

#[test]
fn test_needed_grade_binding_minimum_weighted_evaluations() {
    // Same idea but with per-evaluation weights inside the category.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut tipo1 = Category::new("Tipo 1".to_string(), 50.0);
    tipo1.rules.weighted_evaluations = true;
    tipo1.rules.minimum_average = Some(50.0);
    tipo1.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    tipo1
        .evaluations
        .push(Evaluation::with_weight("E1".to_string(), 30.0, 50.0));
    let mut e2 = Evaluation::new("E2".to_string());
    e2.weight = Some(50.0);
    tipo1.evaluations.push(e2);
    course.categories.push(tipo1);

    let mut tipo2 = Category::new("Tipo 2".to_string(), 50.0);
    tipo2
        .evaluations
        .push(Evaluation::with_grade("G1".to_string(), 100.0));
    course.categories.push(tipo2);

    // Category min: 30*0.5 + X*0.5 >= 50  =>  X >= 70.
    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let value = result.value.unwrap();
    assert!(
        (69.0..=71.0).contains(&value),
        "expected needed grade around 70, got {value}"
    );
}

#[test]
fn test_needed_grade_minimum_above_passing_not_binding() {
    // FinalEqualsAverage with a minimum ABOVE the passing grade is not a pass
    // blocker (the capped average could still pass), so it must not inflate the
    // needed grade. Locks the `min <= passing_grade` gate.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_average = Some(70.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 60.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    // Weighted need is (54.5 - 30) / 0.5 = 49; the 70 minimum must NOT raise it.
    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    assert!(
        result.value.unwrap() < 55.0,
        "minimum above passing should not inflate the needed grade"
    );
}

#[test]
fn test_needed_grade_respects_per_eval_minimum_failcourse() {
    // Each evaluation must be >= 50 or the course is failed. The weighted total
    // needs only 49, but the per-evaluation floor lifts it to 50.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(50.0);
    cat.rules.on_min_per_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 60.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let value = result.value.unwrap();
    assert!(
        (49.5..=51.0).contains(&value),
        "expected per-eval floor of 50, got {value}"
    );
}

#[test]
fn test_needed_grade_one_eval_minimum_binding_when_none_meets() {
    // At least one evaluation must reach 80 (FailCourse). No other eval does,
    // so this one must — even though the weighted total needs far less.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_one_eval = Some(80.0);
    cat.rules.on_min_one_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 40.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let value = result.value.unwrap();
    assert!(
        (79.0..=81.0).contains(&value),
        "expected one-eval floor of 80, got {value}"
    );
}

#[test]
fn test_needed_grade_one_eval_minimum_not_binding_when_other_meets() {
    // Same rule, but another evaluation already reaches 80, so the requirement
    // is satisfied and must NOT inflate this evaluation's needed grade.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_one_eval = Some(80.0);
    cat.rules.on_min_one_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 90.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    assert!(
        result.value.unwrap() < 50.0,
        "an already-satisfied one-eval rule must not inflate the needed grade"
    );
}

// =========================================================================
// CategoryRules default tests
// =========================================================================

#[test]
fn test_rules_is_default() {
    let rules = CategoryRules::default();
    assert!(rules.is_default());

    let rules_with_min = CategoryRules {
        minimum_average: Some(50.0),
        ..Default::default()
    };
    assert!(!rules_with_min.is_default());
}

// =========================================================================
// Misc tests
// =========================================================================

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

#[test]
fn test_effective_eval_count() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 70.0));

    assert_eq!(cat.effective_eval_count(), 3);

    cat.rules.drop_lowest = 1;
    assert_eq!(cat.effective_eval_count(), 2);
}

// =========================================================================
// Round grade tests
// =========================================================================

#[test]
fn test_round_grade_half_rounds_up() {
    // 54.5 should round to 55 (0.5+ rounds up)
    assert!((Course::round_grade(54.5) - 55.0).abs() < 0.01);
}

#[test]
fn test_round_grade_below_half_rounds_down() {
    // 54.4 should round to 54
    assert!((Course::round_grade(54.4) - 54.0).abs() < 0.01);
}

#[test]
fn test_round_grade_exact_integer() {
    assert!((Course::round_grade(70.0) - 70.0).abs() < 0.01);
}

#[test]
fn test_round_grade_zero() {
    assert!((Course::round_grade(0.0) - 0.0).abs() < 0.01);
}

#[test]
fn test_round_grade_max() {
    assert!((Course::round_grade(100.0) - 100.0).abs() < 0.01);
}

#[test]
fn test_round_grade_just_above_half() {
    // 54.51 should round to 55
    assert!((Course::round_grade(54.51) - 55.0).abs() < 0.01);
}

#[test]
fn test_round_grade_just_below_half() {
    // 54.49 should round to 54
    assert!((Course::round_grade(54.49) - 54.0).abs() < 0.01);
}

// =========================================================================
// is_passing_grade boundary tests
// =========================================================================

#[test]
fn test_is_passing_grade_exact_boundary() {
    let course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    // 55.0 rounds to 55, which is >= 55
    assert!(course.is_passing_grade(55.0));
}

#[test]
fn test_is_passing_grade_rounds_up_to_pass() {
    let course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    // 54.5 rounds to 55, which is >= 55 → passing
    assert!(course.is_passing_grade(54.5));
}

#[test]
fn test_is_passing_grade_rounds_down_to_fail() {
    let course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    // 54.4 rounds to 54, which is < 55 → failing
    assert!(!course.is_passing_grade(54.4));
}

#[test]
fn test_is_passing_grade_custom_threshold() {
    let course = Course::new("Math".to_string(), 60.0);
    assert!(course.is_passing_grade(60.0));
    assert!(course.is_passing_grade(59.5));
    assert!(!course.is_passing_grade(59.4));
}

// =========================================================================
// graded_count tests
// =========================================================================

#[test]
fn test_graded_count_mixed() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 60.0));

    assert_eq!(cat.graded_count(), 2);
}

#[test]
fn test_graded_count_all_graded() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 60.0));

    assert_eq!(cat.graded_count(), 2);
}

#[test]
fn test_graded_count_none_graded() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations.push(Evaluation::new("T1".to_string()));
    cat.evaluations.push(Evaluation::new("T2".to_string()));

    assert_eq!(cat.graded_count(), 0);
}

#[test]
fn test_graded_count_empty() {
    let cat = Category::new("Tests".to_string(), 100.0);
    assert_eq!(cat.graded_count(), 0);
}

// =========================================================================
// needed_grade with multiple categories
// =========================================================================

#[test]
fn test_needed_grade_multiple_categories() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: Certamenes 70%, one graded eval
    let mut certs = Category::new("Certamenes".to_string(), 70.0);
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 60.0));
    certs.evaluations.push(Evaluation::new("C2".to_string()));

    // Category 1: Controles 30%, one graded eval
    let mut controls = Category::new("Controles".to_string(), 30.0);
    controls
        .evaluations
        .push(Evaluation::with_grade("Co1".to_string(), 80.0));

    course.categories.push(certs);
    course.categories.push(controls);

    // Solving for C2 (category 0, eval 1):
    // Controles contribution = 80 * 30 / 100 = 24
    // Certamenes: effective_grades_excluding(1) → others [60] + target slot
    //   other_sum = 60, effective_count = 2
    //   contribution from others = 60 * 70 / (100 * 2) = 21
    // eval_weight = 70 / (100 * 2) = 0.35
    // needed = (54.5 - 21 - 24) / 0.35 = 9.5 / 0.35 = 27.14...
    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let val = result.value.unwrap();
    assert!(val > 25.0 && val < 30.0, "Expected ~27.1, got {}", val);
}

// =========================================================================
// needed_grade edge cases
// =========================================================================

#[test]
fn test_needed_grade_zero_weight_category() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Bonus".to_string(), 0.0);
    cat.evaluations.push(Evaluation::new("B1".to_string()));
    course.categories.push(cat);

    // Zero weight → cannot affect outcome
    let result = course.needed_grade_for_evaluation(0, 0, true);
    assert_eq!(result.status, NeededGradeStatus::Failure);
}

#[test]
fn test_needed_grade_geometric_returns_info() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.averaging_method = AveragingMethod::Geometric;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 60.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    // Geometric mean → calculation not supported, should return Info
    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(result.status, NeededGradeStatus::Info);
}

#[test]
fn test_needed_grade_already_failing() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 30.0));
    course.categories.push(cat);

    // Already graded and failing, not ignoring current grade
    let result = course.needed_grade_for_evaluation(0, 0, false);
    assert_eq!(result.status, NeededGradeStatus::Failure);
    assert!((result.value.unwrap() - 30.0).abs() < 0.01);
}

#[test]
fn test_needed_grade_invalid_indices() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let cat = Category::new("Tests".to_string(), 100.0);
    course.categories.push(cat);

    // Invalid category index
    let result = course.needed_grade_for_evaluation(5, 0, true);
    assert_eq!(result.status, NeededGradeStatus::Failure);

    // Invalid eval index
    let result = course.needed_grade_for_evaluation(0, 5, true);
    assert_eq!(result.status, NeededGradeStatus::Failure);
}

#[test]
fn test_needed_grade_single_eval_ungraded() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations.push(Evaluation::new("T1".to_string()));
    course.categories.push(cat);

    // Single ungraded eval: need >= 54.5 to pass
    let result = course.needed_grade_for_evaluation(0, 0, true);
    assert_eq!(result.status, NeededGradeStatus::Warning);
    let val = result.value.unwrap();
    assert!((val - 54.5).abs() < 0.01, "Expected 54.5, got {}", val);
}

#[test]
fn test_needed_grade_already_passing_any_grade() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 100.0));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    // With T1=100, avg = (100 + X) / 2 >= 54.5 → X >= 9
    // That's achievable, should be Warning with a low value
    let result = course.needed_grade_for_evaluation(0, 1, true);
    assert!(
        result.status == NeededGradeStatus::Warning || result.status == NeededGradeStatus::Success
    );
}

// =========================================================================
// compute_grade with eval violations
// =========================================================================

#[test]
fn test_compute_grade_eval_violations() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(40.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 20.0)); // Below 40
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 35.0)); // Below 40
    course.categories.push(cat);

    let result = course.compute_grade();
    assert_eq!(result.eval_violations.len(), 1);
    assert_eq!(result.eval_violations[0].category_idx, 0);
    assert_eq!(result.eval_violations[0].failing_indices, vec![1, 2]);
    assert!((result.eval_violations[0].required - 40.0).abs() < 0.01);
}

#[test]
fn test_compute_grade_no_eval_violations_when_all_pass() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(40.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 50.0));
    course.categories.push(cat);

    let result = course.compute_grade();
    assert!(result.eval_violations.is_empty());
}

// =========================================================================
// compute_grade with multiple minimum failures
// =========================================================================

#[test]
fn test_compute_grade_multiple_final_equals_avg_uses_worst() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: avg=30, min=50, action=FinalEqualsAverage
    let mut cat_a = Category::new("CatA".to_string(), 50.0);
    cat_a.rules.minimum_average = Some(50.0);
    cat_a.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat_a
        .evaluations
        .push(Evaluation::with_grade("A1".to_string(), 30.0));

    // Category 1: avg=40, min=50, action=FinalEqualsAverage
    let mut cat_b = Category::new("CatB".to_string(), 50.0);
    cat_b.rules.minimum_average = Some(50.0);
    cat_b.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat_b
        .evaluations
        .push(Evaluation::with_grade("B1".to_string(), 40.0));

    course.categories.push(cat_a);
    course.categories.push(cat_b);

    let result = course.compute_grade();
    // Both fail minimum. Worst average is 30 (CatA).
    assert!((result.grade - 30.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("CatA".to_string()));
    assert_eq!(result.failed_minimums.len(), 2);
}

#[test]
fn test_compute_grade_mixed_minimum_actions() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: avg=40, min=50, action=RequiresGlobal
    let mut cat_a = Category::new("Certs".to_string(), 70.0);
    cat_a.rules.minimum_average = Some(50.0);
    cat_a.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    cat_a
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 40.0));

    // Category 1: normal, no minimum
    let mut cat_b = Category::new("Tasks".to_string(), 30.0);
    cat_b
        .evaluations
        .push(Evaluation::with_grade("T1".to_string(), 90.0));

    course.categories.push(cat_a);
    course.categories.push(cat_b);

    let result = course.compute_grade();
    // Certs fails minimum → RequiresGlobal, but no global policy
    // configured, so falls back to FinalEqualsAverage (grade = cat avg).
    assert!(!result.needs_global);
    assert_eq!(result.overridden_by, Some("Certs".to_string()));
    assert!((result.grade - 40.0).abs() < 0.01);
}

#[test]
fn test_compute_grade_final_equals_avg_takes_priority_over_requires_global() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: avg=25, min=50, FinalEqualsAverage
    let mut cat_a = Category::new("CatA".to_string(), 50.0);
    cat_a.rules.minimum_average = Some(50.0);
    cat_a.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat_a
        .evaluations
        .push(Evaluation::with_grade("A1".to_string(), 25.0));

    // Category 1: avg=40, min=50, RequiresGlobal
    let mut cat_b = Category::new("CatB".to_string(), 50.0);
    cat_b.rules.minimum_average = Some(50.0);
    cat_b.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    cat_b
        .evaluations
        .push(Evaluation::with_grade("B1".to_string(), 40.0));

    course.categories.push(cat_a);
    course.categories.push(cat_b);

    let result = course.compute_grade();
    // FinalEqualsAverage overrides: grade = 25, no needs_global checked
    assert!((result.grade - 25.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("CatA".to_string()));
    assert!(!result.needs_global);
}

#[test]
fn test_compute_grade_fail_course_basic() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with avg=30, min=50, action=FailCourse
    let mut cat = Category::new("Certs".to_string(), 100.0);
    cat.rules.minimum_average = Some(50.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));

    course.categories.push(cat);

    let result = course.compute_grade();
    // FailCourse → grade is 0, overridden by the failing category
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("Certs".to_string()));
    assert!(!result.needs_global);
    assert_eq!(result.failed_minimums.len(), 1);
    assert_eq!(
        result.failed_minimums[0].action,
        MinimumNotMetAction::FailCourse
    );
}

#[test]
fn test_compute_grade_fail_course_takes_priority_over_final_equals_avg() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: avg=40, min=50, FinalEqualsAverage
    let mut cat_a = Category::new("CatA".to_string(), 50.0);
    cat_a.rules.minimum_average = Some(50.0);
    cat_a.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat_a
        .evaluations
        .push(Evaluation::with_grade("A1".to_string(), 40.0));

    // Category 1: avg=30, min=50, FailCourse
    let mut cat_b = Category::new("CatB".to_string(), 50.0);
    cat_b.rules.minimum_average = Some(50.0);
    cat_b.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    cat_b
        .evaluations
        .push(Evaluation::with_grade("B1".to_string(), 30.0));

    course.categories.push(cat_a);
    course.categories.push(cat_b);

    let result = course.compute_grade();
    // FailCourse has highest priority → grade is 0
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("CatB".to_string()));
    assert!(!result.needs_global);
}

#[test]
fn test_compute_grade_fail_course_takes_priority_over_requires_global() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category 0: avg=40, min=50, RequiresGlobal
    let mut cat_a = Category::new("CatA".to_string(), 50.0);
    cat_a.rules.minimum_average = Some(50.0);
    cat_a.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    cat_a
        .evaluations
        .push(Evaluation::with_grade("A1".to_string(), 40.0));

    // Category 1: avg=20, min=50, FailCourse
    let mut cat_b = Category::new("CatB".to_string(), 50.0);
    cat_b.rules.minimum_average = Some(50.0);
    cat_b.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    cat_b
        .evaluations
        .push(Evaluation::with_grade("B1".to_string(), 20.0));

    course.categories.push(cat_a);
    course.categories.push(cat_b);

    let result = course.compute_grade();
    // FailCourse overrides RequiresGlobal → grade is 0, no needs_global
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("CatB".to_string()));
    assert!(!result.needs_global);
    assert_eq!(result.failed_minimums.len(), 2);
}

#[test]
fn test_compute_grade_fail_course_minimum_met_no_effect() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with avg=60, min=50, action=FailCourse — minimum IS met
    let mut cat = Category::new("Certs".to_string(), 100.0);
    cat.rules.minimum_average = Some(50.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 60.0));

    course.categories.push(cat);

    let result = course.compute_grade();
    // Minimum is met, so FailCourse does not trigger
    assert!((result.grade - 60.0).abs() < 0.01);
    assert!(result.overridden_by.is_none());
    assert!(!result.needs_global);
    assert!(result.failed_minimums.is_empty());
}

#[test]
fn test_compute_grade_fail_course_from_per_eval_minimum() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with min_per_eval=40, action=FailCourse
    // One eval is below 40 → should trigger FailCourse via per-eval violation
    let mut cat = Category::new("Certs".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(40.0);
    cat.rules.on_min_per_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("C2".to_string(), 30.0)); // below 40

    course.categories.push(cat);

    let result = course.compute_grade();
    // Per-eval violation triggers on_minimum_not_met = FailCourse → grade 0
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("Certs".to_string()));
    assert!(!result.needs_global);
    assert!(!result.eval_violations.is_empty());
    assert!(!result.failed_minimums.is_empty());
    assert_eq!(
        result.failed_minimums[0].action,
        MinimumNotMetAction::FailCourse
    );
}

#[test]
fn test_compute_grade_per_eval_minimum_requires_global() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with min_per_eval=50, action=RequiresGlobal
    // One eval below 50 → should trigger RequiresGlobal
    let mut cat = Category::new("Certs".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(50.0);
    cat.rules.on_min_per_eval_not_met = MinimumNotMetAction::RequiresGlobal;
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 70.0));
    cat.evaluations
        .push(Evaluation::with_grade("C2".to_string(), 40.0)); // below 50

    course.categories.push(cat);

    let result = course.compute_grade();
    // Per-eval violation triggers RequiresGlobal, but no global policy
    // configured, so falls back to FinalEqualsAverage (grade = cat avg).
    assert!(!result.needs_global);
    assert_eq!(result.overridden_by, Some("Certs".to_string()));
    // Category average: (70+40)/2 = 55
    assert!((result.grade - 55.0).abs() < 0.01);
}

#[test]
fn test_compute_grade_per_eval_minimum_all_passing_no_trigger() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with min_per_eval=30, action=FailCourse
    // All evals above 30 → no violation, FailCourse should NOT trigger
    let mut cat = Category::new("Certs".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(30.0);
    cat.rules.on_min_per_eval_not_met = MinimumNotMetAction::FailCourse;
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 50.0));
    cat.evaluations
        .push(Evaluation::with_grade("C2".to_string(), 60.0));

    course.categories.push(cat);

    let result = course.compute_grade();
    // No violations → normal grade: (50+60)/2 = 55
    assert!((result.grade - 55.0).abs() < 0.01);
    assert!(result.overridden_by.is_none());
    assert!(!result.needs_global);
    assert!(result.failed_minimums.is_empty());
    assert!(result.eval_violations.is_empty());
}

// =========================================================================
// auto_balance_weights edge cases
// =========================================================================

#[test]
fn test_auto_balance_weights_three_categories() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    course.categories.push(Category::new("A".to_string(), 10.0));
    course.categories.push(Category::new("B".to_string(), 20.0));
    course.categories.push(Category::new("C".to_string(), 70.0));

    course.auto_balance_weights();

    // Should sum to exactly 100.0 after rounding correction on the last category
    assert!((course.total_weight() - 100.0).abs() < 0.01);
    // Each should be ~33.3
    assert!((course.categories[0].weight - 33.3).abs() < 0.1);
    assert!((course.categories[1].weight - 33.3).abs() < 0.1);
}

#[test]
fn test_auto_balance_weights_single_category() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    course.categories.push(Category::new("A".to_string(), 50.0));

    course.auto_balance_weights();

    assert!((course.categories[0].weight - 100.0).abs() < 0.01);
    assert!((course.total_weight() - 100.0).abs() < 0.01);
}

#[test]
fn test_auto_balance_weights_empty_course() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);

    // Should not panic on empty categories
    course.auto_balance_weights();

    assert!(course.categories.is_empty());
}

// =========================================================================
// effective_grades_excluding with drop_lowest
// =========================================================================

#[test]
fn test_effective_grades_excluding_with_drop_lowest() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 20.0));
    cat.evaluations.push(Evaluation::new("T3".to_string())); // eval_idx=2, the target

    // Solving for eval 2: the others are [80, 20]; the target is assumed to
    // survive drop_lowest, so the lowest *other* (20) is dropped, leaving [80].
    // count includes the target's own slot (1 + 1 = 2).
    let (grades, count) = cat.effective_grades_excluding(2);
    assert_eq!(count, 2);
    assert_eq!(grades, vec![80.0]);
}

#[test]
fn test_effective_grades_excluding_target_not_dropped() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    // All high grades except eval 0 which we set to 0
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 90.0)); // will be set to 0
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 70.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 80.0));

    // Solving for eval 0: the others are [70, 80]; dropping their lowest (70)
    // leaves [80], plus the target's own slot -> count 2.
    let (grades, count) = cat.effective_grades_excluding(0);
    assert_eq!(count, 2);
    assert_eq!(grades, vec![80.0]);
}

// =========================================================================
// Drop lowest boundary: 2 evals with drop=1
// =========================================================================

#[test]
fn test_drop_lowest_two_evals_drop_one() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 40.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 90.0));

    // Drop 40, average of [90] = 90
    assert!((cat.average_grade().unwrap() - 90.0).abs() < 0.01);
    assert_eq!(cat.effective_eval_count(), 1);
}

// =========================================================================
// Geometric mean with drop_lowest
// =========================================================================

#[test]
fn test_geometric_mean_with_drop_lowest() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.averaging_method = AveragingMethod::Geometric;
    cat.rules.drop_lowest = 1;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 10.0)); // lowest, will be dropped
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 64.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 36.0));

    // Drop 10, geometric mean of [36, 64] = sqrt(36*64) = sqrt(2304) = 48
    assert!((cat.average_grade().unwrap() - 48.0).abs() < 0.1);
}

// =========================================================================
// Round before weighting with geometric mean
// =========================================================================

#[test]
fn test_round_before_weighting_geometric() {
    let mut cat = Category::new("Tests".to_string(), 60.0);
    cat.rules.averaging_method = AveragingMethod::Geometric;
    cat.rules.round_before_weighting = true;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 64.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 36.0));

    // Geometric mean = 48.0, rounded = 48
    // Weighted = 48 * 60 / 100 = 28.8
    assert!((cat.average_grade().unwrap() - 48.0).abs() < 0.1);
    assert!((cat.weighted_contribution().unwrap() - 28.8).abs() < 0.1);
}

// =========================================================================
// Course current_grade edge cases
// =========================================================================

#[test]
fn test_current_grade_no_evaluations() {
    let course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    assert!(course.current_grade().is_none());
}

#[test]
fn test_current_grade_empty_categories() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    course
        .categories
        .push(Category::new("A".to_string(), 100.0));

    // Category exists but has no evaluations
    assert!(course.current_grade().is_none());
}

#[test]
fn test_current_grade_all_ungraded() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations.push(Evaluation::new("T1".to_string()));
    cat.evaluations.push(Evaluation::new("T2".to_string()));
    course.categories.push(cat);

    // Has evaluations but all ungraded → treated as 0
    let grade = course.current_grade().unwrap();
    assert!((grade - 0.0).abs() < 0.01);
}

// =========================================================================
// Weight validation edge cases
// =========================================================================

#[test]
fn test_weight_validation_tolerance() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    // 33.33 * 3 = 99.99, which is under 100 but within rounding
    course
        .categories
        .push(Category::new("A".to_string(), 33.34));
    course
        .categories
        .push(Category::new("B".to_string(), 33.33));
    course
        .categories
        .push(Category::new("C".to_string(), 33.33));

    // Total = 100.00, should be valid
    assert_eq!(course.validate_weights(), WeightValidation::Valid);
}

// =========================================================================
// Course::new clamps passing_grade
// =========================================================================

#[test]
fn test_course_new_clamps_passing_grade() {
    let course_low = Course::new("Test".to_string(), -10.0);
    assert!((course_low.passing_grade - MIN_GRADE).abs() < 0.01);

    let course_high = Course::new("Test".to_string(), 150.0);
    assert!((course_high.passing_grade - MAX_GRADE).abs() < 0.01);
}

// =========================================================================
// Category weight clamping
// =========================================================================

#[test]
fn test_category_weight_clamped() {
    let cat = Category::new("Test".to_string(), 150.0);
    assert!((cat.weight - 100.0).abs() < 0.01);

    let cat_neg = Category::new("Test".to_string(), -5.0);
    assert!((cat_neg.weight - 0.0).abs() < 0.01);
}

// =========================================================================
// evals_below_minimum edge cases
// =========================================================================

#[test]
fn test_evals_below_minimum_no_requirement() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 10.0));

    // No minimum_per_evaluation set
    assert!(cat.evals_below_minimum().is_none());
}

#[test]
fn test_evals_below_minimum_all_failing() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_per_evaluation = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 20.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 10.0));

    let failing = cat.evals_below_minimum().unwrap();
    assert_eq!(failing, vec![0, 1, 2]);
}

// =========================================================================
// dropped_indices edge cases
// =========================================================================

#[test]
fn test_dropped_indices_no_drop() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0));

    assert!(cat.dropped_indices().is_empty());
}

#[test]
fn test_dropped_indices_drop_exceeds_evals() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 5;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0));

    // drop_lowest >= evaluations.len() → nothing dropped
    assert!(cat.dropped_indices().is_empty());
}

#[test]
fn test_dropped_indices_empty_evals() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;

    assert!(cat.dropped_indices().is_empty());
}

#[test]
fn test_dropped_indices_drop_two() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 2;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 80.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 20.0));
    cat.evaluations
        .push(Evaluation::with_grade("T4".to_string(), 70.0));

    let dropped = cat.dropped_indices();
    assert_eq!(dropped.len(), 2);
    // T3 (index 2, grade 20) and T2 (index 1, grade 30) should be dropped
    assert!(dropped.contains(&1));
    assert!(dropped.contains(&2));
}

// =========================================================================
// meets_minimum with no evaluations
// =========================================================================

#[test]
fn test_meets_minimum_no_evaluations() {
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.minimum_average = Some(50.0);

    // No evaluations → average_grade is None → meets_minimum is None
    assert_eq!(cat.meets_minimum(), None);
}

// =========================================================================
// weighted_contribution edge cases
// =========================================================================

#[test]
fn test_weighted_contribution_no_evaluations() {
    let cat = Category::new("Tests".to_string(), 80.0);
    assert!(cat.weighted_contribution().is_none());
}

#[test]
fn test_weighted_contribution_zero_weight() {
    let mut cat = Category::new("Bonus".to_string(), 0.0);
    cat.evaluations
        .push(Evaluation::with_grade("B1".to_string(), 100.0));

    // 100 * 0 / 100 = 0
    assert!((cat.weighted_contribution().unwrap() - 0.0).abs() < 0.01);
}

// =========================================================================
// to_template roundtrip test
// =========================================================================

#[test]
fn test_to_template_roundtrip() {
    let mut course = Course::new("Fisica".to_string(), 60.0);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(30.0),
        ..Default::default()
    };

    let rules = CategoryRules {
        drop_lowest: 1,
        minimum_average: Some(45.0),
        on_minimum_not_met: MinimumNotMetAction::RequiresGlobal,
        ..Default::default()
    };

    let certs = Category::with_rules(
        "Certamen".to_string(),
        70.0,
        vec![
            Evaluation::with_grade("C1".to_string(), 80.0),
            Evaluation::with_grade("C2".to_string(), 50.0),
            Evaluation::new("C3".to_string()),
        ],
        rules,
    );
    // Ensure grades exist to verify they are NOT carried into the template
    assert_eq!(certs.graded_count(), 2);

    let controls = Category::with_evaluations(
        "Control".to_string(),
        30.0,
        vec![
            Evaluation::new("Co1".to_string()),
            Evaluation::new("Co2".to_string()),
        ],
    );

    course.categories.push(certs);
    course.categories.push(controls);

    let template = course.to_template("Fisica Template".to_string(), "test desc".to_string());

    assert_eq!(template.name, "Fisica Template");
    assert_eq!(template.description, "test desc");
    assert_eq!(template.categories.len(), 2);

    // Verify structure preserved
    assert_eq!(template.categories[0].name, "Certamen");
    assert!((template.categories[0].weight - 70.0).abs() < 0.01);
    assert_eq!(template.categories[0].default_evaluation_count, 3);
    assert_eq!(template.categories[0].rules.drop_lowest, 1);
    assert_eq!(template.categories[0].rules.minimum_average, Some(45.0));
    assert_eq!(
        template.categories[0].rules.on_minimum_not_met,
        MinimumNotMetAction::RequiresGlobal
    );

    assert_eq!(template.categories[1].name, "Control");
    assert!((template.categories[1].weight - 30.0).abs() < 0.01);
    assert_eq!(template.categories[1].default_evaluation_count, 2);
    assert!(template.categories[1].rules.is_default());

    // Verify global config preserved in template
    assert_eq!(
        template.global_policy,
        GlobalExamPolicy::Weighted {
            semester_weight: 0.7,
            global_weight: 0.3,
        }
    );
    assert_eq!(template.global_eligibility.min_grade, Some(30.0));

    // Roundtrip: create a new course from the template
    let course2 = Course::from_template("Fisica 2".to_string(), 60.0, &template);
    assert_eq!(course2.categories.len(), 2);
    assert_eq!(course2.categories[0].evaluations.len(), 3);
    assert_eq!(course2.categories[0].rules.drop_lowest, 1);
    assert_eq!(course2.categories[0].rules.minimum_average, Some(45.0));
    // Grades should NOT be carried over — all evals ungraded
    assert_eq!(course2.categories[0].graded_count(), 0);
    assert_eq!(course2.categories[1].evaluations.len(), 2);
    assert_eq!(course2.categories[1].graded_count(), 0);
    // Global config should be carried over from template
    assert_eq!(
        course2.global_policy,
        GlobalExamPolicy::Weighted {
            semester_weight: 0.7,
            global_weight: 0.3,
        }
    );
    assert_eq!(course2.global_eligibility.min_grade, Some(30.0));
}

// =========================================================================
// is_passing on empty category
// =========================================================================

#[test]
fn test_is_passing_empty_category() {
    let cat = Category::new("Tests".to_string(), 100.0);
    // No evaluations → average_grade is None → is_passing is None
    assert_eq!(cat.is_passing(55.0), None);
}

// =========================================================================
// compute_grade with drop_lowest + minimum average interaction
// =========================================================================

#[test]
fn test_compute_grade_drop_lowest_affects_minimum_check() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with drop_lowest=1 and minimum_average=50
    // Grades: [20, 60, 70] → after drop 20 → avg = (60+70)/2 = 65 → passes minimum
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.rules.minimum_average = Some(50.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 20.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 60.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 70.0));
    course.categories.push(cat);

    let result = course.compute_grade();
    // After dropping 20, avg = 65 which is >= 50 → minimum met
    // Grade = 65 (no override)
    assert!((result.grade - 65.0).abs() < 0.01);
    assert!(result.overridden_by.is_none());
    assert!(result.failed_minimums.is_empty());
}

#[test]
fn test_compute_grade_drop_lowest_still_fails_minimum() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    // Category with drop_lowest=1 and minimum_average=50
    // Grades: [10, 20, 30] → after drop 10 → avg = (20+30)/2 = 25 → fails minimum
    let mut cat = Category::new("Tests".to_string(), 100.0);
    cat.rules.drop_lowest = 1;
    cat.rules.minimum_average = Some(50.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat.evaluations
        .push(Evaluation::with_grade("T1".to_string(), 10.0));
    cat.evaluations
        .push(Evaluation::with_grade("T2".to_string(), 20.0));
    cat.evaluations
        .push(Evaluation::with_grade("T3".to_string(), 30.0));
    course.categories.push(cat);

    let result = course.compute_grade();
    // After dropping 10, avg = 25 which is < 50 → minimum not met → override
    assert!((result.grade - 25.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("Tests".to_string()));
    assert_eq!(result.failed_minimums.len(), 1);
}

// =========================================================================
// Global Exam tests
// =========================================================================

/// Helper: build a course with two categories (Certamenes 70%, Controles 30%)
/// and the given grades. Returns the course ready for global-exam tests.
fn make_global_test_course(cert_grades: &[f64], ctrl_grades: &[f64]) -> Course {
    let mut course = Course::new("Fisica".to_string(), DEFAULT_PASSING_GRADE);

    let mut certs = Category::new("Certamenes".to_string(), 70.0);
    for (i, &g) in cert_grades.iter().enumerate() {
        certs
            .evaluations
            .push(Evaluation::with_grade(format!("C{}", i + 1), g));
    }
    course.categories.push(certs);

    let mut ctrls = Category::new("Controles".to_string(), 30.0);
    for (i, &g) in ctrl_grades.iter().enumerate() {
        ctrls
            .evaluations
            .push(Evaluation::with_grade(format!("Q{}", i + 1), g));
    }
    course.categories.push(ctrls);

    course
}

// ---- Policy: None (default) ----

#[test]
fn test_global_policy_none_by_default() {
    let course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    assert_eq!(course.global_policy, GlobalExamPolicy::None);
    assert!(!course.is_eligible_for_global());
}

#[test]
fn test_global_policy_none_grade_after_global_is_none() {
    let mut course = make_global_test_course(&[40.0, 50.0, 60.0], &[70.0, 80.0]);
    course.global_exam_grade = Some(90.0); // Even if a grade is set
    let result = course.compute_grade();
    assert!(result.grade_after_global.is_none());
}

// ---- Policy: Weighted ----

#[test]
fn test_global_weighted_basic() {
    // Certamenes: avg=40, Controles: avg=60
    // Semester = 40*0.7 + 60*0.3 = 28 + 18 = 46
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_exam_grade = Some(80.0);

    let result = course.compute_grade();
    // grade_after_global = 46 * 0.7 + 80 * 0.3 = 32.2 + 24 = 56.2
    let after = result.grade_after_global.unwrap();
    assert!((after - 56.2).abs() < 0.01);
}

#[test]
fn test_global_weighted_still_fails() {
    // Semester = 30*0.7 + 40*0.3 = 21 + 12 = 33
    let mut course = make_global_test_course(&[30.0], &[40.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_exam_grade = Some(50.0);

    let result = course.compute_grade();
    // 33 * 0.7 + 50 * 0.3 = 23.1 + 15 = 38.1
    let after = result.grade_after_global.unwrap();
    assert!((after - 38.1).abs() < 0.01);
    assert!(!course.is_passing_grade(after));
}

#[test]
fn test_global_weighted_no_grade_entered() {
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    // No global_exam_grade set
    let result = course.compute_grade();
    assert!(result.grade_after_global.is_none());
}

#[test]
fn test_global_weighted_60_40() {
    // Different weights: 60/40
    // Semester = 50*0.7 + 60*0.3 = 35 + 18 = 53
    let mut course = make_global_test_course(&[50.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.6,
        global_weight: 0.4,
    };
    course.global_exam_grade = Some(70.0);

    let result = course.compute_grade();
    // 53 * 0.6 + 70 * 0.4 = 31.8 + 28 = 59.8
    let after = result.grade_after_global.unwrap();
    assert!((after - 59.8).abs() < 0.01);
}

// ---- Policy: ReplacesWorstGrade ----

#[test]
fn test_global_replaces_worst_basic() {
    // Certamenes: [30, 60, 80] → avg=56.67, Controles: [70] → avg=70
    // Semester = 56.67*0.7 + 70*0.3 = 39.67 + 21 = 60.67
    let mut course = make_global_test_course(&[30.0, 60.0, 80.0], &[70.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0); // Certamenes
    course.global_exam_grade = Some(90.0);

    let result = course.compute_grade();
    // Replace worst (30) with 90 → Certamenes: [90, 60, 80] → avg=76.67
    // New grade = 76.67*0.7 + 70*0.3 = 53.67 + 21 = 74.67
    let after = result.grade_after_global.unwrap();
    assert!((after - 74.67).abs() < 0.1);
}

#[test]
fn test_global_replaces_worst_grade_can_lower() {
    // Certamenes: [50, 60, 80] → avg=63.33
    // Controles: [70] → avg=70
    // Semester = 63.33*0.7 + 70*0.3 = 44.33 + 21 = 65.33
    let mut course = make_global_test_course(&[50.0, 60.0, 80.0], &[70.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0);
    // Global grade is LOWER than worst (50), so it lowers the average
    course.global_exam_grade = Some(20.0);

    let result = course.compute_grade();
    // Replace worst (50) with 20 → Certamenes: [20, 60, 80] → avg=53.33
    // New grade = 53.33*0.7 + 70*0.3 = 37.33 + 21 = 58.33
    let after = result.grade_after_global.unwrap();
    assert!((after - 58.33).abs() < 0.1);
    // Was 65.33 before global, now 58.33 — grade went DOWN
    assert!(after < result.grade);
}

#[test]
fn test_global_replaces_worst_auto_detect_category() {
    // Certamenes has RequiresGlobal rule and fails minimum
    let mut course = make_global_test_course(&[30.0, 40.0], &[80.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    // Don't set global_target_category — should auto-detect

    course.categories[0].rules.minimum_average = Some(50.0);
    course.categories[0].rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;

    course.global_exam_grade = Some(70.0);

    let result = course.compute_grade();
    assert!(result.needs_global);
    // Auto-detected target = Certamenes (idx 0)
    // Replace worst (30) with 70 → [70, 40] → avg=55
    // New grade = 55*0.7 + 80*0.3 = 38.5 + 24 = 62.5
    let after = result.grade_after_global.unwrap();
    assert!((after - 62.5).abs() < 0.1);
}

#[test]
fn test_global_replaces_worst_no_grade_entered() {
    let mut course = make_global_test_course(&[30.0, 60.0], &[70.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0);
    // No global grade
    let result = course.compute_grade();
    assert!(result.grade_after_global.is_none());
}

// ---- Eligibility ----

#[test]
fn test_global_eligibility_no_restriction() {
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    // Default eligibility = no restrictions
    assert!(course.is_eligible_for_global());
}

#[test]
fn test_global_eligibility_within_range() {
    // Semester = 40*0.7 + 60*0.3 = 46
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(45.0),
        ..Default::default()
    };
    // 46 >= 45 → eligible
    assert!(course.is_eligible_for_global());
}

#[test]
fn test_global_eligibility_below_minimum() {
    // Semester = 30*0.7 + 40*0.3 = 33
    let mut course = make_global_test_course(&[30.0], &[40.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(45.0),
        ..Default::default()
    };
    // 33 < 45 → not eligible
    assert!(!course.is_eligible_for_global());
}

#[test]
fn test_global_eligibility_only_min() {
    // Semester = 50*0.7 + 60*0.3 = 53
    let mut course = make_global_test_course(&[50.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(40.0),
        ..Default::default()
    };
    // 53 >= 40 → eligible
    assert!(course.is_eligible_for_global());
}

#[test]
fn test_global_eligibility_replaces_worst_no_restriction() {
    // ReplacesWorstGrade with no eligibility restrictions (anyone can take it)
    let mut course = make_global_test_course(&[90.0], &[95.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    // Even with high grades, eligible since no restriction
    assert!(course.is_eligible_for_global());
}

// ---- Needed Global Grade ----

#[test]
fn test_needed_global_weighted() {
    // Semester = 40*0.7 + 60*0.3 = 46
    // Need: 46*0.7 + X*0.3 >= 54.5 (55 - 0.5)
    // 32.2 + 0.3X >= 54.5
    // 0.3X >= 22.3
    // X >= 74.33
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Warning);
    assert!((needed.value.unwrap() - 74.33).abs() < 0.1);
}

#[test]
fn test_needed_global_weighted_impossible() {
    // Semester = 20*0.7 + 20*0.3 = 20
    // Need: 20*0.7 + X*0.3 >= 54.5
    // 14 + 0.3X >= 54.5
    // X >= 135 → impossible (> 100)
    let mut course = make_global_test_course(&[20.0], &[20.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Failure);
    assert!(needed.value.unwrap() > 100.0);
}

#[test]
fn test_needed_global_weighted_already_passing() {
    // Semester = 80*0.7 + 90*0.3 = 83
    // Already passing, needed would be <= 0
    let mut course = make_global_test_course(&[80.0], &[90.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Success);
}

#[test]
fn test_needed_global_replaces_worst() {
    // Certamenes: [30, 70, 80] → avg=60, Controles: [50] → avg=50
    // Semester = 60*0.7 + 50*0.3 = 42 + 15 = 57
    // Replace worst (30) with X → new avg = (X + 70 + 80)/3
    // new_contribution_certs = ((X+150)/3) * 70/100
    // total = ((X+150)/3)*0.7 + 15 >= 54.5
    // ((X+150)/3)*0.7 >= 39.5
    // (X+150)/3 >= 56.43
    // X+150 >= 169.29
    // X >= 19.29
    let mut course = make_global_test_course(&[30.0, 70.0, 80.0], &[50.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0);

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Warning);
    assert!((needed.value.unwrap() - 19.29).abs() < 0.5);
}

#[test]
fn test_needed_global_replaces_worst_impossible() {
    // Certamenes: [10, 10, 10] → avg=10, Controles: [10] → avg=10
    // Semester = 10*0.7 + 10*0.3 = 10
    // Replace worst (10) with X → avg = (X+10+10)/3
    // ((X+20)/3)*0.7 + 3 >= 54.5
    // ((X+20)/3)*0.7 >= 51.5
    // (X+20)/3 >= 73.57
    // X >= 200.71 → impossible
    let mut course = make_global_test_course(&[10.0, 10.0, 10.0], &[10.0]);
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0);

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Failure);
}

#[test]
fn test_needed_global_no_policy() {
    let course = make_global_test_course(&[40.0], &[60.0]);
    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Info);
}

#[test]
fn test_needed_global_not_eligible() {
    // Semester = 30*0.7 + 40*0.3 = 33 → below min eligibility of 45
    let mut course = make_global_test_course(&[30.0], &[40.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(45.0),
        ..Default::default()
    };

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Failure);
    assert!(needed.value.is_none());
}

#[test]
fn test_needed_global_already_took_global_passing() {
    // Semester = 46, global = 80, after = 46*0.7 + 80*0.3 = 56.2 → pass
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_exam_grade = Some(80.0);

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Success);
}

#[test]
fn test_needed_global_already_took_global_failing() {
    // Semester = 46, global = 30, after = 46*0.7 + 30*0.3 = 32.2 + 9 = 41.2
    let mut course = make_global_test_course(&[40.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_exam_grade = Some(30.0);

    let needed = course.needed_global_grade();
    assert_eq!(needed.status, NeededGradeStatus::Failure);
}

// ---- Serde backward compatibility ----

#[test]
fn test_global_fields_serde_default() {
    // A course serialized without global fields should deserialize fine
    let json = r#"{
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "Old Course",
        "passing_grade": 55.0,
        "categories": []
    }"#;

    let course: Course = serde_json::from_str(json).unwrap();
    assert_eq!(course.global_policy, GlobalExamPolicy::None);
    assert!(course.global_eligibility.min_grade.is_none());
    assert!(course.global_exam_grade.is_none());
    assert!(course.global_target_category.is_none());
}

#[test]
fn test_global_weighted_serde_roundtrip() {
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_eligibility = GlobalEligibility {
        min_grade: Some(45.0),
        ..Default::default()
    };
    course.global_exam_grade = Some(75.0);

    let json = serde_json::to_string(&course).unwrap();
    let deserialized: Course = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.global_policy, course.global_policy);
    assert_eq!(deserialized.global_eligibility.min_grade, Some(45.0));
    assert_eq!(deserialized.global_exam_grade, Some(75.0));
}

// ---- Edge cases ----

#[test]
fn test_global_replaces_worst_with_drop_lowest() {
    // Certamenes: [20, 40, 60, 80] with drop_lowest=1 → effective: [40, 60, 80] avg=60
    // Controles: [50] → avg=50
    // Semester = 60*0.7 + 50*0.3 = 42 + 15 = 57
    let mut course = make_global_test_course(&[20.0, 40.0, 60.0, 80.0], &[50.0]);
    course.categories[0].rules.drop_lowest = 1;
    course.global_policy = GlobalExamPolicy::ReplacesWorstGrade;
    course.global_target_category = Some(0);
    // Replace worst GRADED eval (20) with 90
    // After replacement: [90, 40, 60, 80] with drop_lowest=1 → effective: [60, 80, 90] avg=76.67
    // New grade = 76.67*0.7 + 50*0.3 = 53.67 + 15 = 68.67
    course.global_exam_grade = Some(90.0);

    let result = course.compute_grade();
    let after = result.grade_after_global.unwrap();
    assert!((after - 68.67).abs() < 0.1);
}

#[test]
fn test_global_weighted_100_0_edge() {
    // Edge case: semester_weight=1.0, global_weight=0.0
    // Global doesn't matter at all
    let mut course = make_global_test_course(&[50.0], &[60.0]);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 1.0,
        global_weight: 0.0,
    };
    course.global_exam_grade = Some(100.0);

    let result = course.compute_grade();
    let after = result.grade_after_global.unwrap();
    // 53 * 1.0 + 100 * 0.0 = 53 (semester grade unchanged)
    assert!((after - result.grade).abs() < 0.01);
}

#[test]
fn test_needs_global_when_np_below_passing_with_global_policy() {
    // When NP < passing_grade and a global policy is configured,
    // needs_global should be true even without any category triggering
    // RequiresGlobal.  This covers the common rule:
    //   "NP < 54.5 => must take certamen global"
    // Certamenes: 55 * 0.60 = 33, Laboratorio: 40 * 0.40 = 16 => NP = 49
    let mut course = Course::new("MAT270".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let mut certs = Category::new("Certamenes".to_string(), 60.0);
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 55.0));
    course.categories.push(certs);

    let mut labs = Category::new("Laboratorio".to_string(), 40.0);
    labs.evaluations
        .push(Evaluation::with_grade("Lab".to_string(), 40.0));
    course.categories.push(labs);

    let result = course.compute_grade();
    // NP = 55*0.6 + 40*0.4 = 33 + 16 = 49, which is < 55
    assert!((result.grade - 49.0).abs() < 0.01);
    assert!(
        result.needs_global,
        "needs_global should be true when NP < passing and global policy exists"
    );
    assert!(result.overridden_by.is_none());
}

#[test]
fn test_no_needs_global_when_np_above_passing_with_global_policy() {
    // When NP >= passing_grade, needs_global should NOT be set even if a
    // global policy is configured — the student already passes.
    let mut course = Course::new("MAT270".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let mut certs = Category::new("Certamenes".to_string(), 60.0);
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 70.0));
    course.categories.push(certs);

    let mut labs = Category::new("Laboratorio".to_string(), 40.0);
    labs.evaluations
        .push(Evaluation::with_grade("Lab".to_string(), 60.0));
    course.categories.push(labs);

    let result = course.compute_grade();
    // NP = 70*0.6 + 60*0.4 = 42 + 24 = 66, which is >= 55
    assert!((result.grade - 66.0).abs() < 0.01);
    assert!(
        !result.needs_global,
        "needs_global should be false when NP >= passing"
    );
}

#[test]
fn test_no_needs_global_when_no_global_policy() {
    // When there is no global policy, needs_global should never be set,
    // even if NP < passing_grade.
    let mut course = Course::new("MAT270".to_string(), DEFAULT_PASSING_GRADE);
    // global_policy is None by default

    let mut certs = Category::new("Certamenes".to_string(), 60.0);
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 55.0));
    course.categories.push(certs);

    let mut labs = Category::new("Laboratorio".to_string(), 40.0);
    labs.evaluations
        .push(Evaluation::with_grade("Lab".to_string(), 40.0));
    course.categories.push(labs);

    let result = course.compute_grade();
    // NP = 49, below passing, but no global policy
    assert!(
        !result.needs_global,
        "needs_global should be false when no global policy is configured"
    );
}

#[test]
fn test_needs_global_np_below_passing_with_global_grade_entered() {
    // When NP < passing and global grade has been entered, needs_global
    // should still be true, but grade_after_global should be computed.
    // NF = NP * 0.7 + CG * 0.3
    let mut course = Course::new("MAT270".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_exam_grade = Some(80.0);

    let mut certs = Category::new("Certamenes".to_string(), 60.0);
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 55.0));
    course.categories.push(certs);

    let mut labs = Category::new("Laboratorio".to_string(), 40.0);
    labs.evaluations
        .push(Evaluation::with_grade("Lab".to_string(), 40.0));
    course.categories.push(labs);

    let result = course.compute_grade();
    // NP = 49, NF = 49 * 0.7 + 80 * 0.3 = 34.3 + 24 = 58.3
    assert!(result.needs_global);
    let after = result.grade_after_global.unwrap();
    assert!((after - 58.3).abs() < 0.01);
    assert!(
        course.is_passing_grade(after),
        "58.3 rounds to 58, which should pass"
    );
}

// =============================================================================
// minimum_one_eval (at least one eval >= X)
// =============================================================================

#[test]
fn test_any_eval_meets_minimum_no_requirement() {
    let mut cat = Category::new("Certs".to_string(), 80.0);
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    assert_eq!(cat.any_eval_meets_minimum(), None);
}

#[test]
fn test_any_eval_meets_minimum_passing() {
    let mut cat = Category::new("Certs".to_string(), 80.0);
    cat.rules.minimum_one_eval = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("C2".to_string(), 60.0));
    // At least one eval (60) >= 50 → passes
    assert_eq!(cat.any_eval_meets_minimum(), Some(true));
}

#[test]
fn test_any_eval_meets_minimum_failing() {
    let mut cat = Category::new("Certs".to_string(), 80.0);
    cat.rules.minimum_one_eval = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    cat.evaluations
        .push(Evaluation::with_grade("C2".to_string(), 40.0));
    // No eval >= 50 → fails
    assert_eq!(cat.any_eval_meets_minimum(), Some(false));
}

#[test]
fn test_any_eval_meets_minimum_ungraded_ignored() {
    let mut cat = Category::new("Certs".to_string(), 80.0);
    cat.rules.minimum_one_eval = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 40.0)); // graded, below
    cat.evaluations.push(Evaluation::new("C2".to_string())); // ungraded, ignored
    // One eval is graded (below the threshold); the ungraded one is ignored, so
    // the requirement is judged failed on the strength of the graded eval alone.
    assert_eq!(cat.any_eval_meets_minimum(), Some(false));
}

#[test]
fn test_any_eval_meets_minimum_exact_boundary() {
    let mut cat = Category::new("Certs".to_string(), 80.0);
    cat.rules.minimum_one_eval = Some(50.0);
    cat.evaluations
        .push(Evaluation::with_grade("C1".to_string(), 50.0));
    // Exactly 50 >= 50 → passes
    assert_eq!(cat.any_eval_meets_minimum(), Some(true));
}

#[test]
fn test_compute_grade_min_one_eval_triggers_requires_global() {
    let mut course = Course::new("Physics".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let mut certs = Category::new("Certamenes".to_string(), 80.0);
    certs.rules.minimum_one_eval = Some(50.0);
    certs.rules.on_min_one_eval_not_met = MinimumNotMetAction::RequiresGlobal;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    certs
        .evaluations
        .push(Evaluation::with_grade("C2".to_string(), 40.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 20.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 80.0));

    course.categories.push(certs);
    course.categories.push(quizzes);

    let result = course.compute_grade();
    // No cert >= 50 → RequiresGlobal triggered
    assert!(result.needs_global);
    assert!(!result.failed_minimums.is_empty());
}

#[test]
fn test_compute_grade_min_one_eval_no_trigger_when_one_passes() {
    let mut course = Course::new("Physics".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let mut certs = Category::new("Certamenes".to_string(), 80.0);
    certs.rules.minimum_one_eval = Some(50.0);
    certs.rules.on_min_one_eval_not_met = MinimumNotMetAction::RequiresGlobal;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    certs
        .evaluations
        .push(Evaluation::with_grade("C2".to_string(), 55.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 20.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 80.0));

    course.categories.push(certs);
    course.categories.push(quizzes);

    let result = course.compute_grade();
    // C2 = 55 >= 50 → rule satisfied, no trigger from min_one_eval
    let min_one_triggered = result
        .failed_minimums
        .iter()
        .any(|fm| fm.category_name == "Certamenes" && !fm.from_per_eval);
    assert!(!min_one_triggered);
}

#[test]
fn test_compute_grade_min_one_eval_fail_course() {
    let mut course = Course::new("Physics".to_string(), DEFAULT_PASSING_GRADE);

    let mut certs = Category::new("Certamenes".to_string(), 80.0);
    certs.rules.minimum_one_eval = Some(50.0);
    certs.rules.on_min_one_eval_not_met = MinimumNotMetAction::FailCourse;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    certs
        .evaluations
        .push(Evaluation::with_grade("C2".to_string(), 40.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 20.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 80.0));

    course.categories.push(certs);
    course.categories.push(quizzes);

    let result = course.compute_grade();
    // No cert >= 50 and action is FailCourse → grade forced to 0
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("Certamenes".to_string()));
}

#[test]
fn test_min_one_eval_independent_from_min_avg() {
    // When both minimum_average and minimum_one_eval fail for the same
    // category, each rule independently produces its own FailedMinimum
    // entry because they have independent action fields.
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);

    let mut certs = Category::new("Certamenes".to_string(), 80.0);
    certs.rules.minimum_average = Some(55.0);
    certs.rules.on_minimum_not_met = MinimumNotMetAction::RequiresGlobal;
    certs.rules.minimum_one_eval = Some(50.0);
    certs.rules.on_min_one_eval_not_met = MinimumNotMetAction::FailCourse;
    certs
        .evaluations
        .push(Evaluation::with_grade("C1".to_string(), 30.0));
    certs
        .evaluations
        .push(Evaluation::with_grade("C2".to_string(), 40.0));

    let mut quizzes = Category::new("Quizzes".to_string(), 20.0);
    quizzes
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), 80.0));

    course.categories.push(certs);
    course.categories.push(quizzes);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let result = course.compute_grade();
    // Both rules fail independently — two FailedMinimum entries for "Certamenes"
    let cert_failures: Vec<_> = result
        .failed_minimums
        .iter()
        .filter(|fm| fm.category_name == "Certamenes")
        .collect();
    assert_eq!(
        cert_failures.len(),
        2,
        "each rule produces its own FailedMinimum"
    );
    // FailCourse (from min_one_eval) takes priority → grade forced to 0
    assert!((result.grade - 0.0).abs() < 0.01);
    assert_eq!(result.overridden_by, Some("Certamenes".to_string()));
}

#[test]
fn test_no_needs_global_when_no_evaluations() {
    // A course with a global policy but no categories/evaluations
    // should NOT mark needs_global — there are no grades yet.
    let mut course = Course::new("Empty".to_string(), DEFAULT_PASSING_GRADE);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };

    let result = course.compute_grade();
    assert!(
        !result.needs_global,
        "no evaluations → needs_global must be false"
    );
    assert!((result.grade - 0.0).abs() < f64::EPSILON);
    assert!(result.overridden_by.is_none());
}

// =========================================================================
// Weighted Evaluations tests
// =========================================================================

#[test]
fn test_weighted_evals_average_basic() {
    // Category with weighted evaluations: 40% weight on 80, 60% weight on 60
    // Expected: 0.4 * 80 + 0.6 * 60 = 32 + 36 = 68
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("Report".to_string(), 80.0, 40.0),
        Evaluation::with_weight("Presentation".to_string(), 60.0, 60.0),
    ];
    let cat = Category::with_rules("Project".to_string(), 100.0, evals, rules);
    let avg = cat.average_grade().unwrap();
    assert!((avg - 68.0).abs() < 0.01, "Expected 68.0, got {avg}");
}

#[test]
fn test_weighted_evals_average_three_components() {
    // Informe avance 20%, Informe final 40%, Presentación 40%
    // Grades: 70, 80, 90
    // Expected: 0.2*70 + 0.4*80 + 0.4*90 = 14 + 32 + 36 = 82
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("Informe avance".to_string(), 70.0, 20.0),
        Evaluation::with_weight("Informe final".to_string(), 80.0, 40.0),
        Evaluation::with_weight("Presentacion".to_string(), 90.0, 40.0),
    ];
    let cat = Category::with_rules("Proyecto".to_string(), 5.0, evals, rules);
    let avg = cat.average_grade().unwrap();
    assert!((avg - 82.0).abs() < 0.01, "Expected 82.0, got {avg}");
}

#[test]
fn test_weighted_evals_ungraded_counts_as_zero() {
    // Two evals: one graded 100 (weight 50%), one ungraded (weight 50%)
    // Expected: 0.5 * 100 + 0.5 * 0 = 50
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let mut eval1 = Evaluation::new("Done".to_string());
    eval1.grade = Some(100.0);
    eval1.weight = Some(50.0);
    let mut eval2 = Evaluation::new("Not done".to_string());
    eval2.weight = Some(50.0);
    let cat = Category::with_rules("Cat".to_string(), 100.0, vec![eval1, eval2], rules);
    let avg = cat.average_grade().unwrap();
    assert!((avg - 50.0).abs() < 0.01, "Expected 50.0, got {avg}");
}

#[test]
fn test_weighted_evals_weighted_contribution() {
    // Category weight 5%, weighted evals: 0.2*70 + 0.4*80 + 0.4*90 = 82
    // Contribution: 82 * 5 / 100 = 4.1
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 70.0, 20.0),
        Evaluation::with_weight("B".to_string(), 80.0, 40.0),
        Evaluation::with_weight("C".to_string(), 90.0, 40.0),
    ];
    let cat = Category::with_rules("Project".to_string(), 5.0, evals, rules);
    let contribution = cat.weighted_contribution().unwrap();
    assert!(
        (contribution - 4.1).abs() < 0.01,
        "Expected 4.1, got {contribution}"
    );
}

#[test]
fn test_weighted_evals_course_grade() {
    // Simulate the slide example:
    // NF = 0.55*PC + 0.05*PT + 0.4*TI
    // PC (controles, equal weight): 3 evals all 70 => avg 70
    // PT (proyecto, weighted): avance 20% => 60, final 40% => 80, pres 40% => 90
    //   PT avg = 0.2*60 + 0.4*80 + 0.4*90 = 12 + 32 + 36 = 80
    // TI (tarea individual, equal weight): 1 eval => 85
    // NF = 0.55*70 + 0.05*80 + 0.4*85 = 38.5 + 4.0 + 34.0 = 76.5
    let mut course = Course::new("ILI 101".to_string(), DEFAULT_PASSING_GRADE);

    // Controles (55%) — equal weight
    let controles = Category::with_evaluations(
        "Controles".to_string(),
        55.0,
        vec![
            Evaluation::with_grade("C1".to_string(), 70.0),
            Evaluation::with_grade("C2".to_string(), 70.0),
            Evaluation::with_grade("C3".to_string(), 70.0),
        ],
    );

    // Proyecto (5%) — weighted evaluations
    let proyecto_rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let proyecto = Category::with_rules(
        "Proyecto".to_string(),
        5.0,
        vec![
            Evaluation::with_weight("Informe avance".to_string(), 60.0, 20.0),
            Evaluation::with_weight("Informe final".to_string(), 80.0, 40.0),
            Evaluation::with_weight("Presentacion".to_string(), 90.0, 40.0),
        ],
        proyecto_rules,
    );

    // Tarea Individual (40%) — equal weight, 1 eval
    let tarea = Category::with_evaluations(
        "Tarea Individual".to_string(),
        40.0,
        vec![Evaluation::with_grade("TI 1".to_string(), 85.0)],
    );

    course.categories = vec![controles, proyecto, tarea];
    let grade = course.current_grade().unwrap();
    assert!((grade - 76.5).abs() < 0.1, "Expected ~76.5, got {grade}");
}

#[test]
fn test_weighted_evals_drop_lowest_ignored() {
    // When weighted_evaluations is true, drop_lowest should have no effect
    let rules = CategoryRules {
        weighted_evaluations: true,
        drop_lowest: 1,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 10.0, 50.0),
        Evaluation::with_weight("B".to_string(), 90.0, 50.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);

    // With weighted mode, drop_lowest is ignored:
    // avg = 0.5 * 10 + 0.5 * 90 = 50
    let avg = cat.average_grade().unwrap();
    assert!((avg - 50.0).abs() < 0.01, "Expected 50.0, got {avg}");

    // dropped_indices should be empty
    assert!(cat.dropped_indices().is_empty());
}

#[test]
fn test_weighted_evals_validate_weights_valid() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 80.0, 60.0),
        Evaluation::with_weight("B".to_string(), 70.0, 40.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    assert_eq!(cat.validate_eval_weights(), WeightValidation::Valid);
}

#[test]
fn test_weighted_evals_validate_weights_under() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 80.0, 30.0),
        Evaluation::with_weight("B".to_string(), 70.0, 40.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    assert!(matches!(
        cat.validate_eval_weights(),
        WeightValidation::Under(t) if (t - 70.0).abs() < 0.01
    ));
}

#[test]
fn test_weighted_evals_validate_weights_over() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 80.0, 70.0),
        Evaluation::with_weight("B".to_string(), 70.0, 40.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    assert!(matches!(
        cat.validate_eval_weights(),
        WeightValidation::Over(t) if (t - 110.0).abs() < 0.01
    ));
}

#[test]
fn test_weighted_evals_validate_weights_empty() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let cat = Category::with_rules("Cat".to_string(), 100.0, Vec::new(), rules);
    assert_eq!(cat.validate_eval_weights(), WeightValidation::Empty);
}

#[test]
fn test_weighted_evals_total_eval_weight() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 80.0, 20.0),
        Evaluation::with_weight("B".to_string(), 70.0, 40.0),
        Evaluation::with_weight("C".to_string(), 90.0, 40.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    assert!((cat.total_eval_weight() - 100.0).abs() < 0.01);
}

#[test]
fn test_weighted_evals_none_weight_treated_as_zero() {
    // Evaluation with weight = None in a weighted category is treated as 0% weight
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let mut eval_with_weight = Evaluation::with_grade("A".to_string(), 100.0);
    eval_with_weight.weight = Some(100.0);
    let eval_no_weight = Evaluation::with_grade("B".to_string(), 50.0);
    // B has weight = None => treated as 0%
    let cat = Category::with_rules(
        "Cat".to_string(),
        100.0,
        vec![eval_with_weight, eval_no_weight],
        rules,
    );
    // avg = 1.0 * 100 + 0.0 * 50 = 100
    let avg = cat.average_grade().unwrap();
    assert!((avg - 100.0).abs() < 0.01, "Expected 100.0, got {avg}");
}

#[test]
fn test_weighted_evals_needed_grade() {
    // Course with one weighted category (100%)
    // Two evals: A (60% weight, grade 50), B (40% weight, unknown)
    // Course passing grade: 55
    // We need: 0.6*50 + 0.4*x >= 54.5  (rounding)
    // 30 + 0.4x >= 54.5 => 0.4x >= 24.5 => x >= 61.25
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 50.0, 60.0),
        Evaluation::with_weight("B".to_string(), 0.0, 40.0), // ungraded, will be solved
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    course.categories = vec![cat];

    let needed = course.needed_grade_for_evaluation(0, 1, true);
    assert_eq!(needed.status, NeededGradeStatus::Warning);
    let val = needed.value.unwrap();
    assert!((val - 61.25).abs() < 0.1, "Expected ~61.25, got {val}");
}

#[test]
fn test_weighted_evals_needed_grade_already_passing() {
    // Both evals graded, course already passing
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 80.0, 50.0),
        Evaluation::with_weight("B".to_string(), 90.0, 50.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    course.categories = vec![cat];

    let needed = course.needed_grade_for_evaluation(0, 0, false);
    assert_eq!(needed.status, NeededGradeStatus::Success);
}

#[test]
fn test_weighted_evals_needed_grade_zero_weight_eval() {
    // Evaluation with 0% weight cannot affect outcome
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let mut eval = Evaluation::new("Zero".to_string());
    eval.weight = Some(0.0);
    let cat = Category::with_rules("Cat".to_string(), 100.0, vec![eval], rules);
    course.categories = vec![cat];

    let needed = course.needed_grade_for_evaluation(0, 0, true);
    assert_eq!(needed.status, NeededGradeStatus::Failure);
}

#[test]
fn test_weighted_evals_needed_grade_multi_category() {
    // Two categories: one normal (60%), one weighted (40%)
    // Normal cat: 1 eval graded 70
    // Weighted cat: eval A (50%, grade 80), eval B (50%, unknown)
    // Course grade = 0.6*70 + 0.4*(0.5*80 + 0.5*x)
    //             = 42 + 0.4*(40 + 0.5x)
    //             = 42 + 16 + 0.2x
    //             = 58 + 0.2x
    // Need: 58 + 0.2x >= 54.5  => 0.2x >= -3.5 => x >= -17.5
    // Since x >= 0 always, any grade works => Success(0)
    let mut course = Course::new("Test".to_string(), DEFAULT_PASSING_GRADE);
    let normal_cat = Category::with_evaluations(
        "Normal".to_string(),
        60.0,
        vec![Evaluation::with_grade("N1".to_string(), 70.0)],
    );
    let weighted_rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    let weighted_cat = Category::with_rules(
        "Weighted".to_string(),
        40.0,
        vec![
            Evaluation::with_weight("A".to_string(), 80.0, 50.0),
            Evaluation::with_weight("B".to_string(), 0.0, 50.0),
        ],
        weighted_rules,
    );
    course.categories = vec![normal_cat, weighted_cat];

    let needed = course.needed_grade_for_evaluation(1, 1, true);
    assert_eq!(needed.status, NeededGradeStatus::Success);
}

#[test]
fn test_weighted_evals_round_before_weighting() {
    // Weighted evals with round_before_weighting
    // avg = 0.5 * 73 + 0.5 * 78 = 36.5 + 39.0 = 75.5
    // Rounded: 76
    let rules = CategoryRules {
        weighted_evaluations: true,
        round_before_weighting: true,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 73.0, 50.0),
        Evaluation::with_weight("B".to_string(), 78.0, 50.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    let avg = cat.average_grade().unwrap();
    assert!(
        (avg - 76.0).abs() < 0.01,
        "Expected 76.0 (rounded), got {avg}"
    );
}

#[test]
fn test_weighted_evals_geometric_mean() {
    // Weighted geometric mean: exp(w1*ln(g1) + w2*ln(g2))
    // Weights: 50%, 50%. Grades: 64, 100.
    // exp(0.5*ln(64) + 0.5*ln(100)) = exp(0.5*4.158 + 0.5*4.605)
    //   = exp(2.079 + 2.303) = exp(4.382) = 80.0
    let rules = CategoryRules {
        weighted_evaluations: true,
        averaging_method: AveragingMethod::Geometric,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 64.0, 50.0),
        Evaluation::with_weight("B".to_string(), 100.0, 50.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    let avg = cat.average_grade().unwrap();
    assert!((avg - 80.0).abs() < 0.1, "Expected ~80.0, got {avg}");
}

#[test]
fn test_weighted_evals_geometric_mean_with_zero() {
    // Weighted geometric mean: any zero grade → result is 0
    let rules = CategoryRules {
        weighted_evaluations: true,
        averaging_method: AveragingMethod::Geometric,
        ..Default::default()
    };
    let evals = vec![
        Evaluation::with_weight("A".to_string(), 0.0, 50.0),
        Evaluation::with_weight("B".to_string(), 100.0, 50.0),
    ];
    let cat = Category::with_rules("Cat".to_string(), 100.0, evals, rules);
    let avg = cat.average_grade().unwrap();
    assert!((avg - 0.0).abs() < 0.01, "Expected 0.0, got {avg}");
}

#[test]
fn test_weighted_evals_is_default_false() {
    let rules = CategoryRules {
        weighted_evaluations: true,
        ..Default::default()
    };
    assert!(!rules.is_default());
}

#[test]
fn test_weighted_evals_serde_default() {
    // Deserializing old data without weighted_evaluations should default to false
    let json = r#"{"drop_lowest":0,"averaging_method":"Arithmetic"}"#;
    let rules: CategoryRules = serde_json::from_str(json).unwrap();
    assert!(!rules.weighted_evaluations);
}

#[test]
fn test_eval_weight_serde_default() {
    // Deserializing old eval data without weight should default to None
    let json = r#"{"id":"00000000-0000-0000-0000-000000000000","name":"Test","grade":80.0}"#;
    let eval: Evaluation = serde_json::from_str(json).unwrap();
    assert!(eval.weight.is_none());
}

// =============================================================================
// Course outcome
// =============================================================================

/// Build a single-category course whose average is exactly `grade`.
fn course_with_grade(name: &str, grade: f64) -> Course {
    let mut course = Course::new(name.to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Cat".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("E1".to_string(), grade));
    course.categories.push(cat);
    course
}

#[test]
fn test_outcome_no_data_without_evaluations() {
    let course = Course::new("Empty".to_string(), DEFAULT_PASSING_GRADE);
    assert_eq!(course.outcome(), CourseOutcome::NoData);
    assert!(course.final_grade().is_none());
}

#[test]
fn test_outcome_passing_and_failing() {
    assert_eq!(
        course_with_grade("Ok", 80.0).outcome(),
        CourseOutcome::Passing
    );
    assert_eq!(
        course_with_grade("Bad", 30.0).outcome(),
        CourseOutcome::Failing
    );
}

#[test]
fn test_outcome_passing_at_rounding_boundary() {
    // 54.5 rounds to 55 and must count as passing.
    assert_eq!(
        course_with_grade("Edge", 54.5).outcome(),
        CourseOutcome::Passing
    );
}

#[test]
fn test_outcome_failing_when_minimum_caps_the_grade() {
    let mut course = Course::new("Capped".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Cat".to_string(), 100.0);
    cat.rules.minimum_average = Some(60.0);
    cat.rules.on_minimum_not_met = MinimumNotMetAction::FinalEqualsAverage;
    cat.evaluations
        .push(Evaluation::with_grade("E1".to_string(), 50.0));
    course.categories.push(cat);

    assert_eq!(course.outcome(), CourseOutcome::Failing);
}

// =============================================================================
// Semester metrics
// =============================================================================

fn semester_with(courses: Vec<Course>) -> Semester {
    let mut s = Semester::new("2026-1".to_string(), 0);
    s.courses = courses;
    s
}

#[test]
fn test_semester_average_is_arithmetic_without_credits() {
    let s = semester_with(vec![
        course_with_grade("A", 40.0),
        course_with_grade("B", 80.0),
    ]);
    let m = s.metrics();
    assert!(!m.weighted);
    assert!((m.average.unwrap() - 60.0).abs() < 0.01);
    assert!(m.total_credits.is_none());
}

#[test]
fn test_semester_average_is_credit_weighted_when_all_declare_credits() {
    let mut heavy = course_with_grade("Heavy", 40.0);
    heavy.credits = Some(10);
    let mut light = course_with_grade("Light", 90.0);
    light.credits = Some(3);

    let m = semester_with(vec![heavy, light]).metrics();
    assert!(m.weighted);
    // (40*10 + 90*3) / 13 = 51.53...
    assert!(
        (m.average.unwrap() - 51.538).abs() < 0.01,
        "got {:?}",
        m.average
    );
    assert_eq!(m.total_credits, Some(13));
}

#[test]
fn test_semester_average_falls_back_when_credits_are_partial() {
    // A partial set of credits must not silently weight the average.
    let mut a = course_with_grade("A", 40.0);
    a.credits = Some(10);
    let b = course_with_grade("B", 80.0);

    let m = semester_with(vec![a, b]).metrics();
    assert!(!m.weighted);
    assert!((m.average.unwrap() - 60.0).abs() < 0.01);
}

#[test]
fn test_semester_counts_and_credits_at_risk() {
    let mut ok = course_with_grade("Ok", 80.0);
    ok.credits = Some(5);
    let mut bad = course_with_grade("Bad", 20.0);
    bad.credits = Some(8);
    let empty = Course::new("Empty".to_string(), DEFAULT_PASSING_GRADE);

    let m = semester_with(vec![ok, bad, empty]).metrics();
    assert_eq!(m.counts.passing, 1);
    assert_eq!(m.counts.failing, 1);
    assert_eq!(m.counts.no_data, 1);
    assert_eq!(m.total_courses, 3);
    assert_eq!(m.credits_at_risk, Some(8));
}

#[test]
fn test_semester_critical_is_the_worst_unsecured_course() {
    let m = semester_with(vec![
        course_with_grade("Fine", 90.0),
        course_with_grade("Bad", 40.0),
        course_with_grade("Worst", 10.0),
    ])
    .metrics();
    assert_eq!(m.critical, Some(2));
}

#[test]
fn test_semester_metrics_on_empty_semester() {
    let m = semester_with(vec![]).metrics();
    assert!(m.average.is_none());
    assert!(m.critical.is_none());
    assert_eq!(m.total_courses, 0);
    assert!(m.credits_at_risk.is_none());
}

#[test]
fn test_cumulative_average_spans_semesters() {
    let s1 = semester_with(vec![course_with_grade("A", 40.0)]);
    let s2 = semester_with(vec![course_with_grade("B", 80.0)]);
    let avg = cumulative_average(&[s1, s2]).unwrap();
    assert!((avg - 60.0).abs() < 0.01);
}

#[test]
fn test_course_credits_serde_default() {
    // Course data written before credits existed must still load.
    let json = r#"{"id":"00000000-0000-0000-0000-000000000000","name":"Old",
        "passing_grade":55.0,"categories":[]}"#;
    let course: Course = serde_json::from_str(json).unwrap();
    assert!(course.credits.is_none());
}

#[test]
fn test_semester_counts_evaluation_progress() {
    let mut course = Course::new("Math".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Cat".to_string(), 100.0);
    cat.evaluations
        .push(Evaluation::with_grade("E1".to_string(), 70.0));
    cat.evaluations.push(Evaluation::new("E2".to_string()));
    course.categories.push(cat);

    let m = semester_with(vec![course]).metrics();
    assert_eq!(m.graded_evaluations, 1);
    assert_eq!(m.total_evaluations, 2);
}

// =============================================================================
// Outlook: ceiling, margin, pending weight
// =============================================================================

/// Course with one category of `total` evaluations, the first `graded` of
/// them scored at `grade`.
fn course_partially_graded(total: usize, graded: usize, grade: f64) -> Course {
    let mut course = Course::new("C".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Cat".to_string(), 100.0);
    for i in 0..total {
        if i < graded {
            cat.evaluations
                .push(Evaluation::with_grade(format!("E{i}"), grade));
        } else {
            cat.evaluations.push(Evaluation::new(format!("E{i}")));
        }
    }
    course.categories.push(cat);
    course
}

#[test]
fn test_best_case_fills_ungraded_with_full_marks() {
    // Two of four graded at 40: floor is 20, ceiling is 70.
    let course = course_partially_graded(4, 2, 40.0);
    assert!((course.final_grade().unwrap() - 20.0).abs() < 0.01);
    assert!((course.best_case_grade().unwrap() - 70.0).abs() < 0.01);
}

#[test]
fn test_best_case_equals_current_grade_when_everything_is_graded() {
    let course = course_partially_graded(3, 3, 60.0);
    let current = course.final_grade().unwrap();
    assert!((course.best_case_grade().unwrap() - current).abs() < 0.01);
}

#[test]
fn test_best_case_is_none_without_evaluations() {
    let course = Course::new("Empty".to_string(), DEFAULT_PASSING_GRADE);
    assert!(course.best_case_grade().is_none());
}

#[test]
fn test_unrecoverable_when_full_marks_still_fall_short() {
    // Three of four zeroed: the ceiling is 25, below the 55 needed.
    let course = course_partially_graded(4, 3, 0.0);
    assert!(course.is_unrecoverable());

    // One of four zeroed: the ceiling is 75, still winnable.
    let winnable = course_partially_graded(4, 1, 0.0);
    assert!(!winnable.is_unrecoverable());
}

#[test]
fn test_unrecoverable_accounts_for_the_global_exam() {
    // Semester grade is stuck at 40, but a 30%-weighted global can lift it.
    let mut course = course_partially_graded(2, 2, 40.0);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    // 40 * 0.7 + 100 * 0.3 = 58, which passes.
    assert!(!course.is_unrecoverable());
}

#[test]
fn test_margin_is_signed_distance_from_passing() {
    let passing = course_partially_graded(1, 1, 72.0);
    assert!((passing.margin().unwrap() - 17.0).abs() < 0.01);

    let failing = course_partially_graded(1, 1, 41.0);
    assert!((failing.margin().unwrap() + 14.0).abs() < 0.01);
}

#[test]
fn test_pending_weight_tracks_what_is_still_in_play() {
    let untouched = course_partially_graded(4, 0, 0.0);
    assert!((untouched.pending_weight() - 100.0).abs() < 0.01);

    let half = course_partially_graded(4, 2, 50.0);
    assert!((half.pending_weight() - 50.0).abs() < 0.01);

    let done = course_partially_graded(4, 4, 50.0);
    assert!(done.pending_weight().abs() < 0.01);
}

#[test]
fn test_pending_weight_uses_eval_weights_when_the_category_does() {
    let mut course = Course::new("C".to_string(), DEFAULT_PASSING_GRADE);
    let mut cat = Category::new("Cat".to_string(), 100.0);
    cat.rules.weighted_evaluations = true;
    cat.evaluations
        .push(Evaluation::with_weight("Done".to_string(), 60.0, 80.0));
    let mut pending = Evaluation::new("Pending".to_string());
    pending.weight = Some(20.0);
    cat.evaluations.push(pending);
    course.categories.push(cat);

    // The ungraded evaluation carries 20 of the 100 weight in the category.
    assert!((course.pending_weight() - 20.0).abs() < 0.01);
}

#[test]
fn test_empty_category_counts_as_fully_pending() {
    let mut course = Course::new("C".to_string(), DEFAULT_PASSING_GRADE);
    course
        .categories
        .push(Category::new("Nothing yet".to_string(), 40.0));
    let mut graded = Category::new("Done".to_string(), 60.0);
    graded
        .evaluations
        .push(Evaluation::with_grade("E".to_string(), 70.0));
    course.categories.push(graded);

    assert!((course.pending_weight() - 40.0).abs() < 0.01);
}

#[test]
fn test_next_ungraded_finds_the_first_gap() {
    let course = course_partially_graded(3, 1, 50.0);
    assert_eq!(course.next_ungraded(), Some((0, 1)));

    let complete = course_partially_graded(3, 3, 50.0);
    assert!(complete.next_ungraded().is_none());
}

#[test]
fn test_semester_reports_ceiling_and_unrecoverable_courses() {
    let m = semester_with(vec![
        course_partially_graded(4, 2, 40.0), // floor 20, ceiling 70
        course_partially_graded(4, 3, 0.0),  // ceiling 25 — lost
    ])
    .metrics();

    assert_eq!(m.unrecoverable, 1);
    // Ceilings 70 and 25 average to 47.5.
    assert!((m.best_case.unwrap() - 47.5).abs() < 0.01);
}

#[test]
fn test_semester_pending_weight_averages_across_courses() {
    let m = semester_with(vec![
        course_partially_graded(4, 2, 50.0), // 50% pending
        course_partially_graded(4, 4, 50.0), // 0% pending
    ])
    .metrics();
    assert!((m.pending_weight.unwrap() - 25.0).abs() < 0.01);
}

// =============================================================================
// Conditional caps, attendance and prerequisites
// =============================================================================

/// LabCom: tests averaging 50 or less cap the whole course at 54.
fn course_with_cap(controls_grade: f64) -> Course {
    let mut course = Course::new("LabCom".to_string(), DEFAULT_PASSING_GRADE);

    let mut controls = Category::new("Controles".to_string(), 40.0);
    controls.rules.minimum_average = Some(51.0);
    controls.rules.on_minimum_not_met = MinimumNotMetAction::CapFinalGrade;
    controls.rules.cap_final_grade = Some(54.0);
    controls
        .evaluations
        .push(Evaluation::with_grade("Q1".to_string(), controls_grade));

    let mut reports = Category::new("Informes".to_string(), 60.0);
    reports
        .evaluations
        .push(Evaluation::with_grade("I1".to_string(), 90.0));

    course.categories.push(controls);
    course.categories.push(reports);
    course
}

#[test]
fn test_cap_applies_when_the_minimum_is_missed() {
    // 50*0.4 + 90*0.6 = 74, but the cap pulls it down to 54.
    let capped = course_with_cap(50.0);
    assert!((capped.final_grade().unwrap() - 54.0).abs() < 0.01);
    assert_eq!(capped.outcome(), CourseOutcome::Failing);
}

#[test]
fn test_cap_leaves_the_grade_alone_when_the_minimum_is_met() {
    // 60*0.4 + 90*0.6 = 78, no cap.
    let fine = course_with_cap(60.0);
    assert!((fine.final_grade().unwrap() - 78.0).abs() < 0.01);
    assert_eq!(fine.outcome(), CourseOutcome::Passing);
}

#[test]
fn test_a_capped_course_is_unrecoverable() {
    // Even acing everything left cannot beat a 54 ceiling with 55 to pass.
    assert!(course_with_cap(50.0).is_unrecoverable());
}

#[test]
fn test_needed_grade_does_not_promise_a_rescue_that_a_cap_forbids() {
    let mut course = course_with_cap(50.0);
    course
        .categories
        .get_mut(1)
        .unwrap()
        .evaluations
        .push(Evaluation::new("I2".to_string()));

    // The only ungraded evaluation cannot lift a course capped below passing.
    let needed = course.needed_grade_for_evaluation(1, 1, true);
    assert_eq!(
        needed.status,
        NeededGradeStatus::Failure,
        "got {needed:?} — the cap makes passing impossible"
    );
}

#[test]
fn test_a_cap_above_passing_does_not_block_anything() {
    let mut course = course_with_cap(50.0);
    course.categories[0].rules.cap_final_grade = Some(80.0);
    assert!(!course.is_unrecoverable());
    assert_eq!(course.outcome(), CourseOutcome::Passing);
}

#[test]
fn test_attendance_percentage_and_margin() {
    let attendance = Attendance {
        total_classes: Some(28),
        missed: 6,
        required_percent: Some(85.0),
        action: AttendanceAction::FailCourse,
    };
    // 22 of 28 attended.
    assert!((attendance.percent().unwrap() - 78.571).abs() < 0.01);
    assert!(attendance.is_below_requirement());
    // 28 * 15% = 4.2, floored to 4 allowed misses; 6 already used.
    assert_eq!(attendance.misses_remaining(), Some(0));
}

#[test]
fn test_attendance_can_fail_an_otherwise_passing_course() {
    let mut course = course_partially_graded(1, 1, 90.0);
    course.attendance = Attendance {
        total_classes: Some(10),
        missed: 5,
        required_percent: Some(85.0),
        action: AttendanceAction::FailCourse,
    };

    assert_eq!(course.outcome(), CourseOutcome::Failing);
    assert!(course.final_grade().unwrap() < 1.0);
}

#[test]
fn test_attendance_set_to_warn_only_leaves_the_grade_alone() {
    let mut course = course_partially_graded(1, 1, 90.0);
    course.attendance = Attendance {
        total_classes: Some(10),
        missed: 5,
        required_percent: Some(85.0),
        action: AttendanceAction::WarnOnly,
    };

    assert!(course.attendance.is_below_requirement());
    assert_eq!(course.outcome(), CourseOutcome::Passing);
}

#[test]
fn test_attendance_is_inert_without_a_requirement() {
    let attendance = Attendance {
        total_classes: Some(10),
        missed: 9,
        required_percent: None,
        action: AttendanceAction::FailCourse,
    };
    assert!(!attendance.is_below_requirement());
    assert!(!attendance.fails_course());
}

#[test]
fn test_global_outcome_caps_the_final_grade() {
    // Semester 40, global 100 at 30% weight would give 58, but passing the
    // recovery exam caps the course at 55.
    let mut course = course_partially_graded(1, 1, 40.0);
    course.global_policy = GlobalExamPolicy::Weighted {
        semester_weight: 0.7,
        global_weight: 0.3,
    };
    course.global_outcome = GlobalOutcome {
        cap_if_passed: Some(55.0),
        cap_if_failed: Some(54.0),
    };
    course.global_exam_grade = Some(100.0);

    assert!((course.final_grade().unwrap() - 55.0).abs() < 0.01);
}

#[test]
fn test_toggle_fields_have_no_text_buffer() {
    // The crash this guards: a form routed a toggle field to the text path,
    // which typed into whatever buffer the getter happened to return.
    use crate::app::{App, InputField};

    let mut app = App::default();
    for field in [
        InputField::DropLowest,
        InputField::AvgMethod,
        InputField::OnMinNotMet,
        InputField::OnMinPerEvalNotMet,
        InputField::OnMinOneEvalNotMet,
        InputField::RoundBeforeWeight,
        InputField::WeightedEvals,
        InputField::AttendanceAction,
        InputField::GlobalPolicy,
    ] {
        assert!(field.is_toggle(), "{field:?} should be a toggle");
        app.input_field = field;
        assert!(
            app.current_input_buffer().is_none(),
            "{field:?} handed out a text buffer"
        );
    }
}

#[test]
fn test_every_typed_field_has_a_buffer() {
    use crate::app::{App, InputField};

    let mut app = App::default();
    for field in [
        InputField::Name,
        InputField::PassingGrade,
        InputField::Credits,
        InputField::Weight,
        InputField::Grade,
        InputField::Description,
        InputField::EvalWeight,
        InputField::MinimumAverage,
        InputField::MinPerEval,
        InputField::MinOneEval,
        InputField::ClassesTotal,
        InputField::ClassesMissed,
        InputField::AttendanceRequired,
        InputField::GlobalSemesterWeight,
        InputField::GlobalExamWeight,
        InputField::GlobalMinGrade,
    ] {
        assert!(!field.is_toggle(), "{field:?} should not be a toggle");
        app.input_field = field;
        assert!(
            app.current_input_buffer().is_some(),
            "{field:?} has no buffer to type into"
        );
    }
}

#[test]
fn test_margin_is_silent_until_something_is_graded() {
    // A fresh course would otherwise report -55 on day one, which is true and
    // useless: every course in every new semester says the same thing.
    let untouched = course_partially_graded(4, 0, 0.0);
    assert!(untouched.margin().is_none());
    assert!(!untouched.has_any_grade());

    // The ceiling still has something to say: everything is still reachable.
    assert!((untouched.best_case_grade().unwrap() - 100.0).abs() < 0.01);

    let started = course_partially_graded(4, 1, 40.0);
    assert!(started.has_any_grade());
    // 40/4 = 10 → 10 - 55.
    assert!((started.margin().unwrap() + 45.0).abs() < 0.01);
}
