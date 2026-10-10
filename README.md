# Rayap Musik

Local-first terminal music player for Linux, written in Rust. It browses a
local music directory, reads audio metadata, plays supported files, and manages
an in-memory playback queue — all from the keyboard, without a server or
network access.

> Status: **Milestone 5 — integration and hardening.** All MVP features are
> implemented. See `docs/PRD.md` for the full product requirements.

## Features

- Manual folder browsing with metadata preview (title, artist, album, duration).
- Deterministic sorting: folders first, then tracks by track number, then name.
- Playback with play/pause, next, previous/restart, and volume control.
- Recursive queue built from the selected track's folder subtree.
- Auto-next, stop at end of queue, and automatic skipping of files that fail.
- Real-time progress, position, and total duration when available.
- Safe behavior for empty, unreadable, or corrupt inputs.
- Terminal is always restored on exit.

## Requirements

- Linux with an interactive terminal.
- Rust toolchain (stable). Built and tested with Rust 1.85.
- Audio backend build dependencies (ALSA + `pkg-config`):
  - Debian/Ubuntu: `sudo apt install libasound2-dev pkg-config build-essential`

## Build

```sh
cargo build --release
```

The binary is produced at `target/release/rayap-musik`.

## Install

### Option A — Debian package (.deb)

Build the package, then install it with `apt` (this also resolves the ALSA
runtime dependency):

```sh
packaging/build-deb.sh
sudo apt install ./dist/rayap-musik_*.deb
```

- Installs the binary to `/usr/bin/rayap-musik`.
- Installs documentation to `/usr/share/doc/rayap-musik/`.
- Runtime dependencies: `libasound2t64` (Debian 13+) or `libasound2`, plus
  `libc6`.
- Remove with `sudo apt remove rayap-musik`.

To upgrade after changing the code, rebuild and run the same `apt install`
command again.

### Option B — Install with cargo (no root required)

```sh
cargo install --path . --root "$HOME/.local" --locked
```

This places the binary in `~/.local/bin/rayap-musik`. Make sure `~/.local/bin`
is on your `PATH` (add `export PATH="$HOME/.local/bin:$PATH"` to your shell
profile if needed).

To reinstall after changing the code:

```sh
cargo install --path . --root "$HOME/.local" --locked --force
```

> Always pass `--locked`: `Cargo.lock` pins dependency versions that are
> compatible with Rust 1.85. Without it, `cargo install` may resolve newer
> crates that require a newer compiler.

## Usage

```sh
rayap-musik <music-directory>
rayap-musik --help
rayap-musik --version

# or during development
cargo run -- ~/Music
```

- The root directory argument is required and validated before the UI starts.
- A leading `~` or `~/` is expanded using `$HOME`.
- Run `rayap-musik --help` for the full usage text (arguments, options,
  supported formats, key bindings, and exit codes).

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including `--help` and `--version`) |
| `1` | Runtime error (e.g. terminal initialization failure) |
| `2` | Usage error (missing, unknown, or invalid directory) |

## Supported formats

`.mp3`, `.flac`, `.wav`, `.ogg`, `.oga`, `.opus`, `.m4a`, `.mp4`, `.aac`.

- Decoding is handled by [Symphonia](https://github.com/pdeljanov/Symphonia)
  through [Rodio](https://github.com/RustAudio/rodio).
- **M4A/AAC is a required format** for this project and is verified to play.
- **Opus is a known gap:** the pinned Rodio/Symphonia stack does not decode
  Opus. `.opus`/`.oga` files are listed but will be reported as skipped.
- Non-audio files and symlinks are ignored.
- If a file has no metadata, the title falls back to the file name (without
  extension) and artist/album/duration show as unknown.

## Keys

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection in the active panel |
| `Tab` | Switch focus between Library and Queue |
| `Enter` | Open folder / play selected track (Library) or play selected queue item (Queue) |
| `Backspace` | Go to parent directory (Library) |
| `Space` | Play / Pause |
| `n` | Next track |
| `p` | Previous / restart (restart if played > 3s, else previous track) |
| `+` / `=` | Volume up (5% step) |
| `-` | Volume down (5% step) |
| `q` | Quit and stop playback |

Volume starts at 70% each run; configuration is not persisted (out of MVP
scope).

## Queue behavior

Selecting an audio file replaces the queue entirely:

1. The selected track becomes the first item.
2. The selected track's folder and all subfolders are scanned recursively.
3. Other tracks follow in deterministic traversal order (folders before files,
   names sorted case-insensitively, tracks sorted by number then name).
4. The selected track never appears twice.

Navigating folders does not change the queue. Selecting an item in the Queue
panel jumps playback to that item.

## Manual test plan

Run the app against a folder containing your audio files and verify:

1. **Playback** — select a track, press `Enter`, and confirm audio is audible.
2. **M4A** — repeat with a real `.m4a` file (required format).
3. **Pause/resume** — press `Space` twice and confirm playback pauses and resumes.
4. **Next/previous** — press `n`; press `p` after > 3s (restarts) and within 3s
   (goes to previous).
5. **Volume** — press `+`/`-` and confirm the level changes.
6. **Progress** — confirm the position and progress bar advance during playback.
7. **Auto-next / end** — let the last track finish; playback stops after the
   last queue item and does not repeat.
8. **Bad file** — include a corrupt audio file; confirm it is skipped and the
   next track plays without crashing.
9. **Empty/unreadable folder** — confirm an informative status message appears.
10. **Small terminal** — shrink the terminal below 60x20; confirm a clear
    warning is shown instead of a broken UI.
11. **Terminal restore** — quit with `q` and confirm the terminal is restored.

## Development

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

Tests cover track-number parsing, deterministic sorting, metadata fallback,
recursive scanning, queue construction, queue navigation, and the previous
rule. Tests that require an audio device are performed manually.

## Documentation

- `README.md` (this file) — install, usage, formats, keymap, manual test plan
- `docs/PRD.md` — product requirements
- `docs/QA.md` — QA report, test evidence, and traceability matrix

## Project structure

```text
src/
  main.rs      lifecycle, event loop, terminal setup/cleanup, CLI parsing
  lib.rs       library crate root (enables integration testing)
  app.rs       application state and decisions
  input.rs     keyboard -> action mapping
  ui.rs        rendering
  library.rs   filesystem, formats, metadata, sorting, scanning, queue building
  player.rs    audio playback
  models.rs    shared types (Track, LibraryItem, Queue, Action, ...)
docs/
  PRD.md       product requirements
  QA.md        quality assurance report
packaging/
  build-deb.sh builds a Debian package into dist/
tests/
  library_tests.rs
  queue_tests.rs
  cli_tests.rs
```

## Known limitations

- Directory listing reads metadata synchronously; very large folders may cause a
  brief pause (no async runtime in the MVP).
- Opus is not supported by the current backend (see above).
- Queue and volume are not persisted between sessions (out of MVP scope).
