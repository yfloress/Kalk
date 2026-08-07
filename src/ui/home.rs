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

//! Home screen: the semester list and the dashboard metrics for the selected
//! one.  All figures come from `model::SemesterMetrics` — nothing is computed
//! here.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::app::App;
use crate::model::{
    Course, CourseOutcome, MAX_GRADE, NeededGradeStatus, SemesterMetrics, cumulative_average,
};

use super::helpers::{
    centered_rect, focused_border_style, render_delete_confirmation, render_input_field,
};
use super::icons::icons;
use super::keyhints::{render_hint, styled_keybindings};
use super::theme::{Theme, theme};

/// Wordmark shown when there is nothing to report yet.
const WORDMARK: &str = r#" ___  __    ________  ___       ___  __
|\  \|\  \ |\   __  \|\  \     |\  \|\  \
\ \  \/  /|\ \  \|\  \ \  \    \ \  \/  /|_
 \ \   ___  \ \   __  \ \  \    \ \   ___  \
  \ \  \\ \  \ \  \ \  \ \  \____\ \  \\ \  \
   \ \__\\ \__\ \__\ \__\ \_______\ \__\\ \__\
    \|__| \|__|\|__|\|__|\|_______|\|__| \|__|"#;

/// The wordmark as styled lines, ready to render left-aligned.
pub(super) fn wordmark_lines(color: ratatui::style::Color) -> Vec<Line<'static>> {
    WORDMARK
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(color))))
        .collect()
}

/// Rows the wordmark needs, plus breathing room above and below.
const WORDMARK_ROWS: u16 = 11;

/// Width of the label column in the metrics pane.
const LABEL_W: usize = 14;

const BAR_FULL: &str = "\u{2588}";
const BAR_EMPTY: &str = "\u{2591}";
const ACCENT: &str = "\u{258E} ";

pub fn draw_home(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(chunks[0]);

    draw_semester_list(frame, app, columns[0]);
    draw_metrics(frame, app, columns[1]);
    draw_home_footer(frame, app, chunks[1]);
}

fn draw_semester_list(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();
    let ic = icons(app.use_nerd_fonts);

    let items: Vec<ListItem> = app
        .semesters
        .iter()
        .map(|s| {
            let count = s.courses.len();
            Line::from(vec![
                Span::styled(&s.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}", count), Style::default().fg(t.text_muted)),
            ])
        })
        .map(ListItem::new)
        .collect();

    let title = format!(" {}{} ({}) ", ic.course, m.semesters, app.semesters.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(focused_border_style(true)),
        )
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(ic.highlight);

    let mut state = ListState::default();
    state.select(Some(app.selected_semester));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_metrics(frame: &mut Frame, app: &App, area: Rect) {
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

    frame.render_widget(Paragraph::new(summary), rows[0]);
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
fn summary_lines(app: &App, metrics: &SemesterMetrics, width: u16) -> Vec<(u8, Line<'static>)> {
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
    lines.push((KEEP, Line::from("")));

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
    lines.push((KEEP, Line::from(spans)));

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

    lines.push((USEFUL, Line::from("")));

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
        lines.push((NICETY, Line::from("")));
        lines.push((NICETY, trend_row(app, m.metric_trend)));
    }

    lines
}

/// The current grade and the best grade still reachable, as a range.
/// Collapses to a single figure once nothing is left to play for.
fn average_row(m: &crate::i18n::Messages, t: &Theme, metrics: &SemesterMetrics) -> Line<'static> {
    let Some(avg) = metrics.average else {
        return metric_row(m.metric_average, m.metric_no_data.to_string(), t.text_muted);
    };

    let mut spans = vec![
        Span::styled(
            format!("{:<width$}", m.metric_average, width = LABEL_W),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(format!("{:.1}", avg), Style::default().fg(t.text_primary)),
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

    Line::from(spans)
}

/// "Physics (41) \u{b7} needs 72 in C3", or a plain note when it is already lost.
fn critical_detail(course: &Course, m: &crate::i18n::Messages) -> String {
    let grade = match course.final_grade() {
        Some(g) => format!("{}  ({:.0})", course.name, Course::round_grade(g)),
        None => course.name.clone(),
    };

    if course.is_unrecoverable() {
        return format!("{}  \u{b7}  {}", grade, m.metric_lost);
    }

    // What the next untaken evaluation would have to score.
    let Some((ci, ei)) = course.next_ungraded() else {
        return grade;
    };
    let needed = course.needed_grade_for_evaluation(ci, ei, true);
    let (Some(value), Some(name)) = (
        needed.value,
        course
            .categories
            .get(ci)
            .and_then(|c| c.evaluations.get(ei))
            .map(|e| e.name.as_str()),
    ) else {
        return grade;
    };

    if matches!(needed.status, NeededGradeStatus::Failure) {
        return format!("{}  \u{b7}  {}", grade, m.metric_lost);
    }

    format!(
        "{}  \u{b7}  {} {:.0} {} {}",
        grade, m.metric_needs, value, m.metric_in, name
    )
}

/// Drop the most expendable rows until the summary fits `budget`, keeping the
/// surviving rows in their original order and never leaving a dangling blank.
fn fit_summary(rows: Vec<(u8, Line<'static>)>, budget: u16) -> Vec<Line<'static>> {
    let budget = budget as usize;
    let mut kept: Vec<(u8, Line<'static>)> = rows;

    for level in (KEEP + 1..=NICETY).rev() {
        if kept.len() <= budget {
            break;
        }
        kept.retain(|(priority, _)| *priority < level);
    }

    let mut lines: Vec<Line<'static>> = kept.into_iter().map(|(_, line)| line).collect();

    lines.dedup_by(|a, b| is_blank(a) && is_blank(b));
    lines.truncate(budget);

    // Trim last: truncating to the budget can re-expose a separator whose
    // group was cut off below it.
    while lines.last().is_some_and(is_blank) {
        lines.pop();
    }
    lines
}

fn is_blank(line: &Line<'static>) -> bool {
    line.spans.iter().all(|s| s.content.trim().is_empty())
}

/// A label, a partially filled bar, and a caption.
fn gauge_row(
    label: &str,
    value: usize,
    total: usize,
    bar_w: usize,
    color: ratatui::style::Color,
    caption: String,
) -> Line<'static> {
    let t = theme();
    let filled = scale(value, total, bar_w);

    Line::from(vec![
        Span::styled(
            format!("{:<width$}", label, width = LABEL_W),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(BAR_FULL.repeat(filled), Style::default().fg(color)),
        Span::styled(
            BAR_EMPTY.repeat(bar_w.saturating_sub(filled)),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(caption, Style::default().fg(t.text_muted)),
    ])
}

/// One block glyph per semester, height proportional to its average.
fn trend_row(app: &App, label: &str) -> Line<'static> {
    let t = theme();

    let averages: Vec<Option<f64>> = app.semesters.iter().map(|s| s.metrics().average).collect();

    let mut spans = vec![Span::styled(
        format!("{:<width$}", label, width = LABEL_W),
        Style::default().fg(t.text_muted),
    )];

    for (i, avg) in averages.iter().enumerate() {
        let (glyph, color) = match avg {
            Some(v) => (spark_glyph(*v), t.status_info),
            None => (BAR_EMPTY, t.text_muted),
        };
        let style = if i == app.selected_semester {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_secondary)
        };
        spans.push(Span::styled(glyph, style));
    }

    if let Some(Some(v)) = averages.get(app.selected_semester) {
        spans.push(Span::styled(
            format!("  {:.1}", v),
            Style::default().fg(t.text_muted),
        ));
    }

    Line::from(spans)
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

    for course in app
        .courses()
        .iter()
        .take(area.height.saturating_sub(1) as usize)
    {
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

/// Pick a block glyph for a 0-100 grade.
fn spark_glyph(grade: f64) -> &'static str {
    const GLYPHS: [&str; 8] = [
        "\u{2581}", "\u{2582}", "\u{2583}", "\u{2584}", "\u{2585}", "\u{2586}", "\u{2587}",
        "\u{2588}",
    ];
    let idx = ((grade / MAX_GRADE) * GLYPHS.len() as f64).floor() as usize;
    GLYPHS[idx.min(GLYPHS.len() - 1)]
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
fn metric_row(label: &str, value: String, color: ratatui::style::Color) -> Line<'static> {
    let t = theme();
    Line::from(vec![
        Span::styled(
            format!("{:<width$}", label, width = LABEL_W),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(value, Style::default().fg(color)),
    ])
}

fn draw_wordmark(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();

    let art_w = WORDMARK
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;

    // Too cramped for the art — the hint alone is more useful than a mangled
    // wordmark.
    if area.height < WORDMARK_ROWS || area.width < art_w {
        let hint = Paragraph::new(m.home_no_courses)
            .style(Style::default().fg(t.text_muted))
            .alignment(Alignment::Center);
        frame.render_widget(hint, area);
        return;
    }

    let art_h = WORDMARK.lines().count() as u16;
    let top = area.y + (area.height.saturating_sub(WORDMARK_ROWS)) / 2;

    // Centred as one block and drawn left-aligned inside it. Centring each
    // line individually would shift them apart and break the diagonals,
    // because the lines are not all the same width.
    let art_area = Rect {
        x: area.x + (area.width - art_w) / 2,
        y: top,
        width: art_w,
        height: art_h,
    };
    frame.render_widget(
        Paragraph::new(wordmark_lines(t.status_info)).alignment(Alignment::Left),
        art_area,
    );

    let hint_area = Rect {
        x: area.x,
        y: top + art_h + 1,
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(m.home_no_courses)
            .style(Style::default().fg(t.text_muted))
            .alignment(Alignment::Center),
        hint_area,
    );
}

fn draw_home_footer(frame: &mut Frame, app: &App, area: Rect) {
    let m = app.messages();
    let t = theme();

    // Only the essentials; renaming and reordering live in the `?` overlay.
    // `?` stays last so the width ladder never drops it.
    let keys = [
        ("j/k", m.help_move_updown),
        ("Enter", m.semester_open),
        ("n", m.semester_new),
        ("d", m.semester_delete),
        ("Esc", m.help_close_popup),
        ("?", m.help_open),
    ];

    let line = styled_keybindings(&keys, t, area.width.saturating_sub(2));

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(t.border_type)
            .border_style(Style::default().fg(t.footer_border)),
    );
    frame.render_widget(footer, area);
}

// =============================================================================
// Semester popups
// =============================================================================

/// Single-field form for creating or renaming a semester.
pub fn draw_semester_popup(frame: &mut Frame, app: &App, is_new: bool) {
    let m = app.messages();
    let t = theme();

    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let title = if is_new {
        m.semester_new
    } else {
        m.semester_rename
    };
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    render_input_field(frame, m.semester_name_label, &app.edit_name, true, rows[0]);

    render_hint(frame, &[("Enter", m.confirm), ("Esc", m.cancel)], rows[1]);
}

pub fn draw_delete_semester_popup(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let semester = app.current_semester();
    let count = semester.courses.len();
    let label = if count == 1 {
        m.course_singular
    } else {
        m.course_plural
    };

    let warning = format!("{} ({} {})", m.semester_delete_warning, count, label);

    render_delete_confirmation(
        frame,
        m.confirm_delete,
        m.semester_delete,
        &warning,
        m.confirm,
        m.cancel,
        app.use_nerd_fonts,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<(u8, Line<'static>)> {
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
    }

    fn text(lines: &[Line<'static>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
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
