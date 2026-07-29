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

//! Kalk - Your academic dashboard in the terminal
//!
//! A TUI application for managing courses and calculating grades.
//! Supports hierarchical grade calculation: Course -> Categories -> Evaluations.

mod app;
mod clipboard;
mod events;
mod i18n;
mod model;
mod persistence;
mod templates;
mod ui;

use app::App;
use color_eyre::eyre::Result;
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};

/// RAII guard that ensures the terminal is restored even if setup fails.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        // Enable bracketed paste so the AI import wizard can receive whole
        // JSON blobs as `Event::Paste` instead of a stream of key events.
        execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

/// Initialize panic hooks to restore terminal on panic.
/// This is critical for ensuring the terminal is usable after a crash.
fn init_panic_hook() -> Result<()> {
    color_eyre::install()?;

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal before printing panic
        let _ = restore_terminal();
        hook(panic_info);
    }));
    Ok(())
}

/// Setup terminal for TUI rendering.
fn setup_terminal() -> Result<(Terminal<CrosstermBackend<io::Stdout>>, TerminalGuard)> {
    let guard = TerminalGuard::new()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok((terminal, guard))
}

/// Restore terminal to normal state.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), DisableBracketedPaste, LeaveAlternateScreen)?;
    Ok(())
}

/// Main application loop.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if events::handle_events(app)? {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // Initialize panic hooks first (critical for terminal restoration)
    init_panic_hook()?;

    // Setup terminal
    let (mut terminal, _terminal_guard) = setup_terminal()?;

    // Load application state
    let mut app = App::load();

    // Run the application (terminal restoration handled by guard)
    run_app(&mut terminal, &mut app)
}
