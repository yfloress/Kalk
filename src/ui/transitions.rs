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
use ratatui::{Frame, buffer::Buffer, layout::Rect, style::Color};
use std::time::{Duration, Instant};
use tachyonfx::{Effect, Interpolation, fx};

#[derive(Default)]
pub struct Transitions {
    previous: Option<Screen>,
    effect: Option<(Effect, Rect)>,
    last_frame: Option<Instant>,
    /// Last frame as drawn, before any effect, to find what a popup covers.
    last_buffer: Option<Buffer>,
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
        let drawn = frame.buffer_mut().clone();
        let previous_buffer = self.last_buffer.replace(drawn);
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
            let area = previous_buffer
                .and_then(|before| changed_area(&before, frame.buffer_mut()))
                .unwrap_or(frame.area());
            self.effect = Some((
                fx::fade_from_fg(t.text_secondary, (120, Interpolation::QuadOut)),
                area,
            ));
        }
        if let Some((effect, area)) = &mut self.effect {
            let area = area.intersection(frame.area());
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

/// Bounding box of the cells that differ: the popup that just opened.
fn changed_area(before: &Buffer, after: &Buffer) -> Option<Rect> {
    if before.area != after.area {
        return None;
    }
    let area = after.area;
    let (mut left, mut top, mut right, mut bottom) = (u16::MAX, u16::MAX, 0, 0);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if before[(x, y)] != after[(x, y)] {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }
    (left < right).then(|| Rect::new(left, top, right - left, bottom - top))
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

    #[test]
    fn the_fade_covers_only_what_the_popup_changed() {
        let before = Buffer::empty(Rect::new(0, 0, 20, 10));
        let mut after = before.clone();
        after[(4, 3)].set_symbol("a");
        after[(9, 6)].set_symbol("b");
        assert_eq!(changed_area(&before, &after), Some(Rect::new(4, 3, 6, 4)));
        assert_eq!(changed_area(&before, &before), None);
    }
}
