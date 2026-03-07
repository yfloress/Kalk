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

//! Icon sets for the UI layer.
//!
//! Provides two sets of icons: one using Nerd Font glyphs (requires a patched
//! font) and one using plain Unicode characters that render correctly in any
//! modern terminal. The active set is selected at runtime based on the user's
//! `use_nerd_fonts` configuration flag.

/// A complete set of icons used throughout the UI.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct IconSet {
    // Status indicators
    pub passed: &'static str,
    pub failed: &'static str,
    pub warning: &'static str,
    pub info: &'static str,

    // Navigation / selection
    pub arrow_right: &'static str,
    pub arrow_left: &'static str,
    pub arrow_up: &'static str,
    pub arrow_down: &'static str,
    pub selected: &'static str,
    pub unselected: &'static str,
    pub highlight: &'static str,

    // Panel / entity indicators
    pub course: &'static str,
    pub category: &'static str,
    pub evaluation: &'static str,
    pub template: &'static str,
    pub user_template: &'static str,

    // Actions
    pub delete: &'static str,
    pub edit: &'static str,
    pub add: &'static str,
    pub save: &'static str,
    pub settings: &'static str,
    pub language: &'static str,
    pub balance: &'static str,

    // Grade / weight indicators
    pub weight_ok: &'static str,
    pub weight_warn: &'static str,
    pub weight_error: &'static str,
    pub grade: &'static str,
    pub average: &'static str,
    pub dropped: &'static str,
    pub below_min: &'static str,
    pub needs_global: &'static str,
    pub capped: &'static str,

    // Rules
    pub rules_active: &'static str,
    pub advanced_expand: &'static str,
    pub advanced_collapse: &'static str,

    // Separators / decorative
    pub separator_dot: &'static str,
    pub key_hint_sep: &'static str,
    pub bullet: &'static str,
}

/// Nerd Font icon set — requires a Nerd Font patched terminal font.
pub const NERD: IconSet = IconSet {
    // Status indicators
    passed: "\u{f00c} ",  //  (check mark)
    failed: "\u{f00d} ",  //  (x mark)
    warning: "\u{f071} ", //  (warning triangle)
    info: "\u{f05a} ",    //  (info circle)

    // Navigation / selection
    arrow_right: "\u{f054} ", //  (chevron right)
    arrow_left: "\u{f053} ",  //  (chevron left)
    arrow_up: "\u{f077} ",    //  (chevron up)
    arrow_down: "\u{f078} ",  //  (chevron down)
    selected: "\u{f192} ",    //  (dot circle)
    unselected: "\u{f10c} ",  //  (circle outline)
    highlight: "\u{f054} ",   //  (chevron right)

    // Panel / entity indicators
    course: "\u{f19d} ",        //  (graduation cap)
    category: "\u{f07b} ",      //  (folder)
    evaluation: "\u{f15c} ",    //  (file text)
    template: "\u{f0c5} ",      //  (copy / template)
    user_template: "\u{f005} ", //  (star)

    // Actions
    delete: "\u{f1f8} ",   //  (trash)
    edit: "\u{f044} ",     //  (edit / pencil)
    add: "\u{f067} ",      //  (plus)
    save: "\u{f0c7} ",     //  (floppy disk)
    settings: "\u{f013} ", //  (gear)
    language: "\u{f0ac} ", //  (globe)
    balance: "\u{f24e} ",  //  (balance scale)

    // Grade / weight indicators
    weight_ok: "\u{f00c}",     //  (check)
    weight_warn: "\u{f071}",   //  (warning)
    weight_error: "\u{f057}",  //  (times circle)
    grade: "\u{f005} ",        //  (star)
    average: "\u{f1ec} ",      //  (calculator)
    dropped: "\u{f056} ",      //  (minus circle)
    below_min: "\u{f06a} ",    //  (exclamation circle)
    needs_global: "\u{f0e7} ", //  (bolt)
    capped: "\u{f023} ",       //  (lock)

    // Rules
    rules_active: "\u{f085} ",      //  (gears / cogs)
    advanced_expand: "\u{f0da} ",   //  (caret right)
    advanced_collapse: "\u{f0d7} ", //  (caret down)

    // Separators / decorative
    separator_dot: " \u{2502} ", // │ (box drawing vertical)
    key_hint_sep: " \u{2502} ",  // │
    bullet: "\u{f111} ",         //  (circle)
};

/// Plain Unicode fallback icon set — works in any modern terminal without
/// special fonts.
pub const UNICODE: IconSet = IconSet {
    // Status indicators
    passed: "\u{2713} ", // ✓
    failed: "\u{2717} ", // ✗
    warning: "! ",
    info: "i ",

    // Navigation / selection
    arrow_right: "\u{25b8} ", // ▸
    arrow_left: "\u{25c2} ",  // ◂
    arrow_up: "\u{25b4} ",    // ▴
    arrow_down: "\u{25be} ",  // ▾
    selected: "\u{25cf} ",    // ●
    unselected: "\u{25cb} ",  // ○
    highlight: "\u{25b8} ",   // ▸

    // Panel / entity indicators
    course: "\u{25a0} ",        // ■
    category: "\u{25c6} ",      // ◆
    evaluation: "\u{25aa} ",    // ▪
    template: "\u{25cb} ",      // ○
    user_template: "\u{2605} ", // ★

    // Actions
    delete: "\u{2717} ", // ✗
    edit: "\u{270e} ",   // ✎
    add: "+ ",
    save: "\u{2713} ",     // ✓
    settings: "\u{2699} ", // ⚙
    language: "\u{2637} ", // ☷
    balance: "\u{2696} ",  // ⚖

    // Grade / weight indicators
    weight_ok: "\u{2713}", // ✓
    weight_warn: "!",
    weight_error: "\u{2717}", // ✗
    grade: "\u{2605} ",       // ★
    average: "\u{00d8} ",     // Ø (average symbol)
    dropped: "- ",
    below_min: "! ",
    needs_global: "!! ",
    capped: "\u{2193} ", // ↓

    // Rules
    rules_active: "\u{2699} ",      // ⚙
    advanced_expand: "\u{25b6} ",   // ▶
    advanced_collapse: "\u{25bc} ", // ▼

    // Separators / decorative
    separator_dot: " | ",
    key_hint_sep: " | ",
    bullet: "\u{2022} ", // •
};

/// Return the icon set corresponding to the user's preference.
pub fn icons(use_nerd_fonts: bool) -> &'static IconSet {
    if use_nerd_fonts { &NERD } else { &UNICODE }
}
