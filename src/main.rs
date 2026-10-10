//! Rayap Musik — local-first terminal music player for Linux.

use std::io::{self, stdout};
use std::path::Path;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rayap_musik::{app::App, input, library, models::Action, ui};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Exit code for usage / argument errors.
const EXIT_USAGE: i32 = 2;
/// Exit code for runtime errors.
const EXIT_RUNTIME: i32 = 1;

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

fn main() {
    if let Err(code) = start() {
        std::process::exit(code);
    }
}

fn start() -> Result<(), i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let raw_root = match args.first() {
        Some(arg) => arg.clone(),
        None => {
            eprintln!("rayap-musik: missing root directory argument");
            eprintln!("Usage: rayap-musik <music-directory>");
            return Err(EXIT_USAGE);
        }
    };

    let expanded = library::expand_tilde(&raw_root);
    let root = match library::validate_root(Path::new(&expanded)) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("rayap-musik: {error}");
            return Err(EXIT_USAGE);
        }
    };

    let mut guard = match TerminalGuard::enter() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("rayap-musik: failed to initialize terminal: {error}");
            return Err(EXIT_RUNTIME);
        }
    };

    let app = App::new(root);
    let result = run(&mut guard.terminal, app);
    drop(guard);

    if let Err(error) = result {
        eprintln!("rayap-musik: {error}");
        return Err(EXIT_RUNTIME);
    }
    Ok(())
}

fn run(terminal: &mut Terminal<Backend>, mut app: App) -> io::Result<()> {
    while !app.should_quit {
        app.tick();
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(EVENT_POLL_INTERVAL)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::map_key(key) {
                        app.handle_action(action);
                    }
                }
                Event::Resize(_, _) => app.handle_action(Action::Resize),
                _ => {}
            }
        }
    }
    Ok(())
}
