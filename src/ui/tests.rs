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

use super::*;
use crate::app::Screen;
use crate::i18n::Language;
use crate::model::{Category, Evaluation};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

fn render(app: &App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    terminal.backend().buffer().clone()
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

#[test]
fn status_and_keybindings_remain_visible_on_main_and_home() {
    for screen in [Screen::Main, Screen::Home] {
        for language in [Language::English, Language::Spanish] {
            let app = App {
                screen: screen.clone(),
                language,
                use_nerd_fonts: false,
                status_message: Some("Visible warning".to_string()),
                ..Default::default()
            };
            for height in [9, 30] {
                let buffer = render(&app, 120, height);
                assert!(row_text(&buffer, height - 5).contains("Visible warning"));
                assert!(row_text(&buffer, height - 2).contains('?'));
            }
        }
    }
}

#[test]
fn short_terminals_prioritize_the_status_banner() {
    for screen in [Screen::Main, Screen::Home] {
        let app = App {
            screen,
            status_message: Some("Visible warning".to_string()),
            ..Default::default()
        };
        for height in [3, 5, 8] {
            let buffer = render(&app, 120, height);
            let footer = row_text(&buffer, height - 2);
            assert!(footer.contains("Visible warning"));
            assert!(!footer.contains('?'));
        }
    }
}

#[test]
fn overlays_keep_the_screen_they_were_opened_from() {
    for origin in [Screen::Main, Screen::Home] {
        for overlay in [Screen::Settings, Screen::SelectingLanguage, Screen::Help] {
            let mut app = App {
                screen: origin.clone(),
                use_nerd_fonts: false,
                ..Default::default()
            };
            let background = render(&app, 120, 48);
            match overlay {
                Screen::Settings => app.show_settings(),
                Screen::SelectingLanguage => app.show_language_popup(),
                Screen::Help => app.show_help(),
                _ => unreachable!(),
            }
            let buffer = render(&app, 120, 48);
            for y in 0..3 {
                for x in 0..16 {
                    assert_eq!(buffer[(x, y)], background[(x, y)]);
                }
            }
            assert_ne!(buffer, background);
        }
    }
}

#[test]
fn scrollbar_reaches_both_ends_of_the_track() {
    let thumb = ratatui::symbols::scrollbar::VERTICAL.thumb;
    for content_len in [11, 50] {
        let mut terminal = Terminal::new(TestBackend::new(1, 12)).unwrap();
        for position in [0, content_len - 10] {
            terminal
                .draw(|frame| {
                    helpers::draw_scrollbar(frame, frame.area(), content_len, 10, position);
                })
                .unwrap();
            let buffer = terminal.backend().buffer();
            if position == 0 {
                assert_eq!(buffer[(0, 0)].symbol(), thumb);
                assert_ne!(buffer[(0, 11)].symbol(), thumb);
            } else {
                assert_ne!(buffer[(0, 0)].symbol(), thumb);
                assert_eq!(buffer[(0, 11)].symbol(), thumb);
            }
        }
    }
}

#[test]
fn preview_scroll_accounts_for_wrapping_and_terminal_resize() {
    let mut course = Course::new("Preview".to_string(), 55.0);
    let mut category = Category::new("Category".to_string(), 100.0);
    category.evaluations = (0..30)
        .map(|i| Evaluation::new(format!("Evaluation {i}: {}", "界".repeat(60))))
        .collect();
    category
        .evaluations
        .push(Evaluation::new("LAST_EVAL".to_string()));
    course.categories.push(category);
    let mut app = App {
        screen: Screen::ImportPreview,
        import_parsed: Some(course),
        ..Default::default()
    };

    let narrow_limit = import_preview_max_scroll(&app, 50, 30);
    assert!(narrow_limit > import_preview_max_scroll(&app, 120, 30));
    app.import_scroll_preview(i32::MAX, narrow_limit);
    assert_eq!(app.import_preview_scroll, narrow_limit);
    let buffer = render(&app, 50, 30);
    assert!((0..30).any(|y| row_text(&buffer, y).contains("LAST_EVAL")));

    // Resize with the old offset still stored: rendering must clamp it too.
    let resized_limit = import_preview_max_scroll(&app, 160, 60);
    assert!(resized_limit < narrow_limit);
    let buffer = render(&app, 160, 60);
    assert!((0..60).any(|y| row_text(&buffer, y).contains("LAST_EVAL")));
    app.import_scroll_preview(-1, resized_limit);
    assert_eq!(app.import_preview_scroll, resized_limit.saturating_sub(1));

    app.import_scroll_preview(i32::MIN, resized_limit);
    assert_eq!(app.import_preview_scroll, 0);
    assert_eq!(import_preview_max_scroll(&app, 0, 0), 0);
}
