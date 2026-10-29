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

//! Semantic colour theme for the UI layer.
//!
//! Every colour used by the rendering code is defined here so that the entire
//! palette can be swapped by switching to a different `Theme` instance.
//! The default theme uses the Catppuccin Mocha palette with rounded borders.

use std::sync::OnceLock;

use ratatui::style::Color;
use ratatui::widgets::BorderType;

// =============================================================================
// Catppuccin Mocha Palette
// =============================================================================

/// Subtle tint between the terminal base and SURFACE0 — used for the
/// alternating "zebra" rows in the evaluations table.  Distinct enough to
/// be perceptible but far softer than a regular Catppuccin surface.
const ZEBRA: Color = Color::Rgb(0x2a, 0x2b, 0x3c);
const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);
const SURFACE2: Color = Color::Rgb(0x58, 0x5b, 0x70);
const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
const SUBTEXT0: Color = Color::Rgb(0xa6, 0xad, 0xc8);
const SUBTEXT1: Color = Color::Rgb(0xba, 0xc2, 0xde);
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const FLAMINGO: Color = Color::Rgb(0xf2, 0xcd, 0xcd);
const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);
const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const AMBER: Color = Color::Rgb(0xf5, 0xb9, 0x5f);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const SAPPHIRE: Color = Color::Rgb(0x74, 0xc7, 0xec);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const LAVENDER: Color = Color::Rgb(0xb4, 0xbe, 0xfe);
const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);

/// A complete semantic colour theme used by every UI component.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // -- Borders ----------------------------------------------------------
    /// Border colour for the currently focused panel.
    pub border_focused: Color,
    /// Border colour for unfocused panels.
    pub border_unfocused: Color,
    /// Border type (Rounded, Plain, Double, Thick).
    pub border_type: BorderType,

    // -- Text -------------------------------------------------------------
    /// Primary text colour (headings, names, bold labels).
    pub text_primary: Color,
    /// Secondary text colour (descriptions, hints).
    pub text_secondary: Color,
    /// Muted text colour (placeholders, disabled items).
    pub text_muted: Color,

    // -- Semantic status --------------------------------------------------
    /// Passing / success colour.
    pub status_pass: Color,
    /// Failing / error colour.
    pub status_fail: Color,
    /// Warning colour (weights under 100%, approaching limits).
    pub status_warn: Color,
    /// A grade still to reach, such as the one needed to pass.
    pub status_target: Color,
    /// Informational / accent colour (active rules, toggles).
    pub status_info: Color,

    // -- Special statuses -------------------------------------------------
    /// Colour for minimum-not-met / capped / rule-override indicators.
    pub status_override: Color,

    // -- Highlight --------------------------------------------------------
    /// Background colour for the selected/highlighted item in lists.
    pub highlight_bg: Color,

    /// Subtle alternating row background for tabular views (zebra striping).
    pub zebra_bg: Color,

    // -- Input fields -----------------------------------------------------
    /// Border / text colour for the currently active input field.
    pub input_active: Color,
    /// Border / text colour for inactive input fields.
    pub input_inactive: Color,

    // -- Footer -----------------------------------------------------------
    /// Colour for key-binding keys in the footer (e.g. "q", "n").
    pub footer_key: Color,
    /// Colour for key-binding descriptions in the footer (e.g. "Quit").
    pub footer_desc: Color,
    /// Border colour for the footer bar.
    pub footer_border: Color,

    // -- Popup specific ---------------------------------------------------
    /// Border colour for popup dialogs.
    pub popup_border: Color,
    /// Border colour for delete/destructive confirmation popups.
    pub popup_border_danger: Color,
    /// Separator line colour inside popups.
    pub popup_separator: Color,

    // -- Weight indicator -------------------------------------------------
    /// Colour for the weight percentage label next to category names.
    pub weight_label: Color,

    // -- User template indicator ------------------------------------------
    /// Colour used for user-created template items.
    pub user_template: Color,
}

/// The default Kalk theme — Catppuccin Mocha with rounded borders.
pub const DEFAULT_THEME: Theme = Theme {
    // Borders — Mauve for focused (vibrant, clear active panel), Overlay 0 for
    // unfocused.  Lavender is reserved for popup borders where the contrast is
    // against the dimmed underlay rather than other panels.
    border_focused: MAUVE,
    border_unfocused: OVERLAY0,
    border_type: BorderType::Rounded,

    // Text — Catppuccin text hierarchy.  `text_muted` uses OVERLAY0 so
    // placeholders/disabled items read as genuinely dim.
    text_primary: TEXT,
    text_secondary: SUBTEXT1,
    text_muted: OVERLAY0,

    // Semantic status — standard Catppuccin accent mapping
    status_pass: GREEN,
    status_fail: RED,
    status_warn: AMBER,
    status_target: LAVENDER,
    status_info: SAPPHIRE,

    // Special — Peach (orange) for rule overrides (alarm/warning semantics)
    status_override: PEACH,

    // Highlight — Surface 2 bg keeps items readable
    highlight_bg: SURFACE2,

    // Zebra striping — a touch lighter than the terminal base so alternating
    // rows are scannable without looking selected.
    zebra_bg: ZEBRA,

    // Input fields — Peach for active (warm accent), Overlay 0 for inactive
    input_active: PEACH,
    input_inactive: OVERLAY0,

    // Footer — Blue keys, Subtext 0 descriptions, Surface 1 border
    footer_key: BLUE,
    footer_desc: SUBTEXT0,
    footer_border: SURFACE1,

    // Popups — Lavender border, Red for danger, Surface 1 separators
    popup_border: LAVENDER,
    popup_border_danger: RED,
    popup_separator: SURFACE1,

    // Weight label — Mauve so it does not visually rhyme with the Peach used
    // for active input borders, while staying distinct from status colours.
    weight_label: MAUVE,

    // User template — Flamingo (soft pink, distinct from system templates)
    user_template: FLAMINGO,
};

impl Theme {
    /// Muted text for a list row. On the selected row it steps up to the
    /// secondary colour: muted on the highlight background is unreadable.
    pub fn muted_on(&self, selected: bool) -> Color {
        if selected {
            self.text_secondary
        } else {
            self.text_muted
        }
    }

    /// The same theme in the 256-colour palette, for terminals that would
    /// otherwise render the RGB values as unrelated colours.
    fn to_256(self) -> Self {
        Self {
            border_focused: indexed(self.border_focused),
            border_unfocused: indexed(self.border_unfocused),
            text_primary: indexed(self.text_primary),
            text_secondary: indexed(self.text_secondary),
            text_muted: indexed(self.text_muted),
            status_pass: indexed(self.status_pass),
            status_fail: indexed(self.status_fail),
            status_warn: indexed(self.status_warn),
            status_target: indexed(self.status_target),
            status_info: indexed(self.status_info),
            status_override: indexed(self.status_override),
            highlight_bg: indexed(self.highlight_bg),
            zebra_bg: indexed(self.zebra_bg),
            input_active: indexed(self.input_active),
            input_inactive: indexed(self.input_inactive),
            footer_key: indexed(self.footer_key),
            footer_desc: indexed(self.footer_desc),
            footer_border: indexed(self.footer_border),
            popup_border: indexed(self.popup_border),
            popup_border_danger: indexed(self.popup_border_danger),
            popup_separator: indexed(self.popup_separator),
            weight_label: indexed(self.weight_label),
            user_template: indexed(self.user_template),
            border_type: self.border_type,
        }
    }
}

/// Map an RGB colour to the nearest of the 256 indexed colours: the 6x6x6
/// cube or the 24-step grey ramp, whichever is closer.
fn indexed(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Indexed(nearest_256(r, g, b)),
        other => other,
    }
}

fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let level = |v: u8| {
        STEPS
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| (i32::from(**s) - i32::from(v)).abs())
            .map_or(0, |(i, _)| i as u8)
    };
    let dist = |(r2, g2, b2): (u8, u8, u8)| {
        let d = |a: u8, b: u8| (i32::from(a) - i32::from(b)).pow(2);
        d(r, r2) + d(g, g2) + d(b, b2)
    };

    let (ri, gi, bi) = (level(r), level(g), level(b));
    let cube = (STEPS[ri as usize], STEPS[gi as usize], STEPS[bi as usize]);
    let cube_index = 16 + 36 * ri + 6 * gi + bi;

    let mean = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
    let grey_step = (mean.saturating_sub(8) / 10).min(23) as u8;
    let grey = 8 + 10 * grey_step;
    let grey_index = 232 + grey_step;

    if dist((grey, grey, grey)) < dist(cube) {
        grey_index
    } else {
        cube_index
    }
}

/// Whether the terminal advertises 24-bit colour. Most that support it set
/// `COLORTERM`; Windows Terminal does not, but sets `WT_SESSION`.
fn supports_truecolor() -> bool {
    let advertises = |var: &str| {
        std::env::var(var).is_ok_and(|v| {
            let v = v.to_ascii_lowercase();
            v.contains("truecolor") || v.contains("24bit") || v.contains("direct")
        })
    };
    advertises("COLORTERM") || advertises("TERM") || std::env::var_os("WT_SESSION").is_some()
}

/// Return a reference to the active theme: the RGB palette where the
/// terminal supports it, its 256-colour approximation elsewhere.
pub fn theme() -> &'static Theme {
    static ACTIVE: OnceLock<Theme> = OnceLock::new();
    ACTIVE.get_or_init(|| {
        if supports_truecolor() {
            DEFAULT_THEME
        } else {
            DEFAULT_THEME.to_256()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_cube_and_grey_colours_map_to_themselves() {
        assert_eq!(nearest_256(0, 0, 0), 16);
        assert_eq!(nearest_256(255, 255, 255), 231);
        assert_eq!(nearest_256(255, 0, 0), 196);
        assert_eq!(nearest_256(0x58, 0x58, 0x58), 240);
    }

    #[test]
    fn the_palette_keeps_its_hue_in_256_colours() {
        // Catppuccin green and red must not collapse into the same index.
        assert_ne!(nearest_256(0xa6, 0xe3, 0xa1), nearest_256(0xf3, 0x8b, 0xa8));
    }
}
