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

use rayap_musik::{
    app::App,
    config::{self, Config},
    db::Db,
    input, library,
    models::Action,
    ui,
};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Exit code for usage / argument errors.
const EXIT_USAGE: i32 = 2;
/// Exit code for runtime errors.
const EXIT_RUNTIME: i32 = 1;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
rayap-musik — a local-first terminal music player for Linux

USAGE:
    rayap-musik [<music-directory>]
    rayap-musik --help
    rayap-musik --version

ARGUMENTS:
    <music-directory>    Music directory to browse. A leading `~` or `~/` is
                         expanded using $HOME. When provided, it is saved as
                         the library root for future launches. If omitted, the
                         root stored in the configuration file is used.

OPTIONS:
    -h, --help           Print this help and exit
    -V, --version        Print version and exit

SUPPORTED FORMATS:
    .mp3 .flac .wav .ogg .oga .m4a .mp4 .aac
    (Opus is not decoded by the current stack and is a known gap)

CONFIGURATION:
    Settings are stored in a commented TOML file. Its location follows the XDG
    spec: $XDG_CONFIG_HOME/rayap-musik/config.toml (default ~/.config/...).

KEYS:
    Up / Down       Move selection in the active panel
    Tab             Switch focus between Library and Queue
    1 / 2 / 3 / 4   Switch view: Songs / Albums / Artists / Playlists
    /               Global search (type to filter, Esc to leave)
    Enter           Play Now / open playlist (Playlists view)
    a               Add selection to the end of the queue
    N               Play Next: insert selection after the active track
    d               Remove the selected queue entry (Queue view)
    s               Toggle shuffle
    R               Cycle repeat: Off -> All -> One -> Off
    m               Toggle Play Mode (full screen)
    T               Cycle theme (rust-walnut/catppuccin/dracula/nord)
    ?               Toggle this help
    c               Create a new playlist
    e               Rename the selected playlist
    D               Duplicate the selected playlist
    x               Delete playlist / remove track (press x twice to confirm)
    P               Add the selection to a playlist
    I / E           Import / export an M3U playlist
    J / K           Move track down/up inside a playlist
    S               Sort the open playlist by artist/album/track
    Esc / Backspace Back (leave a playlist, cancel a picker)
    Space           Play / Pause
    n               Next track
    p               Previous / restart (restart if played > 3s)
    + / =           Volume up (5% step)
    -               Volume down (5% step)
    r               Rescan library (rebuild index)
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

    let mut config = Config::load();

    if positional.len() > 1 {
        eprintln!("rayap-musik: too many arguments");
        eprintln!("Usage: rayap-musik [<music-directory>]");
        eprintln!("Try 'rayap-musik --help' for more information.");
        return Err(EXIT_USAGE);
    }

    let raw_root = match positional.first() {
        Some(root) => Some(root.clone()),
        None => config.root.clone(),
    };

    let raw_root = match raw_root {
        Some(raw_root) => raw_root,
        None => {
            eprintln!("rayap-musik: no music directory configured");
            eprintln!("Run 'rayap-musik <music-directory>' to set your library root.");
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

    config.root = Some(raw_root);
    if let Err(error) = config.save() {
        eprintln!("rayap-musik: warning: could not save configuration: {error}");
    }

    let db_path = Path::new(&config.advanced.data_dir).join(config::DB_FILE);
    let db = match Db::open(&db_path) {
        Ok(db) => Some(db),
        Err(error) => {
            eprintln!("rayap-musik: warning: database unavailable: {error}");
            None
        }
    };

    let mut guard = match TerminalGuard::enter() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("rayap-musik: failed to initialize terminal: {error}");
            return Err(EXIT_RUNTIME);
        }
    };

    let mut app = App::new(root, config.volume(), &config.theme, db);
    let result = run(&mut guard.terminal, &mut app);

    config.volume = app.volume;
    config.theme = app.theme.name.clone();
    if let Err(error) = config.save() {
        eprintln!("rayap-musik: warning: could not save configuration: {error}");
    }

    drop(guard);

    if let Err(error) = result {
        eprintln!("rayap-musik: {error}");
        return Err(EXIT_RUNTIME);
    }
    Ok(())
}

fn run(terminal: &mut Terminal<Backend>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        app.tick();
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(EVENT_POLL_INTERVAL)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::map_key(key, app.text_input_active()) {
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
