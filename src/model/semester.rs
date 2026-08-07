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

//! Semesters group courses in time, and carry the aggregate metrics computed
//! over them. Past semesters are kept rather than deleted, which is what makes
//! a cumulative average possible.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Course, CourseOutcome};

/// A group of courses taken in the same period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub id: Uuid,
    pub name: String,
    /// Display order, lowest first. Separate from the name so renaming never
    /// reshuffles the list.
    pub order: u32,
    pub courses: Vec<Course>,
}

impl Semester {
    pub fn new(name: String, order: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            order,
            courses: Vec::new(),
        }
    }

    /// Total declared credits, `None` when no course declares any.
    pub fn total_credits(&self) -> Option<u32> {
        let sum: u32 = self.courses.iter().filter_map(|c| c.credits).sum();
        if self.courses.iter().any(|c| c.credits.is_some()) {
            Some(sum)
        } else {
            None
        }
    }

    /// Compute the aggregate metrics shown on the dashboard.
    pub fn metrics(&self) -> SemesterMetrics {
        let mut counts = OutcomeCounts::default();
        let mut credits_at_risk: u32 = 0;
        let mut any_credits = false;
        let mut unrecoverable = 0usize;
        let mut failed_minimums = 0usize;
        let mut best_cases: Vec<(&Course, f64)> = Vec::new();
        let mut pending_weights: Vec<f64> = Vec::new();

        // Courses that carry a grade — the basis for averages.
        let mut graded: Vec<(&Course, f64)> = Vec::new();

        for course in &self.courses {
            let outcome = course.outcome();
            match outcome {
                CourseOutcome::NoData => counts.no_data += 1,
                CourseOutcome::Passing => counts.passing += 1,
                CourseOutcome::Failing => counts.failing += 1,
                CourseOutcome::PendingGlobal => counts.pending_global += 1,
            }

            if course.credits.is_some() {
                any_credits = true;
            }

            if matches!(
                outcome,
                CourseOutcome::Failing | CourseOutcome::PendingGlobal
            ) {
                credits_at_risk += course.credits.unwrap_or(0);
            }

            if let Some(grade) = course.final_grade() {
                graded.push((course, grade));
            }
            if let Some(best) = course.best_case_grade() {
                best_cases.push((course, best));
            }
            if course.is_unrecoverable() {
                unrecoverable += 1;
            }
            failed_minimums += course.compute_grade().failed_minimums.len();
            if !course.categories.is_empty() {
                pending_weights.push(course.pending_weight());
            }
        }

        // A partial set of credits would silently distort the average, so
        // weight only when every graded course declares them.
        let weighted = !graded.is_empty() && graded.iter().all(|(c, _)| c.credits.is_some());

        let average = mean(&graded, weighted);

        // Lowest grade among the courses not yet secured.
        let critical = self
            .courses
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                matches!(
                    c.outcome(),
                    CourseOutcome::Failing | CourseOutcome::PendingGlobal
                )
            })
            .filter_map(|(i, c)| c.final_grade().map(|g| (i, g)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);

        let graded_evaluations: usize = self
            .courses
            .iter()
            .flat_map(|c| &c.categories)
            .map(|cat| cat.graded_count())
            .sum();
        let total_evaluations: usize = self
            .courses
            .iter()
            .flat_map(|c| &c.categories)
            .map(|cat| cat.evaluations.len())
            .sum();

        SemesterMetrics {
            average,
            best_case: mean(&best_cases, weighted),
            weighted,
            counts,
            unrecoverable,
            failed_minimums,
            pending_weight: if pending_weights.is_empty() {
                None
            } else {
                Some(pending_weights.iter().sum::<f64>() / pending_weights.len() as f64)
            },
            graded_evaluations,
            total_evaluations,
            total_courses: self.courses.len(),
            total_credits: self.total_credits(),
            credits_at_risk: any_credits.then_some(credits_at_risk),
            critical,
        }
    }
}

/// How many courses fall into each [`CourseOutcome`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutcomeCounts {
    pub passing: usize,
    pub failing: usize,
    pub pending_global: usize,
    pub no_data: usize,
}

/// Aggregate view of one semester, for the dashboard.
#[derive(Debug, Clone)]
pub struct SemesterMetrics {
    /// Average across courses that have a grade. This is the floor: ungraded
    /// evaluations count as zero.
    pub average: Option<f64>,
    /// The same average if every remaining evaluation were aced — the ceiling.
    pub best_case: Option<f64>,
    /// True when `average` is credit-weighted rather than a plain mean.
    pub weighted: bool,
    pub counts: OutcomeCounts,
    /// Courses that can no longer reach their passing grade.
    pub unrecoverable: usize,
    /// Categories sitting below a minimum they are required to meet.
    pub failed_minimums: usize,
    /// Mean share of each course's final grade that no evaluation has settled.
    /// Distinct from the evaluation count: a course can be most of the way
    /// through its evaluations while the heavy ones are still ahead.
    pub pending_weight: Option<f64>,
    /// Evaluations already graded, across every course.
    pub graded_evaluations: usize,
    pub total_evaluations: usize,
    pub total_courses: usize,
    pub total_credits: Option<u32>,
    /// Credits tied up in courses that are failing or pending a global.
    pub credits_at_risk: Option<u32>,
    /// Index into `Semester::courses` of the course in most trouble.
    pub critical: Option<usize>,
}

/// Cumulative average across semesters, weighted the same way one semester is.
pub fn cumulative_average(semesters: &[Semester]) -> Option<f64> {
    let graded: Vec<(&Course, f64)> = semesters
        .iter()
        .flat_map(|s| &s.courses)
        .filter_map(|c| c.final_grade().map(|g| (c, g)))
        .collect();

    if graded.is_empty() {
        return None;
    }

    let weighted = graded.iter().all(|(c, _)| c.credits.is_some());
    mean(&graded, weighted)
}

/// Mean of `(course, value)` pairs, weighted by credits when asked for.
/// Falls back to an unweighted mean if the credits add up to nothing.
fn mean(values: &[(&Course, f64)], weighted: bool) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    if weighted {
        let total: u32 = values.iter().map(|(c, _)| c.credits.unwrap_or(0)).sum();
        if total > 0 {
            let sum: f64 = values
                .iter()
                .map(|(c, v)| v * f64::from(c.credits.unwrap_or(0)))
                .sum();
            return Some(sum / f64::from(total));
        }
    }

    let sum: f64 = values.iter().map(|(_, v)| v).sum();
    Some(sum / values.len() as f64)
}
