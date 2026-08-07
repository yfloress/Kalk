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
        }

        // A partial set of credits would silently distort the average, so
        // weight only when every graded course declares them.
        let weighted = !graded.is_empty() && graded.iter().all(|(c, _)| c.credits.is_some());

        let average = if graded.is_empty() {
            None
        } else if weighted {
            let total: u32 = graded.iter().map(|(c, _)| c.credits.unwrap_or(0)).sum();
            if total == 0 {
                None
            } else {
                let sum: f64 = graded
                    .iter()
                    .map(|(c, g)| g * f64::from(c.credits.unwrap_or(0)))
                    .sum();
                Some(sum / f64::from(total))
            }
        } else {
            let sum: f64 = graded.iter().map(|(_, g)| g).sum();
            Some(sum / graded.len() as f64)
        };

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
            weighted,
            counts,
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
    /// Average across courses that have a grade.
    pub average: Option<f64>,
    /// True when `average` is credit-weighted rather than a plain mean.
    pub weighted: bool,
    pub counts: OutcomeCounts,
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

    if graded.iter().all(|(c, _)| c.credits.is_some()) {
        let total: u32 = graded.iter().map(|(c, _)| c.credits.unwrap_or(0)).sum();
        if total > 0 {
            let sum: f64 = graded
                .iter()
                .map(|(c, g)| g * f64::from(c.credits.unwrap_or(0)))
                .sum();
            return Some(sum / f64::from(total));
        }
    }

    let sum: f64 = graded.iter().map(|(_, g)| g).sum();
    Some(sum / graded.len() as f64)
}
