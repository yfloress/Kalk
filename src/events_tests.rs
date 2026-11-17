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
use crate::app::InputField;
use crate::model::{Category, Course, Evaluation, Semester};

fn press(app: &mut App, code: KeyCode) {
    handle_key_event(app, KeyEvent::new(code, KeyModifiers::NONE)).unwrap();
}

fn ctrl(app: &mut App, c: char) {
    handle_key_event(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)).unwrap();
}

#[test]
fn cursor_edits_unicode_and_pastes_in_the_middle() {
    let mut app = App {
        screen: Screen::EditingCourse { is_new: true },
        edit_name: "Mát界".into(),
        ..Default::default()
    };
    press(&mut app, KeyCode::Left);
    app.paste_into_focused_field("emática\nignored");
    assert_eq!(app.edit_name, "Mátemática界");
    press(&mut app, KeyCode::Home);
    press(&mut app, KeyCode::Delete);
    assert_eq!(app.edit_name, "átemática界");
    press(&mut app, KeyCode::End);
    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.edit_name, "átemática");
    ctrl(&mut app, 'a');
    press(&mut app, KeyCode::Char('M'));
    assert_eq!(app.edit_name, "Mátemática");
}

#[test]
fn numeric_fields_filter_paste_and_keep_cursor_edits() {
    let mut app = App {
        screen: Screen::EditingCourse { is_new: true },
        input_field: InputField::PassingGrade,
        edit_passing_grade: "50".into(),
        ..Default::default()
    };
    press(&mut app, KeyCode::Left);
    app.paste_into_focused_field("a5.\n99");
    assert_eq!(app.edit_passing_grade, "55.0");
    ctrl(&mut app, 'u');
    assert_eq!(app.edit_passing_grade, "0");
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.input_field, InputField::Credits);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.input_field, InputField::PassingGrade);
}

#[test]
fn specialized_buffers_use_the_same_editor() {
    for screen in [
        Screen::EnteringGlobalGrade,
        Screen::BulkAddEvaluations,
        Screen::EditingSemester { is_new: true },
        Screen::ImportBackup,
    ] {
        let mut app = App {
            screen: screen.clone(),
            input_field: InputField::Grade,
            ..Default::default()
        };
        app.edit_name.clear();
        app.edit_bulk_count.clear();
        app.edit_global_grade.clear();
        app.backup_path.clear();
        app.paste_into_focused_field("12");
        press(&mut app, KeyCode::Home);
        press(&mut app, KeyCode::Char('3'));
        let value = match screen {
            Screen::EnteringGlobalGrade => app.edit_global_grade,
            Screen::BulkAddEvaluations => app.edit_bulk_count,
            Screen::EditingSemester { .. } => app.edit_name,
            _ => app.backup_path,
        };
        assert_eq!(value, "312");
    }
}

#[test]
fn toggles_do_not_edit_text_and_reverse_focus_is_consistent() {
    let mut app = App {
        screen: Screen::EditingCategory { is_new: true },
        show_advanced_rules: true,
        input_field: InputField::WeightedEvals,
        edit_name: "Original".into(),
        ..Default::default()
    };
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.edit_name, "Original");
    press(&mut app, KeyCode::Right);
    assert!(app.edit_weighted_evaluations);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.input_field, InputField::Name);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.input_field, InputField::WeightedEvals);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.input_field, InputField::Name);
}

fn library() -> App {
    let mut app = App::default();
    let mut semester = Semester::new("Second".into(), 1);
    let mut course = Course::new("Álgebra".into(), 55.0);
    let mut category = Category::new("Exams".into(), 100.0);
    category
        .evaluations
        .push(Evaluation::new("Integral final".into()));
    course.categories.push(category);
    semester.courses.push(course);
    app.semesters.push(semester);
    app
}

#[test]
fn fuzzy_search_navigates_to_an_evaluation_in_another_semester() {
    let mut app = library();
    press(&mut app, KeyCode::Char('/'));
    app.search_paste("int fnl");
    assert_eq!(app.search.as_ref().unwrap().entries.len(), 1);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Main);
    assert_eq!(app.selected_semester, 1);
    assert_eq!(app.selected_course, Some(0));
    assert_eq!(app.selected_category, Some(0));
    assert_eq!(app.selected_evaluation, Some(0));
    assert_eq!(app.focus, Focus::Evaluations);
}

#[test]
fn search_normalizes_accents_and_cancel_preserves_navigation() {
    let mut app = library();
    app.screen = Screen::Home;
    press(&mut app, KeyCode::Char('/'));
    app.search_paste("algebra");
    assert!(!app.search.as_ref().unwrap().entries.is_empty());
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Home);
    assert_eq!(app.selected_semester, 0);
    press(&mut app, KeyCode::Char('/'));
    app.search_paste("zzzzzzzz");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Searching);
    assert_eq!(app.selected_semester, 0);
}

#[test]
fn template_search_accounts_for_the_synthetic_import_entry() {
    let mut app = App {
        screen: Screen::SelectingTemplate,
        ..Default::default()
    };
    let first = app.all_templates()[0].name.clone();
    press(&mut app, KeyCode::Char('/'));
    app.search_paste(&first);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::SelectingTemplate);
    assert_eq!(app.selected_template, 1);
}

#[test]
fn slash_remains_text_inside_a_form() {
    let mut app = App {
        screen: Screen::EditingSemester { is_new: true },
        ..Default::default()
    };
    app.edit_name.clear();
    press(&mut app, KeyCode::Char('/'));
    assert_eq!(app.edit_name, "/");
    assert!(app.search.is_none());
}

#[test]
fn json_editor_retains_errors_can_undo_and_revalidate_without_saving() {
    let mut app = App::default();
    app.start_ai_import();
    app.import_goto_paste();
    let json = r#"{"schema_version":1,"name":"Math","categories":[{"name":"Exams","weight":100}]}"#;
    app.import_handle_paste(json[..json.len() - 1].into());
    assert_eq!(app.screen, Screen::ImportPaste);
    assert!(app.import_paste_error.is_some());
    assert_eq!(app.import_editor.lines().join("\n"), json[..json.len() - 1]);
    press(&mut app, KeyCode::Char('}'));
    ctrl(&mut app, 'u'); // textarea's native undo
    assert_eq!(app.import_editor.lines().join("\n"), json[..json.len() - 1]);
    ctrl(&mut app, 'r'); // native redo
    press(&mut app, KeyCode::F(2));
    assert_eq!(app.screen, Screen::ImportPreview);
    assert!(app.courses().is_empty());
    app.import_back_to_paste();
    assert_eq!(app.import_editor.lines().join("\n"), json);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.import_editor.lines().len(), 2);
    press(&mut app, KeyCode::F(3));
    assert_eq!(app.screen, Screen::ImportPrompt);
}

#[test]
fn a_second_complete_paste_replaces_the_buffer_and_ctrl_z_undoes() {
    let mut app = App::default();
    app.start_ai_import();
    app.import_goto_paste();
    app.import_handle_paste("{ not json".into());
    assert!(app.import_paste_error.is_some());
    let json = r#"{"schema_version":1,"name":"Math","categories":[{"name":"Exams","weight":100}]}"#;
    app.import_handle_paste(json.into());
    assert_eq!(app.screen, Screen::ImportPreview);
    assert_eq!(app.import_editor.lines().join("\n"), json);

    app.import_back_to_paste();
    press(&mut app, KeyCode::Char('x'));
    ctrl(&mut app, 'z');
    assert_eq!(app.import_editor.lines().join("\n"), json);
    ctrl(&mut app, 'y');
    assert!(app.import_editor.lines().join("\n").ends_with('x'));
}

#[test]
fn animation_setting_is_included_in_config_and_toggles_both_directions() {
    let mut app = App {
        selected_setting: 3,
        ..Default::default()
    };
    assert!(app.animations);
    app.toggle_current_setting();
    assert!(!app.config().animations);
    app.toggle_current_setting_reverse();
    assert!(app.config().animations);
    app.next_setting();
    assert_eq!(app.selected_setting, 3);
}
