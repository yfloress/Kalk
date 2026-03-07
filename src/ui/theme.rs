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

//! Semantic colour theme for the UI layer.
//!
//! Every colour used by the rendering code is defined here so that the entire
//! palette can be swapped by switching to a different `Theme` instance.
//! The default theme preserves the original look of Kalk while using rounded
//! borders for a more modern feel.

use ratatui::style::Color;
use ratatui::widgets::BorderType;

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

/// The default Kalk theme — modernised with rounded borders but keeping the
/// original colour choices so existing users feel at home.
pub const DEFAULT_THEME: Theme = Theme {
    // Borders
    border_focused: Color::Cyan,
    border_unfocused: Color::DarkGray,
    border_type: BorderType::Rounded,

    // Text
    text_primary: Color::White,
    text_secondary: Color::Gray,
    text_muted: Color::DarkGray,

    // Semantic status
    status_pass: Color::Green,
    status_fail: Color::Red,
    status_warn: Color::Yellow,
    status_info: Color::Cyan,

    // Special
    status_override: Color::Magenta,

    // Highlight
    highlight_bg: Color::DarkGray,
    highlight_fg: Color::White,

    // Input fields
    input_active: Color::Yellow,
    input_inactive: Color::Reset,

    // Footer
    footer_key: Color::Cyan,
    footer_desc: Color::DarkGray,
    footer_border: Color::DarkGray,

    // Popups
    popup_border: Color::Cyan,
    popup_border_danger: Color::Red,
    popup_separator: Color::DarkGray,

    // Weight label
    weight_label: Color::Yellow,

    // User template
    user_template: Color::Yellow,
};

/// Return a reference to the active theme.
///
/// Currently only `DEFAULT_THEME` exists; this function is the single point of
/// access so that future theme-switching only requires changing this function.
pub fn theme() -> &'static Theme {
    &DEFAULT_THEME
}
