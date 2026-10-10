# Rayap Musik

Local-first terminal music player for Linux, written in Rust.

> Status: **Milestone 1 — UI shell.** The library browser and audio playback
> are not implemented yet. See `docs/PRD.md` for the full plan.

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

The root directory argument is currently stored for display only. Validation
and browsing arrive in Milestone 2.

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
