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

//! Keybinding hints: the `[key] label` rows shown in the footer and at the
//! bottom of every popup, and the width ladder they degrade through.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::theme::{Theme, theme};

/// Gap between footer entries.
const GAP: &str = "  ";
/// Marks entries dropped because they did not fit.
const ELLIPSIS: &str = "\u{2026}";

/// Build a styled `Line` from a slice of `(key, description)` pairs.
///
/// Degrades in three steps as width shrinks, because dropping every
/// description at once leaves bare keys that mean nothing:
/// - **Full**: `[key] description  [key] description`
/// - **Trimmed**: entries drop off the right, marked with an ellipsis. The
///   last pair is always kept, since it points at the help overlay.
/// - **Keys only**: `[key] [key]`, the last resort.
///
/// The `available_width` is the inner width (excluding borders) of the footer.
pub fn styled_keybindings<'a>(
    pairs: &[(&'a str, &'a str)],
    t: &Theme,
    available_width: u16,
) -> Line<'a> {
    let width = available_width as usize;

    // "[key] desc", plus the gap that precedes every entry but the first.
    let entry_w = |(key, desc): &(&str, &str), first: bool| {
        let w = key.chars().count() + 3 + desc.chars().count();
        if first { w } else { w + GAP.len() }
    };
    let key_only_w = |(key, _): &(&str, &str), first: bool| {
        let w = key.chars().count() + 2;
        if first { w } else { w + GAP.len() }
    };

    let full: usize = pairs
        .iter()
        .enumerate()
        .map(|(i, p)| entry_w(p, i == 0))
        .sum();

    if full <= width {
        return render_pairs(pairs, t, true, false);
    }

    // Keep as many labelled entries as fit, reserving room for the ellipsis
    // and the final pair.
    if let Some(last) = pairs.last() {
        let reserved = GAP.len() + ELLIPSIS.chars().count() + entry_w(last, false);
        let mut used = 0;
        let mut kept = 0;
        for (i, pair) in pairs[..pairs.len().saturating_sub(1)].iter().enumerate() {
            let w = entry_w(pair, i == 0);
            if used + w + reserved > width {
                break;
            }
            used += w;
            kept += 1;
        }

        if kept > 0 {
            let mut shown: Vec<(&str, &str)> = pairs[..kept].to_vec();
            shown.push(*last);
            return render_pairs(&shown, t, true, true);
        }
    }

    // Not even one labelled entry fits — bare keys, trimmed from the right if
    // those overflow too.
    let mut kept = pairs.len();
    while kept > 1 {
        let w: usize = pairs[..kept]
            .iter()
            .enumerate()
            .map(|(i, p)| key_only_w(p, i == 0))
            .sum();
        if w <= width {
            break;
        }
        kept -= 1;
    }
    render_pairs(&pairs[..kept], t, false, false)
}

/// Render `[key] description` entries, optionally without descriptions or with
/// an ellipsis before the final entry.
fn render_pairs<'a>(
    pairs: &[(&'a str, &'a str)],
    t: &Theme,
    with_desc: bool,
    ellipsis_before_last: bool,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::with_capacity(pairs.len() * 4);

    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(GAP));
        }
        if ellipsis_before_last && i == pairs.len() - 1 {
            spans.push(Span::styled(ELLIPSIS, Style::default().fg(t.footer_border)));
            spans.push(Span::raw(GAP));
        }

        spans.push(Span::styled("[", Style::default().fg(t.footer_border)));
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(t.footer_key)
                .add_modifier(Modifier::BOLD),
        ));
        // The trailing space belongs to the label, not the bracket, or
        // keys-only mode renders a stray column per entry.
        spans.push(Span::styled(
            if with_desc { "] " } else { "]" },
            Style::default().fg(t.footer_border),
        ));

        if with_desc {
            spans.push(Span::styled(*desc, Style::default().fg(t.footer_desc)));
        }
    }

    Line::from(spans)
}

/// Draw a keybinding hint centred inside `area`, the way every popup shows the
/// keys that act on it. Shares the width ladder with the main footer, so a
/// narrow terminal degrades the same way everywhere.
pub fn render_hint(frame: &mut Frame, pairs: &[(&str, &str)], area: Rect) {
    if area.height == 0 {
        return;
    }
    let line = styled_keybindings(pairs, theme(), area.width);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme;

    const PAIRS: [(&str, &str); 3] = [("q", "Quit"), ("n", "New"), ("?", "Help")];

    /// Visible width of a rendered line.
    fn width(line: &Line) -> usize {
        line.spans.iter().map(|s| s.content.chars().count()).sum()
    }

    fn text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn full_mode_brackets_every_key_and_keeps_labels() {
        let line = styled_keybindings(&PAIRS, theme(), 80);
        assert_eq!(text(&line), "[q] Quit  [n] New  [?] Help");
    }

    #[test]
    fn trimmed_mode_drops_from_the_right_but_keeps_the_help_key() {
        // Fits "[q] Quit", the ellipsis and "[?] Help", but not "[n] New".
        let line = styled_keybindings(&PAIRS, theme(), 24);
        let rendered = text(&line);
        assert!(rendered.contains("[q] Quit"), "{rendered}");
        assert!(rendered.contains("[?] Help"), "{rendered}");
        assert!(rendered.contains('\u{2026}'), "{rendered}");
        assert!(!rendered.contains("[n] New"), "{rendered}");
    }

    #[test]
    fn keys_only_mode_when_no_label_fits() {
        let line = styled_keybindings(&PAIRS, theme(), 13);
        assert_eq!(text(&line), "[q]  [n]  [?]");
    }

    #[test]
    fn never_exceeds_the_available_width_at_any_tier() {
        for w in 4..=80u16 {
            let line = styled_keybindings(&PAIRS, theme(), w);
            assert!(
                width(&line) <= w as usize,
                "width {w} overflowed: {:?}",
                text(&line)
            );
        }
    }
}
