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

//! Rendering-only transition state, independent from saved application data.

use super::theme::theme;
use crate::app::{App, Screen};
use ratatui::{Frame, style::Color};
use std::time::{Duration, Instant};
use tachyonfx::{CellFilter, Effect, Interpolation, fx};

#[derive(Default)]
pub struct Transitions {
    previous: Option<Screen>,
    effect: Option<Effect>,
    last_frame: Option<Instant>,
}

impl Transitions {
    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        let now = Instant::now();
        let elapsed = self
            .last_frame
            .replace(now)
            .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        self.advance(frame, app, elapsed);
    }

    fn advance(&mut self, frame: &mut Frame, app: &App, elapsed: Duration) {
        let t = theme();
        let enabled = app.animations && !matches!(t.text_primary, Color::Indexed(_));
        let changed = self.previous.as_ref() != Some(&app.screen);
        let entering = changed && self.previous.is_some();
        if changed {
            self.previous = Some(app.screen.clone());
            self.effect = None;
        }
        if !enabled {
            self.effect = None;
            return;
        }
        if entering
            && !matches!(
                app.screen,
                Screen::Main | Screen::Home | Screen::Welcome | Screen::WelcomeFonts
            )
        {
            self.effect = Some(
                fx::fade_from_fg(t.text_secondary, (120, Interpolation::QuadOut))
                    .with_filter(CellFilter::BgColor(t.popup_bg)),
            );
        }
        let area = frame.area();
        if let Some(effect) = &mut self.effect {
            // Do not consume the time spent waiting for the key that opened the dialog.
            effect.process(
                if entering { Duration::ZERO } else { elapsed },
                frame.buffer_mut(),
                area,
            );
            if effect.done() {
                self.effect = None;
            }
        }
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(if self.effect.is_some() { 16 } else { 100 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn transitions_finish_and_respect_disable_and_palette_fallback() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut transitions = Transitions::default();
        let mut app = App::default();
        terminal
            .draw(|frame| transitions.advance(frame, &app, Duration::ZERO))
            .unwrap();
        assert!(transitions.effect.is_none());
        app.screen = Screen::Settings;
        terminal
            .draw(|frame| transitions.advance(frame, &app, Duration::ZERO))
            .unwrap();
        assert_eq!(
            transitions.effect.is_some(),
            matches!(theme().text_primary, Color::Rgb(..))
        );
        terminal
            .draw(|frame| transitions.advance(frame, &app, Duration::from_millis(150)))
            .unwrap();
        assert!(transitions.effect.is_none());
        assert_eq!(transitions.poll_interval(), Duration::from_millis(100));
        app.screen = Screen::Help;
        app.animations = false;
        terminal
            .draw(|frame| transitions.advance(frame, &app, Duration::ZERO))
            .unwrap();
        assert!(transitions.effect.is_none());
    }
}
