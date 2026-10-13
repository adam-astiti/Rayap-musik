# Rayap Musik

Local-first, keyboard-driven **terminal music player and library manager** for
Linux, written in Rust. It indexes a local music directory into an embedded
database, groups the collection into Songs / Albums / Artists, provides global
search, manages playlists (with M3U import/export), and plays audio — all
offline and without a server.

> Status: **Milestone 7 — release review.** See `upgrade/prd_upgrade.md` for the
> v0.2 requirements and `docs/QA.md` for the v0.1 baseline.

## Features

- Persistent library root and preferences (a commented TOML config file).
- Embedded SQLite index; automatic filesystem watcher plus manual rescan.
- Songs, Albums, Artists, and Playlists views with deterministic sorting.
- Global, case-insensitive search across songs, albums, artists, and playlists.
- Play Now, Play Next, Add to Queue, and queue removal.
- Shuffle (active track preserved) and three repeat modes (Off / All / One).
- Playlists: create, rename, delete, duplicate, reorder, sort, add; M3U
  import/export with a summary of imported/skipped/missing paths.
- Four themes (Rust & Walnut default, Catppuccin, Dracula, Nord), Play Mode,
  an animated ASCII termite mascot + equalizer, and an in-app help screen.
- Read-only access to audio files; symlinks are ignored.

## Requirements

- Linux with an interactive terminal.
- Rust toolchain (stable). Built and tested with Rust 1.85.
- Audio backend build dependencies (ALSA + `pkg-config`):
  - Debian/Ubuntu: `sudo apt install libasound2-dev pkg-config build-essential`

## Build

```sh
cargo build --release --locked
```

The binary is produced at `target/release/rayap-musik`.

## Install

### Option A — Debian package (.deb)

```sh
packaging/build-deb.sh
sudo apt install ./dist/rayap-musik_*.deb
```

### Option B — Install with cargo (no root required)

```sh
cargo install --path . --root "$HOME/.local" --locked
```

> Always pass `--locked`: `Cargo.lock` pins versions compatible with Rust 1.85.

## Usage

```sh
rayap-musik                  # uses the root stored in the config file
rayap-musik ~/Music          # sets the root and saves it for next time
rayap-musik --help
rayap-musik --version
```

- A leading `~` or `~/` is expanded using `$HOME`.
- The music directory is remembered in the config file, so you usually only
  pass it once.

### Configuration

Settings live in a commented TOML file at
`$XDG_CONFIG_HOME/rayap-musik/config.toml` (default `~/.config/rayap-musik/`).
The database lives in `$XDG_DATA_HOME/rayap-musik/library.db` (default
`~/.local/share/rayap-musik/`). The config file is human-editable and preserves
your comments and unknown keys; a malformed value falls back to its default
without discarding the rest.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including `--help` and `--version`) |
| `1` | Runtime error (e.g. terminal initialization failure) |
| `2` | Usage error (missing, unknown, or invalid directory) |

## Supported formats

`.mp3`, `.flac`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.mp4`, `.aac`.

- Decoding is handled by Symphonia through Rodio. MP3, FLAC, WAV, OGG Vorbis,
  and M4A/AAC are decoded by the pinned stack.
- **Opus is a known gap:** the pinned Rodio/Symphonia stack does not decode
  Opus, so `.opus`/`.oga` files are listed but skipped on playback.
- Non-audio files and symlinks are ignored.
- Metadata fallback: the title uses the file stem; album and artist fall back to
  the containing folder's name when tags are missing.

## Keys

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection |
| `Tab` | Switch focus (Library / Queue) |
| `1` / `2` / `3` / `4` | Songs / Albums / Artists / Playlists |
| `/` | Global search (type to filter, `Esc` to leave) |
| `Enter` | Play Now / open a playlist |
| `a` | Add selection to the end of the queue |
| `N` | Play Next (insert after the active track) |
| `d` | Remove the selected queue entry (Queue view) |
| `s` | Toggle shuffle |
| `R` | Cycle repeat (Off → All → One → Off) |
| `m` | Toggle Play Mode |
| `T` | Cycle theme |
| `?` | Toggle help |
| `c` / `e` / `D` / `x` | Create / rename / duplicate / delete playlist |
| `P` | Add the selection to a playlist |
| `I` / `E` | Import / export an M3U playlist |
| `J` / `K` / `S` | Move track down / up / sort (inside a playlist) |
| `Esc` / `Backspace` | Back (leave a playlist, cancel a picker) |
| `Space` | Play / Pause |
| `n` / `p` | Next / previous track (previous restarts if played > 3s) |
| `+` / `-` | Volume up / down (5% step) |
| `r` | Rescan the library |
| `q` | Quit and stop playback |

## Manual test plan

Run the app against a folder of real audio files and verify:

1. **Formats** — play a real file of each type: MP3, FLAC, WAV, OGG Vorbis, and
   M4A/AAC. (Opus is a known gap.)
2. **Persistence** — set a root, quit, relaunch; the root, index, playlists, and
   volume are restored without auto-play.
3. **Watcher** — add, rename, and delete files while the app runs; the library
   updates without restarting.
4. **Views / search** — switch Songs/Albums/Artists/Playlists; search returns
   typed results.
5. **Queue** — Play Now/Next/Add/remove; shuffle on/off keeps the active track;
   repeat Off/All/One behave correctly.
6. **Playlists** — create/rename/duplicate/delete, reorder, sort, and M3U
   import/export.
7. **Play Mode / themes** — toggle `m`, cycle `T`, open `?` help.
8. **Play Mode** — toggle `m`; the termite mascot animates and the equalizer
   bars move while playing and freeze when paused.
9. **Error handling** — corrupt/unavailable files are skipped without crashing.
10. **Terminal restore** — quit normally and via handled errors; the terminal is
    always restored.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

Automated tests cover configuration, database persistence/migrations, scanning,
catalog grouping/sorting/search, queue operations (including shuffle/repeat),
M3U parsing, theme selection, and keyboard mapping. Tests that require an audio
device or a real terminal are manual (above).

## Project structure

```text
src/
  main.rs      lifecycle, event loop, terminal setup/cleanup, CLI parsing
  lib.rs       library crate root (enables integration testing)
  app.rs       application state and decisions
  input.rs     keyboard -> action mapping
  ui.rs        rendering (views, Play Mode, help, theme-aware)
  library.rs   filesystem, formats, metadata, scanning
  catalog.rs   grouping/sorting of the index + global search
  indexer.rs   background scan worker + filesystem watcher
  db.rs        SQLite persistence (index + playlists) and migrations
  config.rs    commented TOML preferences
  m3u.rs       M3U import/export
  theme.rs     color themes
  player.rs    audio playback
  models.rs    shared types (Track, Queue, Action, ...)
docs/
  PRD.md       v0.1 product requirements (baseline)
  QA.md        v0.1 quality assurance report
upgrade/
  prd_upgrade.md  v0.2 requirements
packaging/
  build-deb.sh builds a Debian package into dist/
tests/
  library_tests.rs  queue_tests.rs  cli_tests.rs  config_tests.rs
  db_tests.rs  m3u_tests.rs  format_verify.rs
```

## Known limitations

- **Opus is not decoded** by the pinned Rodio/Symphonia stack.
- **No image rendering**: Play Mode uses an animated ASCII equalizer + termite
  mascot instead of real album artwork.
- The filesystem watcher triggers a full rescan (debounced); very large
  libraries may see a brief pause while metadata is read in the background.
- M3U import adds found paths to the index; tracks outside the library root are
  marked unavailable on the next rescan.
