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

//! Fuzzy library/template search with app-owned selection and navigation.

use super::{App, Focus, Screen, text_edit::key_request};
use crossterm::event::KeyEvent;
use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};
use tui_input::Input;

#[derive(Debug, Clone)]
pub enum SearchTarget {
    Course(usize, usize),
    Category(usize, usize, usize),
    Evaluation(usize, usize, usize, usize),
    Template(usize),
}

#[derive(Debug, Clone)]
pub struct SearchEntry {
    pub label: String,
    pub target: SearchTarget,
}

impl AsRef<str> for SearchEntry {
    fn as_ref(&self) -> &str {
        &self.label
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub origin: Screen,
    pub input: Input,
    pub entries: Vec<SearchEntry>,
    pub selected: usize,
}

impl App {
    pub fn start_search(&mut self) {
        if !matches!(
            self.screen,
            Screen::Main | Screen::Home | Screen::SelectingTemplate
        ) {
            return;
        }
        self.search = Some(SearchState {
            origin: self.screen.clone(),
            input: Input::default(),
            entries: Vec::new(),
            selected: 0,
        });
        self.screen = Screen::Searching;
        self.refresh_search();
    }

    fn search_candidates(&self, templates: bool) -> Vec<SearchEntry> {
        if templates {
            return self
                .all_templates()
                .into_iter()
                .enumerate()
                .map(|(i, template)| SearchEntry {
                    label: template.name.clone(),
                    target: SearchTarget::Template(i + 1),
                })
                .collect();
        }
        let mut entries = Vec::new();
        for (si, semester) in self.semesters.iter().enumerate() {
            for (ci, course) in semester.courses.iter().enumerate() {
                let prefix = format!("{} / {}", semester.name, course.name);
                entries.push(SearchEntry {
                    label: prefix.clone(),
                    target: SearchTarget::Course(si, ci),
                });
                for (ki, category) in course.categories.iter().enumerate() {
                    let prefix = format!("{} / {}", prefix, category.name);
                    entries.push(SearchEntry {
                        label: prefix.clone(),
                        target: SearchTarget::Category(si, ci, ki),
                    });
                    for (ei, evaluation) in category.evaluations.iter().enumerate() {
                        entries.push(SearchEntry {
                            label: format!("{} / {}", prefix, evaluation.name),
                            target: SearchTarget::Evaluation(si, ci, ki, ei),
                        });
                    }
                }
            }
        }
        entries
    }

    fn refresh_search(&mut self) {
        let Some(search) = &self.search else {
            return;
        };
        let query = search.input.value().to_string();
        let candidates = self.search_candidates(search.origin == Screen::SelectingTemplate);
        let entries = if query.is_empty() {
            candidates
        } else {
            let mut matcher = Matcher::new(Config::DEFAULT);
            Pattern::parse(&query, CaseMatching::Ignore, Normalization::Smart)
                .match_list(candidates, &mut matcher)
                .into_iter()
                .map(|(entry, _)| entry)
                .collect()
        };
        if let Some(search) = &mut self.search {
            search.entries = entries;
            search.selected = 0;
        }
    }

    pub fn search_edit(&mut self, key: KeyEvent) {
        if let Some(request) = key_request(key)
            && let Some(search) = &mut self.search
        {
            let before = search.input.value().to_string();
            search.input.handle(request);
            if search.input.value() != before {
                self.refresh_search();
            }
        }
    }

    pub fn search_paste(&mut self, text: &str) {
        if let Some(search) = &mut self.search {
            for c in text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .filter(|c| !c.is_control())
            {
                search.input.handle(tui_input::InputRequest::InsertChar(c));
            }
            self.refresh_search();
        }
    }

    pub fn search_move(&mut self, delta: i32) {
        if let Some(search) = &mut self.search {
            search.selected = search
                .selected
                .saturating_add_signed(delta as isize)
                .min(search.entries.len().saturating_sub(1));
        }
    }

    pub fn cancel_search(&mut self) {
        if let Some(search) = self.search.take() {
            self.screen = search.origin;
        }
    }

    pub fn confirm_search(&mut self) {
        let Some(search) = &self.search else {
            return;
        };
        let Some(entry) = search.entries.get(search.selected) else {
            return;
        };
        let target = entry.target.clone();
        self.cancel_search();
        if let SearchTarget::Template(index) = target {
            self.selected_template = index;
            return;
        }
        let (semester, course) = match target {
            SearchTarget::Course(si, ci)
            | SearchTarget::Category(si, ci, _)
            | SearchTarget::Evaluation(si, ci, _, _) => (si, ci),
            SearchTarget::Template(_) => unreachable!(),
        };
        self.selected_semester = semester;
        self.selected_course = Some(course);
        self.reset_category_selection();
        match target {
            SearchTarget::Category(_, _, category) => {
                self.selected_category = Some(category);
                self.reset_evaluation_selection();
                self.focus = Focus::Categories;
            }
            SearchTarget::Evaluation(_, _, category, evaluation) => {
                self.selected_category = Some(category);
                self.selected_evaluation = Some(evaluation);
                self.focus = Focus::Evaluations;
            }
            _ => self.focus = Focus::Courses,
        }
        self.screen = Screen::Main;
    }
}
