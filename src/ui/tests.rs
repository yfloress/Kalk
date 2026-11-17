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
use crate::model::{Category, CategoryRules, Evaluation};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, widgets::Widget};

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

#[test]
fn popup_surfaces_have_padding_and_a_clipped_native_shadow() {
    let t = theme();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 12));
    let area = Rect::new(3, 2, 16, 7);
    let popup = surfaces::popup_block("Dialog");
    assert_eq!(popup.inner(area).x, area.x + 2);
    popup.render(area, &mut buffer);
    assert_eq!(buffer[(5, 4)].bg, t.popup_bg);
    assert_eq!(buffer[(19, 5)].bg, t.shadow_bg);
    assert_eq!(buffer[(6, 9)].bg, t.shadow_bg);
    assert_eq!(buffer[(0, 0)], ratatui::buffer::Cell::default());

    for width in 0..5 {
        for height in 0..5 {
            let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
            surfaces::popup_block("Dialog").render(buffer.area, &mut buffer);
        }
    }
}

#[test]
fn settings_show_aligned_values_and_keep_the_cancel_hint() {
    for language in [Language::English, Language::Spanish] {
        for selected_setting in 0..4 {
            let app = App {
                screen: Screen::Settings,
                language,
                selected_setting,
                use_nerd_fonts: false,
                mouse_scroll: true,
                ..Default::default()
            };
            let buffer = render(&app, 100, 28);
            let lines: Vec<String> = (0..28).map(|y| row_text(&buffer, y)).collect();
            let text = lines.join("\n");
            assert!(text.contains(app.messages().settings_nerd_fonts));
            assert!(text.contains(app.messages().settings_mouse_scroll));
            assert!(text.contains(app.messages().settings_animations));
            assert!(text.contains("[ ]"));
            assert!(text.contains("[x]"));
            assert!(text.contains(app.language.display_name()));
            assert!(text.contains("Esc"));

            // The native table keeps the selected setting in view on resize.
            let buffer = render(&app, 40, 12);
            assert!((0..12).any(|y| row_text(&buffer, y).contains("Esc")));
        }
    }
}

#[test]
fn home_keeps_progress_numbers_and_course_grades_with_native_widgets() {
    let mut course = Course::new("Mechanics".to_string(), 55.0);
    let mut category = Category::new("Exams".to_string(), 100.0);
    category.evaluations = (0..4)
        .map(|i| Evaluation::new(format!("Exam {i}")))
        .collect();
    category.evaluations[0].grade = Some(80.0);
    course.categories.push(category);
    let mut app = App {
        screen: Screen::Home,
        ..Default::default()
    };
    app.courses_mut().push(course);
    for language in [Language::English, Language::Spanish] {
        app.language = language;
        let buffer = render(&app, 120, 36);
        let text = (0..36)
            .map(|y| row_text(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("1/4"));
        assert!(text.contains("Mechanics"));
        assert!(text.contains("20.0"));
        assert!(text.contains("95.0"));
    }
}

#[test]
fn category_form_renders_every_combination_of_advanced_rules() {
    for language in [Language::English, Language::Spanish] {
        for mask in 0..8 {
            let rules = CategoryRules {
                minimum_average: (mask & 1 != 0).then_some(50.0),
                minimum_per_evaluation: (mask & 2 != 0).then_some(30.0),
                minimum_one_eval: (mask & 4 != 0).then_some(60.0),
                cap_final_grade: Some(54.0),
                round_before_weighting: true,
                weighted_evaluations: true,
                ..Default::default()
            };
            let mut course = Course::new("Course".to_string(), 55.0);
            course.categories.push(Category::with_rules(
                "Rules".to_string(),
                100.0,
                Vec::new(),
                rules,
            ));
            let mut app = App {
                language,
                selected_course: Some(0),
                selected_category: Some(0),
                use_nerd_fonts: false,
                ..Default::default()
            };
            app.courses_mut().push(course);
            app.start_edit_category();
            assert!(app.show_advanced_rules);
            assert_eq!(app.edit_cap_final_grade, "54");

            for height in [12, 24, 48, 80] {
                let buffer = render(&app, 100, height);
                if height == 80 {
                    let text = (0..height)
                        .map(|y| row_text(&buffer, y))
                        .collect::<Vec<_>>()
                        .join("\n");
                    assert!(text.contains(app.messages().cap_final_grade));
                    assert!(text.contains(app.messages().round_before_weighting));
                    assert!(text.contains(app.messages().weighted_evaluations));
                    assert!(text.contains("Enter"));
                    assert!(text.contains("Esc"));
                }
            }
        }
    }
}

#[test]
fn new_category_can_expand_and_collapse_advanced_rules() {
    let mut app = App {
        selected_course: Some(0),
        ..Default::default()
    };
    app.courses_mut()
        .push(Course::new("Course".to_string(), 55.0));
    app.start_new_category();
    for _ in 0..4 {
        render(&app, 80, 24);
        app.toggle_advanced_rules();
    }
}

#[test]
fn long_category_forms_keep_the_focused_field_and_hints_visible() {
    use crate::app::InputField;
    for language in [Language::English, Language::Spanish] {
        let mut app = App {
            screen: Screen::EditingCategory { is_new: true },
            language,
            show_advanced_rules: true,
            edit_min_average: "50".into(),
            edit_min_per_eval: "30".into(),
            edit_min_one_eval: "60".into(),
            edit_cap_final_grade: "54".into(),
            ..Default::default()
        };
        for height in [12, 24, 80] {
            for (field, label) in [
                (InputField::Name, app.messages().name),
                (InputField::MinimumAverage, app.messages().minimum_average),
                (InputField::CapFinalGrade, app.messages().cap_final_grade),
                (
                    InputField::WeightedEvals,
                    app.messages().weighted_evaluations,
                ),
            ] {
                app.input_field = field;
                let buffer = render(&app, 100, height);
                let text = (0..height)
                    .map(|y| row_text(&buffer, y))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(text.contains(label), "missing {label} at {height}");
                assert!(text.contains("Esc"));
                assert!(text.contains("Enter"), "{height} {field:?}\n{text}");
                assert!(
                    buffer
                        .content
                        .iter()
                        .any(|cell| cell.fg == theme().input_active)
                );
            }
        }
    }
}

#[test]
fn edited_input_scrolls_horizontally_and_positions_a_real_cursor() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App {
        screen: Screen::EditingCourse { is_new: true },
        edit_name: "界".repeat(70),
        ..Default::default()
    };
    app.edit_focused_text(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    let completed = terminal.draw(|frame| draw(frame, &app)).unwrap();
    assert!(
        !completed
            .buffer
            .content
            .iter()
            .any(|cell| cell.symbol() == "|")
    );
    let cursor = terminal.get_cursor_position().unwrap();
    assert!(cursor.x < 60 && cursor.y < 20);
}

#[test]
fn search_and_json_editor_render_at_small_sizes_and_show_native_hints() {
    for language in [Language::English, Language::Spanish] {
        let mut app = App {
            language,
            ..Default::default()
        };
        app.start_search();
        for (width, height) in [(0, 0), (1, 1), (12, 5), (50, 16)] {
            render(&app, width, height);
        }
        let buffer = render(&app, 90, 24);
        let text = (0..24)
            .map(|y| row_text(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains(app.messages().search_empty));
        assert!(text.contains("Esc"));
        app.cancel_search();
        app.start_ai_import();
        app.import_goto_paste();
        app.import_handle_paste("{broken}".into());
        for (width, height) in [(0, 0), (1, 1), (12, 5), (50, 16)] {
            render(&app, width, height);
        }
        let buffer = render(&app, 100, 30);
        let text = (0..30)
            .map(|y| row_text(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("{broken}"));
        assert!(text.contains("F2"));
        assert!(text.contains("F3"));
        assert!(text.contains("Esc"));
    }
}
