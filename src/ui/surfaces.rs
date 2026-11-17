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

//! Shared floating surfaces built from Ratatui's Block, Padding and Shadow.

use ratatui::{
    layout::Offset,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Padding, Shadow},
};

use super::theme::theme;

pub(super) fn popup_block<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    let t = theme();
    Block::default()
        .title(
            title.into().style(
                Style::default()
                    .fg(t.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .borders(Borders::ALL)
        .border_type(t.border_type)
        .border_style(Style::default().fg(t.popup_border))
        .style(Style::default().fg(t.text_primary).bg(t.popup_bg))
        .padding(Padding::horizontal(1))
        .shadow(
            Shadow::overlay()
                .style(Style::default().bg(t.shadow_bg))
                .offset(Offset::new(2, 1)),
        )
}
