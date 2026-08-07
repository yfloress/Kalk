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

//! First-run wizard: language, then whether Nerd Font icons render.
//!
//! Both questions have to be answered before the app is usable, and neither
//! can be asked in a language the user might not read — so step 1 lists the
//! languages under their own names and applies the highlighted one live, which
//! makes the rest of the wizard readable by the time it is shown.

use crate::i18n::Language;
use crate::persistence;
use crate::templates;

use super::{App, Screen};

impl App {
    pub fn welcome_next_language(&mut self) {
        let count = Language::all().len();
        if count > 0 {
            self.selected_language = (self.selected_language + 1) % count;
            self.apply_previewed_language();
        }
    }

    pub fn welcome_previous_language(&mut self) {
        let count = Language::all().len();
        if count > 0 {
            self.selected_language = (self.selected_language + count - 1) % count;
            self.apply_previewed_language();
        }
    }

    /// Switch to the highlighted language immediately, so moving the cursor
    /// shows what the choice actually means.
    fn apply_previewed_language(&mut self) {
        if let Some(&lang) = Language::all().get(self.selected_language) {
            self.language = lang;
        }
    }

    pub fn welcome_confirm_language(&mut self) {
        self.apply_previewed_language();
        self.built_in_templates = templates::built_in_templates(self.language);
        self.screen = Screen::WelcomeFonts;
    }

    pub fn welcome_toggle_fonts(&mut self) {
        self.use_nerd_fonts = !self.use_nerd_fonts;
    }

    pub fn welcome_set_fonts(&mut self, enabled: bool) {
        self.use_nerd_fonts = enabled;
    }

    pub fn welcome_back(&mut self) {
        self.screen = Screen::Welcome;
    }

    /// Finish the wizard and land on the dashboard.
    pub fn welcome_finish(&mut self) {
        self.configured = true;
        self.screen = Screen::Home;

        if persistence::save_config(&self.config()).is_err() {
            let msg = self.messages().config_save_error.to_string();
            self.set_error(msg);
        }
    }
}
