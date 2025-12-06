//! Kalk - Your academic dashboard in the terminal
//!
//! A TUI application for managing courses and calculating grades.
//! Supports hierarchical grade calculation: Course -> Categories -> Evaluations.

mod app;
mod events;
mod i18n;
mod model;
mod persistence;
mod templates;
mod ui;

use app::App;
use color_eyre::eyre::Result;
use crossterm::{
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
        execute!(stdout(), EnterAlternateScreen)?;
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
    execute!(stdout(), LeaveAlternateScreen)?;
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

    // Run the application
    let result = run_app(&mut terminal, &mut app);

    // Propagate any errors from the main loop (terminal restoration handled by guard)
    result
}
