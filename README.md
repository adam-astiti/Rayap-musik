# Rayap Musik

Local-first terminal music player for Linux, written in Rust.

> Status: **Milestone 3 — single-track playback.** Browsing, metadata, sorting,
> recursive scanning, and single-track playback (incl. M4A/AAC) work. Queue
> management and auto-next are not implemented yet. See `docs/PRD.md`.

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
`.m4a`, `.mp4`, `.aac`. Non-audio files and symlinks are ignored. MP3, FLAC,
WAV, OGG Vorbis and **M4A/AAC are decoded via Symphonia** (M4A verified
manually); Opus support depends on the Symphonia backend.

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
