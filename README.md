# Rayap Musik

Local-first terminal music player for Linux, written in Rust.

> Status: **Milestone 2 — filesystem browser.** Browsing, metadata, sorting,
> and recursive scanning work. Audio playback and queue management are not
> implemented yet. See `docs/PRD.md` for the full plan.

## Requirements

- Linux with an interactive terminal
- Rust toolchain (stable)
- For audio playback (Milestone 3+): ALSA development files and `pkg-config`
  - Debian/Ubuntu: `sudo apt install libasound2-dev pkg-config build-essential`

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run -- ~/Music
```

The root directory argument is required and validated before the UI starts.
A leading `~` is expanded using `$HOME`. Missing arguments, missing paths, and
non-directory paths print an error and exit with a non-zero code.

Supported audio extensions: `.mp3`, `.flac`, `.wav`, `.ogg`, `.oga`, `.opus`,
`.m4a`, `.mp4`, `.aac`. Non-audio files and symlinks are ignored.

## Keys

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection in the active panel |
| `Tab` | Switch focus between Library and Queue |
| `Enter` | Open folder / play selected track |
| `Backspace` | Go to parent directory (Library) |
| `Space` | Play / Pause |
| `n` | Next track |
| `p` | Previous / restart (3-second rule) |
| `+` / `=` | Volume up |
| `-` | Volume down |
| `q` | Quit |

## Development

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```
