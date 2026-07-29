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
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const SAPPHIRE: Color = Color::Rgb(0x74, 0xc7, 0xec);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const LAVENDER: Color = Color::Rgb(0xb4, 0xbe, 0xfe);
const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);

/// A complete semantic colour theme used by every UI component.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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
    /// Informational / accent colour (active rules, toggles).
    pub status_info: Color,

    // -- Special statuses -------------------------------------------------
    /// Colour for minimum-not-met / capped / rule-override indicators.
    pub status_override: Color,

    // -- Highlight --------------------------------------------------------
    /// Background colour for the selected/highlighted item in lists.
    pub highlight_bg: Color,
    /// Foreground colour when an item is highlighted.
    pub highlight_fg: Color,

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
    status_warn: YELLOW,
    status_info: SAPPHIRE,

    // Special — Peach (orange) for rule overrides (alarm/warning semantics)
    status_override: PEACH,

    // Highlight — Surface 2 bg keeps items readable
    highlight_bg: SURFACE2,
    highlight_fg: TEXT,

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

/// Return a reference to the active theme.
///
/// Currently only `DEFAULT_THEME` exists; this function is the single point of
/// access so that future theme-switching only requires changing this function.
pub fn theme() -> &'static Theme {
    &DEFAULT_THEME
}
