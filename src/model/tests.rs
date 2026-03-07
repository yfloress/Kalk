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
    // Certamenes avg = 40 < 55 → needs global
    assert!(result.needs_global);
    assert!(result.overridden_by.is_none());
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
    // With drop lowest: the 20 gets dropped, so we need avg of (80 + X) / 2 >= 54.5
    // X >= 109 - 80 = 29
    assert_eq!(result.status, NeededGradeStatus::Warning);
    assert!(result.value.unwrap() < 35.0); // Should be around 29
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
