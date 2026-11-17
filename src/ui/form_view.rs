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

//! Full-height form rows in a native scroll view; help and keys stay pinned.

use super::{
    helpers::{input_widget, toggle_widget},
    keyhints::render_hint,
    surfaces::popup_block,
    theme::theme,
};
use crate::app::{App, InputField};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect, Size},
    style::Style,
    text::Line,
    widgets::{Clear, Paragraph, Wrap},
};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

pub(super) enum FormRow<'a> {
    Input(InputField, &'a str, &'a str),
    Toggle(InputField, &'a str, String),
    Note(Line<'a>),
}

impl FormRow<'_> {
    fn height(&self) -> u16 {
        if matches!(self, Self::Note(_)) { 1 } else { 3 }
    }
    fn field(&self) -> Option<InputField> {
        match self {
            Self::Input(field, ..) | Self::Toggle(field, ..) => Some(*field),
            _ => None,
        }
    }
}

pub(super) struct FormView<'a> {
    pub title: &'a str,
    pub width: u16,
    pub rows: Vec<FormRow<'a>>,
    pub help: String,
    pub hints: &'a [(&'a str, &'a str)],
}

impl FormView<'_> {
    pub fn render(self, frame: &mut Frame, app: &App) {
        let t = theme();
        let content_height = self.rows.iter().map(FormRow::height).sum::<u16>();
        let help = Paragraph::new(self.help)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(t.text_secondary));
        let width = self.width.min(frame.area().width);
        let help_height = help.line_count(width.saturating_sub(4)).min(3) as u16;
        let area = frame.area().centered(
            Constraint::Length(width),
            Constraint::Length(content_height.saturating_add(help_height + 3)),
        );
        frame.render_widget(Clear, area);
        let block = popup_block(format!(" {} ", self.title));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let help_height = help_height.min(inner.height.saturating_sub(4));
        let parts = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(help_height),
            Constraint::Length(1),
        ])
        .split(inner);
        frame.render_widget(help, parts[1]);
        render_hint(frame, self.hints, parts[2]);
        let viewport = parts[0];
        if viewport.is_empty() {
            return;
        }
        let scrolling = content_height > viewport.height;
        let content_width = viewport.width.saturating_sub(u16::from(scrolling));
        if content_width == 0 {
            return;
        }
        let mut view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);
        let mut y = 0;
        let mut focus_bottom = 0;
        let mut cursor = None;
        for row in self.rows {
            let height = row.height();
            let active = row.field() == Some(app.input_field);
            if active {
                focus_bottom = y + height;
            }
            let rect = Rect::new(0, y, content_width, height);
            match row {
                FormRow::Input(_, label, value) => {
                    let (widget, x) = input_widget(app, label, value, active, content_width);
                    view.render_widget(widget, rect);
                    if active && content_width > 2 {
                        cursor = Some((x + 1, y + 1));
                    }
                }
                FormRow::Toggle(_, label, value) => {
                    view.render_widget(toggle_widget(label, &value, active), rect)
                }
                FormRow::Note(line) => view.render_widget(line, rect),
            }
            y += height;
        }
        let offset = focus_bottom
            .saturating_sub(viewport.height)
            .min(content_height.saturating_sub(viewport.height));
        let mut state = ScrollViewState::with_offset(Position::new(0, offset));
        frame.render_stateful_widget(view, viewport, &mut state);
        if let Some((x, y)) = cursor
            && y >= state.offset().y
            && y - state.offset().y < viewport.height
        {
            frame.set_cursor_position((viewport.x + x, viewport.y + y - state.offset().y));
        }
    }
}
