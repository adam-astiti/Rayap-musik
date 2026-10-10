//! Rayap Musik — local-first terminal music player for Linux.
//!
//! Milestone 1: application lifecycle and UI shell. No library scanning or
//! audio playback yet.

mod app;
mod input;
mod models;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

type Backend = CrosstermBackend<io::Stdout>;

/// Owns the terminal and restores it on drop, so early returns and panics
/// still leave the terminal in a usable state.
struct TerminalGuard {
    terminal: Terminal<Backend>,
}

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(Backend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn main() -> io::Result<()> {
    let root = std::env::args().nth(1);

    let mut guard = TerminalGuard::enter()?;
    let app = App::new(root);
    let result = run(&mut guard.terminal, app);
    drop(guard);
    result
}

fn run(terminal: &mut Terminal<Backend>, mut app: App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(EVENT_POLL_INTERVAL)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::map_key(key) {
                        app.handle_action(action);
                    }
                }
                Event::Resize(_, _) => app.handle_action(models::Action::Resize),
                _ => {}
            }
        }
    }
    Ok(())
}
