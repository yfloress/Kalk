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

//! Dashboard metrics and their responsive presentation on Home.

use super::helpers::focused_border_style;
use super::home::draw_wordmark;
use super::theme::{Theme, theme};
use crate::app::App;
use crate::model::{
    Course, CourseOutcome, MAX_GRADE, NeededGradeStatus, SemesterMetrics, cumulative_average,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline, SparklineBar},
};

/// Width of the label column in the metrics pane.
const LABEL_W: usize = 14;

const BAR_FULL: &str = "\u{2588}";
const BAR_EMPTY: &str = "\u{2591}";
const ACCENT: &str = "\u{258E} ";

enum SummaryRow {
    Text(Line<'static>),
    Gauge {
        label: &'static str,
        value: usize,
        total: usize,
        width: usize,
        color: Color,
        caption: String,
    },
    Trend {
        label: &'static str,
        data: Vec<SparklineBar>,
        selected: usize,
        caption: String,
    },
}

impl From<Line<'static>> for SummaryRow {
    fn from(line: Line<'static>) -> Self {
        Self::Text(line)
    }
}

impl SummaryRow {
    fn render(self, frame: &mut Frame, area: Rect) {
        let t = theme();
        match self {
            Self::Text(line) => frame.render_widget(line, area),
            Self::Gauge {
                label,
                value,
                total,
                width,
                color,
                caption,
            } => {
                let columns = metric_columns(area, width, &caption);
                frame.render_widget(
                    Line::from(label).style(Style::default().fg(t.text_secondary)),
                    columns[0],
                );
                let ratio = value.min(total) as f64 / total.max(1) as f64;
                frame.render_widget(
                    Gauge::default()
                        .ratio(ratio)
                        .label("")
                        .use_unicode(true)
                        .gauge_style(Style::default().fg(color).bg(t.zebra_bg)),
                    columns[1],
                );
                frame.render_widget(
                    Line::from(caption).style(Style::default().fg(t.text_secondary)),
                    columns[2],
                );
            }
            Self::Trend {
                label,
                data,
                selected,
                caption,
            } => {
                let columns = metric_columns(area, data.len(), &caption);
                frame.render_widget(
                    Line::from(label).style(Style::default().fg(t.text_secondary)),
                    columns[0],
                );
                let start = selected.saturating_sub(columns[1].width.saturating_sub(1) as usize);
                frame.render_widget(
                    Sparkline::default()
                        .data(data.into_iter().skip(start))
                        .max((MAX_GRADE * 10.0) as u64)
                        .style(Style::default().fg(t.text_secondary))
                        .absent_value_symbol("·")
                        .absent_value_style(Style::default().fg(t.text_muted)),
                    columns[1],
                );
                frame.render_widget(
                    Line::from(caption).style(Style::default().fg(t.text_secondary)),
                    columns[2],
                );
            }
        }
    }
}

/// Keep the numeric caption visible before spending columns on decoration.
fn metric_columns(area: Rect, graph_width: usize, caption: &str) -> [Rect; 3] {
    let caption_width = Line::from(caption).width().min(u16::MAX as usize) as u16;
    let label_width = (LABEL_W as u16).min(area.width.saturating_sub(caption_width));
    let graph_width =
        graph_width.min(area.width.saturating_sub(label_width + caption_width) as usize) as u16;
    let columns = Layout::horizontal([
        Constraint::Length(label_width),
        Constraint::Length(graph_width),
        Constraint::Min(0),
    ])
    .split(area);
    [columns[0], columns[1], columns[2]]
}

pub(super) fn draw_metrics(frame: &mut Frame, app: &App, area: Rect) {
    let t = theme();

    let semester = app.current_semester();
    let metrics = semester.metrics();

    let block = Block::default()
        .title(format!(" {} ", semester.name))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(focused_border_style(false));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if metrics.total_courses == 0 {
        draw_wordmark(frame, app, inner);
        return;
    }

    // The summary takes what it needs and no more, and yields its optional
    // rows before the course list underneath loses room.
    let width = inner.width.saturating_sub(4);
    let budget = inner.height.saturating_sub(if app.courses().is_empty() {
        0
    } else {
        BREAKDOWN_MIN
    });
    let summary = fit_summary(summary_lines(app, &metrics, width), budget);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(2)
        .constraints([Constraint::Length(summary.len() as u16), Constraint::Min(0)])
        .split(inner);

    for (i, row) in summary.into_iter().enumerate() {
        row.render(
            frame,
            Rect::new(rows[0].x, rows[0].y + i as u16, rows[0].width, 1),
        );
    }
    draw_course_breakdown(frame, app, rows[1]);
}

/// How readily a summary row gives up its space when the pane is short.
/// Lower survives longer; `KEEP` rows are the reason the pane exists.
const KEEP: u8 = 0;
const USEFUL: u8 = 1;
const EXTRA: u8 = 2;
const NICETY: u8 = 3;

/// Rows the per-course breakdown needs before it is worth drawing at all:
/// its heading plus two courses.
const BREAKDOWN_MIN: u16 = 3;

/// The headline block: averages, outcome mix, progress, credits, trend.
/// Each row carries a priority so a short pane can shed the optional ones
/// instead of starving the course list underneath.
fn summary_lines(app: &App, metrics: &SemesterMetrics, width: u16) -> Vec<(u8, SummaryRow)> {
    let m = app.messages();
    let t = theme();
    let counts = &metrics.counts;

    // Width left for a bar after the label column and the trailing caption.
    let bar_w = width.saturating_sub(LABEL_W as u16 + 22).clamp(8, 32) as usize;

    let mut lines = Vec::new();

    // Ungraded evaluations count as zero, so `average` is the floor. Pairing
    // it with the ceiling turns a bare number into the range still in reach.
    lines.push((KEEP, average_row(m, t, metrics)));

    if let Some(avg) = cumulative_average(&app.semesters) {
        lines.push((
            NICETY,
            metric_row(m.metric_cumulative, format!("{:.1}", avg), t.text_secondary),
        ));
    }
    lines.push((KEEP, Line::from("").into()));

    // Outcome mix as one stacked bar, each segment in its status colour.
    let mut spans = vec![Span::styled(
        format!("{:<width$}", m.metric_status, width = LABEL_W),
        Style::default().fg(t.text_muted),
    )];
    let segments = [
        (counts.passing, t.status_pass),
        (counts.pending_global, t.status_override),
        (counts.failing, t.status_fail),
        (counts.no_data, t.text_muted),
    ];
    for (count, color) in segments {
        let cells = scale(count, metrics.total_courses, bar_w);
        if cells > 0 {
            spans.push(Span::styled(
                BAR_FULL.repeat(cells),
                Style::default().fg(color),
            ));
        }
    }
    spans.push(Span::styled(
        format!(
            "  {} {} · {} {}",
            counts.passing, m.metric_passed, counts.failing, m.metric_failing
        ),
        Style::default().fg(t.text_muted),
    ));
    lines.push((KEEP, Line::from(spans).into()));

    // How much of the semester has actually been graded.
    if metrics.total_evaluations > 0 {
        lines.push((
            EXTRA,
            gauge_row(
                m.metric_progress,
                metrics.graded_evaluations,
                metrics.total_evaluations,
                bar_w,
                t.status_info,
                format!(
                    "  {}/{} {}",
                    metrics.graded_evaluations, metrics.total_evaluations, m.metric_evaluations
                ),
            ),
        ));
    }

    if let Some(total) = metrics.total_credits {
        let at_risk = metrics.credits_at_risk.unwrap_or(0);
        lines.push((
            EXTRA,
            gauge_row(
                m.metric_credits,
                at_risk as usize,
                total as usize,
                bar_w,
                t.status_fail,
                format!("  {} {} / {}", at_risk, m.metric_credits_at_risk, total),
            ),
        ));
    }

    if let Some(pending) = metrics.pending_weight {
        lines.push((
            NICETY,
            metric_row(
                m.metric_in_play,
                format!("{:.0}%", pending),
                t.text_secondary,
            ),
        ));
    }

    lines.push((USEFUL, Line::from("").into()));

    if counts.pending_global > 0 {
        lines.push((
            USEFUL,
            metric_row(
                m.metric_global,
                format!("{} {}", counts.pending_global, m.metric_pending_word),
                t.status_override,
            ),
        ));
    }

    if metrics.failed_minimums > 0 {
        lines.push((
            USEFUL,
            metric_row(
                m.metric_minimums,
                format!("{} {}", metrics.failed_minimums, m.metric_unmet_word),
                t.status_override,
            ),
        ));
    }

    if metrics.unrecoverable > 0 {
        lines.push((
            USEFUL,
            metric_row(
                m.metric_unrecoverable,
                metrics.unrecoverable.to_string(),
                t.status_fail,
            ),
        ));
    }

    // The single course furthest from safety, and what would fix it.
    if let Some(course) = metrics.critical.and_then(|i| app.courses().get(i)) {
        lines.push((
            USEFUL,
            metric_row(m.metric_critical, critical_detail(course, m), t.status_fail),
        ));
    }

    // Trend only says something once there is more than one semester.
    if app.semesters.len() > 1 {
        lines.push((NICETY, Line::from("").into()));
        lines.push((NICETY, trend_row(app, m.metric_trend)));
    }

    lines
}

/// The current grade and the best grade still reachable, as a range.
/// Collapses to a single figure once nothing is left to play for.
fn average_row(m: &crate::i18n::Messages, t: &Theme, metrics: &SemesterMetrics) -> SummaryRow {
    let Some(avg) = metrics.average else {
        return metric_row(m.metric_average, m.metric_no_data.to_string(), t.text_muted);
    };

    let mut spans = vec![
        Span::styled(
            format!("{:<width$}", m.metric_average, width = LABEL_W),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(
            format!("{:.1}", avg),
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(best) = metrics.best_case.filter(|b| *b - avg > 0.05) {
        spans.push(Span::styled(
            " \u{2192} ",
            Style::default().fg(t.text_muted),
        ));
        spans.push(Span::styled(
            format!("{:.1}", best),
            Style::default().fg(t.status_info),
        ));
    }

    if metrics.weighted {
        spans.push(Span::styled(
            format!("  ({})", m.metric_average_weighted),
            Style::default().fg(t.text_muted),
        ));
    }

    Line::from(spans).into()
}

/// "Physics (41) \u{b7} needs 62 in each pending one", or a plain note when it
/// is already lost.
fn critical_detail(course: &Course, m: &crate::i18n::Messages) -> String {
    let grade = match course.final_grade() {
        Some(g) => format!("{}  ({:.0})", course.name, Course::round_grade(g)),
        None => course.name.clone(),
    };

    if course.is_unrecoverable() {
        return format!("{}  \u{b7}  {}", grade, m.metric_lost);
    }

    // What everything still open has to score, spread evenly.
    let needed = course.needed_across_open(None);
    match needed.status {
        NeededGradeStatus::Warning => {
            let Some(value) = needed.value else {
                return grade;
            };
            let scope = match course.next_ungraded() {
                Some((ci, ei)) if needed.spread_over == 1 => course
                    .categories
                    .get(ci)
                    .and_then(|c| c.evaluations.get(ei))
                    .map(|e| format!("{} {}", m.metric_in, e.name))
                    .unwrap_or_default(),
                _ => m.metric_in_each_open.to_string(),
            };
            format!(
                "{}  \u{b7}  {} {:.0} {}",
                grade,
                m.metric_needs,
                value.ceil(),
                scope
            )
        }
        _ => grade,
    }
}

/// Drop the most expendable rows until the summary fits `budget`, keeping the
/// surviving rows in their original order and never leaving a dangling blank.
fn fit_summary(rows: Vec<(u8, SummaryRow)>, budget: u16) -> Vec<SummaryRow> {
    let budget = budget as usize;
    let mut kept = rows;

    for level in (KEEP + 1..=NICETY).rev() {
        if kept.len() <= budget {
            break;
        }
        kept.retain(|(priority, _)| *priority < level);
    }

    let mut lines: Vec<SummaryRow> = kept.into_iter().map(|(_, line)| line).collect();

    lines.dedup_by(|a, b| is_blank(a) && is_blank(b));
    lines.truncate(budget);

    // Trim last: truncating to the budget can re-expose a separator whose
    // group was cut off below it.
    while lines.last().is_some_and(is_blank) {
        lines.pop();
    }
    lines
}

fn is_blank(row: &SummaryRow) -> bool {
    matches!(row, SummaryRow::Text(line) if line.spans.iter().all(|s| s.content.trim().is_empty()))
}

/// A label, a partially filled bar, and a caption.
fn gauge_row(
    label: &'static str,
    value: usize,
    total: usize,
    bar_w: usize,
    color: ratatui::style::Color,
    caption: String,
) -> SummaryRow {
    SummaryRow::Gauge {
        label,
        value,
        total,
        width: bar_w,
        color,
        caption,
    }
}

/// One block glyph per semester, height proportional to its average.
fn trend_row(app: &App, label: &'static str) -> SummaryRow {
    let t = theme();

    let averages: Vec<Option<f64>> = app.semesters.iter().map(|s| s.metrics().average).collect();

    let data = averages
        .iter()
        .enumerate()
        .map(|(i, avg)| {
            let style = if i == app.selected_semester {
                Style::default()
                    .fg(t.status_info)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text_secondary)
            };
            SparklineBar::from(avg.map(|v| (v * 10.0) as u64)).style(style)
        })
        .collect();
    let caption = averages
        .get(app.selected_semester)
        .and_then(|v| *v)
        .map(|v| format!("  {v:.1}"))
        .unwrap_or_default();
    SummaryRow::Trend {
        label,
        data,
        selected: app.selected_semester,
        caption,
    }
}

/// Per-course rows: status accent, name, grade and a bar of the grade itself.
fn draw_course_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();

    if area.height < 2 {
        return;
    }

    let name_w = app
        .courses()
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(4)
        .clamp(4, 20);
    let bar_w = area.width.saturating_sub(name_w as u16 + 24).clamp(6, 24) as usize;

    let mut lines = vec![Line::from(Span::styled(
        m.metric_courses_breakdown,
        Style::default()
            .fg(t.text_muted)
            .add_modifier(Modifier::BOLD),
    ))];

    // One row for the header; if any course would be dropped, one more row
    // for the "and N more" line, so the count is always visible.
    let total = app.courses().len();
    let mut visible = area.height.saturating_sub(1) as usize;
    if total > visible {
        visible = visible.saturating_sub(1);
    }
    for course in app.courses().iter().take(visible) {
        let color = outcome_color(course.outcome());
        let grade = course.final_grade();
        let filled = grade.map_or(0, |g| scale(g.round() as usize, MAX_GRADE as usize, bar_w));

        lines.push(Line::from(vec![
            Span::styled(ACCENT, Style::default().fg(color)),
            Span::styled(
                format!("{:<width$}", truncate(&course.name, name_w), width = name_w),
                Style::default(),
            ),
            Span::styled(
                match grade {
                    Some(g) => format!(" {:>3.0} ", Course::round_grade(g)),
                    None => "   - ".to_string(),
                },
                Style::default().fg(color),
            ),
            // Signed distance from passing: the grade alone does not say
            // much when each course sets its own bar.
            Span::styled(
                match course.margin() {
                    Some(margin) => format!("{:>+4.0} ", margin),
                    None => "     ".to_string(),
                },
                Style::default().fg(t.text_muted),
            ),
            Span::styled(BAR_FULL.repeat(filled), Style::default().fg(color)),
            Span::styled(
                BAR_EMPTY.repeat(bar_w.saturating_sub(filled)),
                Style::default().fg(t.text_muted),
            ),
            Span::styled(
                if course.is_unrecoverable() {
                    format!("  {}", m.metric_lost)
                } else {
                    String::new()
                },
                Style::default().fg(t.status_fail),
            ),
        ]));
    }

    // The breakdown is clipped to fit: say how many courses are missing,
    // so a long semester does not silently hide rows.
    if total > visible {
        lines.push(Line::from(Span::styled(
            format!("\u{2026} {} {}", total - visible, m.metric_more),
            Style::default().fg(t.text_muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn outcome_color(outcome: CourseOutcome) -> ratatui::style::Color {
    let t = theme();
    match outcome {
        CourseOutcome::NoData => t.text_muted,
        CourseOutcome::Passing => t.status_pass,
        CourseOutcome::Failing => t.status_fail,
        CourseOutcome::PendingGlobal => t.status_override,
    }
}

/// `value/total` mapped onto `width` cells, rounding up so a non-zero value
/// never renders as nothing.
fn scale(value: usize, total: usize, width: usize) -> usize {
    if total == 0 || value == 0 {
        return 0;
    }
    ((value * width).div_ceil(total)).min(width)
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    out.push('\u{2026}');
    out
}

/// One "Label      value" row, label muted and value coloured.
fn metric_row(label: &str, value: String, color: ratatui::style::Color) -> SummaryRow {
    let t = theme();
    Line::from(vec![
        Span::styled(
            format!("{:<width$}", label, width = LABEL_W),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(value, Style::default().fg(color)),
    ])
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<(u8, SummaryRow)> {
        vec![
            (KEEP, Line::from("average")),
            (NICETY, Line::from("cumulative")),
            (KEEP, Line::from("")),
            (KEEP, Line::from("status")),
            (EXTRA, Line::from("progress")),
            (USEFUL, Line::from("")),
            (USEFUL, Line::from("critical")),
            (NICETY, Line::from("")),
            (NICETY, Line::from("trend")),
        ]
        .into_iter()
        .map(|(priority, line)| (priority, line.into()))
        .collect()
    }

    fn text(lines: &[SummaryRow]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|row| match row {
                SummaryRow::Text(line) => {
                    Some(line.spans.iter().map(|s| s.content.as_ref()).collect())
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn everything_survives_when_there_is_room() {
        let out = fit_summary(rows(), 20);
        assert_eq!(out.len(), 9);
    }

    #[test]
    fn niceties_go_first_and_order_is_preserved() {
        let out = fit_summary(rows(), 6);
        let got = text(&out);
        assert!(!got.contains(&"trend".to_string()), "{got:?}");
        assert!(!got.contains(&"cumulative".to_string()), "{got:?}");
        assert!(got.contains(&"critical".to_string()), "{got:?}");
        assert_eq!(got.first().unwrap(), "average");
    }

    #[test]
    fn the_headline_rows_are_the_last_to_go() {
        let got = text(&fit_summary(rows(), 3));
        assert!(got.contains(&"average".to_string()), "{got:?}");
        assert!(got.contains(&"status".to_string()), "{got:?}");
    }

    #[test]
    fn never_ends_on_a_dangling_separator() {
        for budget in 1..=12u16 {
            let out = fit_summary(rows(), budget);
            assert!(
                !out.last().is_some_and(is_blank),
                "budget {budget} left a trailing blank"
            );
        }
    }

    #[test]
    fn never_exceeds_the_budget() {
        for budget in 0..=12u16 {
            assert!(fit_summary(rows(), budget).len() <= budget as usize);
        }
    }
}
