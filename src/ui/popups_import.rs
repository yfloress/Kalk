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

//! Three-step AI import wizard popups.
//!
//! The wizard is invoked when the synthetic "Create with AI" entry at index 0
//! of the template selector is confirmed:
//!
//! 1. [`draw_import_prompt`] — shows the prompt and copies it on `c`.
//! 2. [`draw_import_paste`]  — waits for a bracketed-paste event from the user.
//! 3. [`draw_import_preview`] — preview of the parsed course; `Enter` commits.

use super::helpers::centered_rect;
use super::keyhints::render_hint;
use super::theme::{Theme, theme};
use crate::app::App;
use crate::i18n::Messages;
use crate::model::{AveragingMethod, Category, Course, GlobalExamPolicy, MinimumNotMetAction};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

// =============================================================================
// Step 1 — Prompt
// =============================================================================

pub fn draw_import_prompt(frame: &mut Frame, app: &App) {
    // Full-screen variant: no borders, fills the terminal so the user can
    // mouse-select the prompt text and copy it using the terminal's own
    // shortcut.  Universal fallback for terminals without OSC 52 support.
    if app.import_prompt_fullscreen {
        draw_import_prompt_fullscreen(frame, app);
        return;
    }

    let m = app.messages();
    let t = theme();
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step1_title),
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // intro hint
            Constraint::Min(0),    // prompt body
            Constraint::Length(2), // feedback + footer
        ])
        .split(inner);

    let hint = Paragraph::new(m.import_step1_hint)
        .style(Style::default().fg(t.text_secondary))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, chunks[0]);

    let prompt = Paragraph::new(m.import_prompt)
        .style(Style::default().fg(t.text_primary))
        .wrap(Wrap { trim: false })
        .scroll((app.import_prompt_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(t.border_type)
                .border_style(Style::default().fg(t.border_unfocused)),
        );
    frame.render_widget(prompt, chunks[1]);

    let mut spans: Vec<Span<'_>> = Vec::new();
    if app.import_copied {
        spans.push(Span::styled(
            format!("✓ {}  ", m.import_step1_copied),
            Style::default()
                .fg(t.status_pass)
                .add_modifier(Modifier::BOLD),
        ));
    }
    // Two rows: the copy confirmation on top, the keys underneath.
    let footer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        footer[0],
    );
    render_hint(
        frame,
        &[
            ("c", m.import_step1_copy),
            ("f", m.import_step1_fullscreen),
            ("Enter", m.import_step1_next),
            ("Esc", m.cancel),
        ],
        footer[1],
    );
}

/// Full-screen prompt view: bare text covering the whole terminal so the
/// user can select with the mouse and copy via their terminal's own shortcut.
fn draw_import_prompt_fullscreen(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = frame.area();

    // Wipe whatever was rendered underneath (popup borders, panels, …) so the
    // selection only picks up the prompt text.
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let prompt = Paragraph::new(m.import_prompt)
        .style(Style::default().fg(t.text_primary))
        .wrap(Wrap { trim: false })
        .scroll((app.import_prompt_scroll, 0));
    frame.render_widget(prompt, chunks[0]);

    render_hint(
        frame,
        &[("Esc/f", m.import_step1_fullscreen_exit)],
        chunks[1],
    );
}

// =============================================================================
// Step 2 — Paste
// =============================================================================

pub fn draw_import_paste(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step2_title),
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // intro hint
            Constraint::Min(0),    // error area
            Constraint::Length(1), // footer
        ])
        .split(inner);

    let hint = Paragraph::new(m.import_step2_hint)
        .style(Style::default().fg(t.text_secondary))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, chunks[0]);

    let body = if let Some(err) = &app.import_paste_error {
        Paragraph::new(err.as_str())
            .style(Style::default().fg(t.status_fail))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("")
    };
    let paste_box = Block::default()
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.border_unfocused));
    let paste_inner = paste_box.inner(chunks[1]);
    frame.render_widget(paste_box, chunks[1]);
    frame.render_widget(body, paste_inner);

    render_hint(frame, &[("b/Esc", m.import_step2_back)], chunks[2]);
}

// =============================================================================
// Step 3 — Preview
// =============================================================================

pub fn draw_import_preview(frame: &mut Frame, app: &App) {
    let m = app.messages();
    let t = theme();
    let area = centered_rect(70, 75, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            format!(" {} ", m.import_step3_title),
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(course) = &app.import_parsed else {
        let p = Paragraph::new("…").style(Style::default().fg(t.text_muted));
        frame.render_widget(p, inner);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    // -- Course header -------------------------------------------------------
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", m.name),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            course.name.clone(),
            Style::default()
                .fg(t.text_primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(from) = &app.import_renamed_from {
        lines.push(Line::from(Span::styled(
            format!(
                "  ({} {} \u{2192} {})",
                m.import_renamed_to, from, course.name
            ),
            Style::default().fg(t.status_warn),
        )));
    }

    let mut header = vec![
        Span::styled(
            format!("{}: ", m.passing_grade),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            format!("{:.0}", course.passing_grade),
            Style::default().fg(t.text_primary),
        ),
    ];
    if let Some(credits) = course.credits {
        header.push(Span::styled(
            format!("   \u{00b7}   {}: ", m.metric_credits),
            Style::default().fg(t.text_secondary),
        ));
        header.push(Span::styled(
            credits.to_string(),
            Style::default().fg(t.text_primary),
        ));
    }
    lines.push(Line::from(header));

    // The global policy moves the final grade as much as any category weight,
    // so it has to be visible before the user confirms.
    lines.push(global_line(course, m, t));

    // -- Weight summary ------------------------------------------------------
    let weight_color = if (app.import_total_weight - 100.0).abs() < 0.01 {
        t.status_pass
    } else {
        t.status_warn
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", m.import_step3_total_weight),
            Style::default().fg(t.text_secondary),
        ),
        Span::styled(
            format!("{:.0}%", app.import_total_weight),
            Style::default()
                .fg(weight_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "   \u{00b7}   {} {}",
                app.import_total_evals, m.import_step3_evaluations
            ),
            Style::default().fg(t.text_muted),
        ),
    ]));

    if (app.import_total_weight - 100.0).abs() >= 0.01 {
        lines.push(Line::from(Span::styled(
            format!("\u{26a0} {}", m.import_warning_weight_not_100),
            Style::default().fg(t.status_warn),
        )));
    }

    lines.push(Line::from(""));

    // -- Categories ----------------------------------------------------------
    for cat in &course.categories {
        let mut header_spans = vec![
            Span::styled("\u{258e} ", Style::default().fg(t.popup_border)),
            Span::styled(
                cat.name.clone(),
                Style::default()
                    .fg(t.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({:.0}%)", cat.weight),
                Style::default().fg(t.weight_label),
            ),
        ];
        for badge in category_badges(cat, m) {
            header_spans.push(Span::styled(
                format!("  {}", badge),
                Style::default().fg(t.status_override),
            ));
        }
        lines.push(Line::from(header_spans));

        for rule in minimum_rules(cat, m) {
            lines.push(Line::from(Span::styled(
                format!("    {}", rule),
                Style::default().fg(t.status_warn),
            )));
        }

        // Every evaluation with whatever grade the AI claimed. A bare count
        // would hide an invented grade until after the import.
        for ev in &cat.evaluations {
            let mut spans = vec![
                Span::styled("    \u{00b7} ", Style::default().fg(t.text_muted)),
                Span::styled(ev.name.clone(), Style::default().fg(t.text_secondary)),
            ];
            match ev.grade {
                Some(g) => spans.push(Span::styled(
                    format!("  {:.0}", g),
                    Style::default().fg(t.status_warn),
                )),
                None => spans.push(Span::styled(
                    "  \u{2014}".to_string(),
                    Style::default().fg(t.text_muted),
                )),
            }
            if cat.rules.weighted_evaluations
                && let Some(w) = ev.weight
            {
                spans.push(Span::styled(
                    format!("  ({:.0}%)", w),
                    Style::default().fg(t.weight_label),
                ));
            }
            lines.push(Line::from(spans));
        }

        if cat.evaluations.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("    0 {}", m.import_step3_evaluations),
                Style::default().fg(t.text_muted),
            )));
        }
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.import_preview_scroll, 0));
    frame.render_widget(body, chunks[0]);

    render_hint(
        frame,
        &[
            ("b/Esc", m.import_step2_back),
            ("j/k", m.help_move_updown),
            ("t", m.import_step3_save_as_template),
            ("Enter", m.import_step3_confirm),
        ],
        chunks[1],
    );
}

/// One line describing the global exam, which moves the final grade as much as
/// the category weights do.
fn global_line(course: &Course, m: &Messages, t: &Theme) -> Line<'static> {
    let label = Span::styled(
        format!("{}: ", m.global_exam),
        Style::default().fg(t.text_secondary),
    );

    match &course.global_policy {
        GlobalExamPolicy::None => Line::from(vec![
            label,
            Span::styled(
                m.global_policy_none.to_string(),
                Style::default().fg(t.text_muted),
            ),
        ]),
        GlobalExamPolicy::Weighted {
            semester_weight,
            global_weight,
        } => {
            let mut spans = vec![
                label,
                Span::styled(
                    format!(
                        "{}  {:.0}% / {:.0}%",
                        m.global_policy_weighted,
                        semester_weight * 100.0,
                        global_weight * 100.0
                    ),
                    Style::default().fg(t.status_override),
                ),
            ];
            push_min_grade(&mut spans, course, m, t);
            Line::from(spans)
        }
        GlobalExamPolicy::ReplacesWorstGrade => {
            let mut spans = vec![
                label,
                Span::styled(
                    m.global_policy_replaces.to_string(),
                    Style::default().fg(t.status_override),
                ),
            ];
            push_min_grade(&mut spans, course, m, t);
            Line::from(spans)
        }
    }
}

fn push_min_grade(spans: &mut Vec<Span<'static>>, course: &Course, m: &Messages, t: &Theme) {
    if let Some(min) = course.global_eligibility.min_grade {
        spans.push(Span::styled(
            format!("   \u{00b7}   {}: {:.0}", m.global_min_grade, min),
            Style::default().fg(t.text_muted),
        ));
    }
}

/// Short badges for the category rules that the numbers alone do not reveal.
fn category_badges(cat: &Category, m: &Messages) -> Vec<String> {
    let mut badges = Vec::new();
    if cat.rules.drop_lowest > 0 {
        badges.push(format!("-{}", cat.rules.drop_lowest));
    }
    if cat.rules.averaging_method == AveragingMethod::Geometric {
        badges.push(m.averaging_geometric.to_string());
    }
    if cat.rules.weighted_evaluations {
        badges.push(m.weighted_evaluations.to_string());
    }
    if cat.rules.round_before_weighting {
        badges.push(m.round_before_weighting.to_string());
    }
    badges
}

/// One line per minimum the syllabus set, each with what failing it does.
fn minimum_rules(cat: &Category, m: &Messages) -> Vec<String> {
    let action = |a: MinimumNotMetAction| match a {
        MinimumNotMetAction::FinalEqualsAverage => m.action_final_equals_avg,
        MinimumNotMetAction::RequiresGlobal => m.action_requires_global,
        MinimumNotMetAction::FailCourse => m.action_fail_course,
    };

    let mut rules = Vec::new();
    if let Some(v) = cat.rules.minimum_average {
        rules.push(format!(
            "{}: {:.0}  \u{2192}  {}",
            m.minimum_average,
            v,
            action(cat.rules.on_minimum_not_met)
        ));
    }
    if let Some(v) = cat.rules.minimum_per_evaluation {
        rules.push(format!(
            "{}: {:.0}  \u{2192}  {}",
            m.minimum_per_evaluation,
            v,
            action(cat.rules.on_min_per_eval_not_met)
        ));
    }
    if let Some(v) = cat.rules.minimum_one_eval {
        rules.push(format!(
            "{}: {:.0}  \u{2192}  {}",
            m.minimum_one_eval,
            v,
            action(cat.rules.on_min_one_eval_not_met)
        ));
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::model::CategoryRules;

    fn category_with_every_rule() -> Category {
        let rules = CategoryRules {
            minimum_average: Some(50.0),
            on_minimum_not_met: MinimumNotMetAction::RequiresGlobal,
            drop_lowest: 1,
            averaging_method: AveragingMethod::Geometric,
            minimum_per_evaluation: Some(30.0),
            on_min_per_eval_not_met: MinimumNotMetAction::FailCourse,
            minimum_one_eval: Some(60.0),
            on_min_one_eval_not_met: MinimumNotMetAction::FinalEqualsAverage,
            round_before_weighting: true,
            weighted_evaluations: true,
        };
        Category::with_rules("Cat".to_string(), 100.0, Vec::new(), rules)
    }

    #[test]
    fn preview_surfaces_every_minimum_with_its_action() {
        let rules = minimum_rules(&category_with_every_rule(), &EN);
        assert_eq!(rules.len(), 3, "one line per minimum: {rules:?}");
        assert!(rules[0].contains(EN.action_requires_global));
        assert!(rules[1].contains(EN.action_fail_course));
        assert!(rules[2].contains(EN.action_final_equals_avg));
    }

    #[test]
    fn preview_badges_every_non_obvious_rule() {
        let badges = category_badges(&category_with_every_rule(), &EN);
        assert_eq!(
            badges.len(),
            4,
            "drop, geometric, weighted, round: {badges:?}"
        );
        assert!(badges.contains(&"-1".to_string()));
    }

    #[test]
    fn preview_shows_nothing_for_a_category_without_rules() {
        let plain = Category::new("Plain".to_string(), 100.0);
        assert!(minimum_rules(&plain, &EN).is_empty());
        assert!(category_badges(&plain, &EN).is_empty());
    }
}
