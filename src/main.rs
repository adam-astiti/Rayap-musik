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

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
rayap-musik — a local-first terminal music player for Linux

USAGE:
    rayap-musik <music-directory>
    rayap-musik --help
    rayap-musik --version

ARGUMENTS:
    <music-directory>    Root directory to browse. A leading `~` or `~/` is
                         expanded using $HOME.

OPTIONS:
    -h, --help           Print this help and exit
    -V, --version        Print version and exit

SUPPORTED FORMATS:
    .mp3 .flac .wav .ogg .oga .opus .m4a .mp4 .aac
    (M4A/AAC is fully supported; Opus is a known gap)

KEYS:
    Up / Down       Move selection in the active panel
    Tab             Switch focus between Library and Queue
    Enter           Open folder / play selected track or queue item
    Backspace       Go to parent directory (Library)
    Space           Play / Pause
    n               Next track
    p               Previous / restart (restart if played > 3s)
    + / =           Volume up (5% step)
    -               Volume down (5% step)
    q               Quit and stop playback

EXIT CODES:
    0    Success (including --help and --version)
    1    Runtime error (e.g. terminal initialization failure)
    2    Usage error (missing, unknown, or invalid directory)
";

fn print_help() {
    println!("{HELP}");
}

fn print_version() {
    println!("rayap-musik {VERSION}");
}

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

    let mut positional: Vec<String> = Vec::new();
    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                eprintln!("rayap-musik: unknown option '{other}'");
                eprintln!("Try 'rayap-musik --help' for more information.");
                return Err(EXIT_USAGE);
            }
            other => positional.push(other.to_string()),
        }
    }

    let raw_root = match positional.as_slice() {
        [] => {
            eprintln!("rayap-musik: missing root directory argument");
            eprintln!("Usage: rayap-musik <music-directory>");
            eprintln!("Try 'rayap-musik --help' for more information.");
            return Err(EXIT_USAGE);
        }
        [root] => root.clone(),
        _ => {
            eprintln!("rayap-musik: too many arguments");
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
