# Rayap Musik

A local-first, keyboard-driven **terminal (TUI) music player and library
manager** for Linux, written in Rust. Rayap Musik indexes a local music
directory into an embedded SQLite database, groups the collection into
Songs / Albums / Artists, provides fast global search, manages persistent
playlists (with M3U import/export), and plays audio — entirely offline, with no
server and no network access.

> **Status:** v0.2.0 — Global Search & UX (Milestone 2 of the 2.2 roadmap).
> See `upgrade/prd_upgrade.md` for the v0.2 requirements,
> `upgrade/prd_upgrade_2.md` for the 2.2 navigation/search work, and `docs/QA.md`
> for the v0.1 baseline.

---

## Features

**Library & indexing**
- Persistent library root and preferences stored in a commented TOML file.
- Embedded SQLite index (no external database server) with schema migration.
- Automatic filesystem watcher plus manual rescan (`r`).
- Unavailable tracks are flagged and automatically pruned from playlists.

**Browsing**
- Songs, Albums, and Artists views with deterministic sorting.
- Album and Artist **detail views** listing their tracks.
- Wrap-around selection on every list.
- A contextual **Keys** panel that always shows the 5 most relevant shortcuts
  for the current view.

**Global search**
- One dedicated Search interface with four categories: Songs, Albums, Artists,
  Playlists (`1`–`4` to switch).
- Live, case-insensitive, multi-word matching across fields
  (e.g. `radiohead kid a` matches the album *Kid A*).
- Two modes: **edit query** and **browse results**, toggled with `/`, so the
  query is preserved while you act on results.

**Playback & queue**
- Play Now, Play Next, Add to Queue, Add all in panel (`A`), and remove.
- Shuffle (keeps the active track) and three repeat modes (Off / All / One).
- Auto-advance, per-track error skipping, and safe queue completion.
- Previous restarts if the current track has played for more than 3 seconds.

**Playlists**
- Create, rename, delete, duplicate, reorder, sort, and add tracks.
- Add whole albums, artists, or the current panel.
- M3U import/export with a summary of imported / skipped / missing paths.

**Interface**
- Four themes: Rust & Walnut (default), Catppuccin, Dracula, Nord.
- Full-screen Play Mode with an animated equalizer and a one-line rayap mascot.
- Searchable in-app Help (`?`).
- Read-only access to audio files; symlinks are never followed.

---

## Requirements

- Linux with an interactive terminal (a real TTY is required for the UI).
- Rust toolchain — built and tested with **Rust 1.85**.
- Audio backend build dependencies (ALSA + `pkg-config`):
  - Debian/Ubuntu: `sudo apt install libasound2-dev pkg-config build-essential`

---

## Build

```sh
cargo build --release --locked
```

The binary is produced at `target/release/rayap-musik`.

> Always pass `--locked`. `Cargo.lock` pins versions (ratatui 0.28, rodio 0.21,
> and others) that compile on Rust 1.85; resolving freely may select crates that
> require a newer toolchain.

## Install

Prebuilt releases are published on the
[GitHub Releases](https://github.com/adam-astiti/Rayap-musik/releases) page.

### Option A — Prebuilt binary (any Linux)

Download `rayap-musik_<version>_x86_64.tar.gz`, extract it, and run the binary:

```sh
tar -xzf rayap-musik_*_x86_64.tar.gz
./rayap-musik_*/rayap-musik ~/Music
```

The binary is built against **glibc 2.31** (runs on Debian 11 / Ubuntu 20.04 and
newer). At runtime the only external dependency is **ALSA** (`libasound2`), which
is already present on virtually every desktop Linux; on a minimal system install
it with `sudo apt install libasound2`.

### Option B — Debian package (`.deb`)

Download the `.deb` from Releases, or build it locally:

```sh
packaging/build-deb.sh
sudo apt install ./dist/rayap-musik_*.deb
```

The package pulls in its runtime dependency (`libasound2`) automatically.

### Option C — Install with cargo (no root required, needs the Rust toolchain)

```sh
cargo install --path . --root "$HOME/.local" --locked
```

---

## Quick start

```sh
rayap-musik                  # uses the root stored in the config file
rayap-musik ~/Music          # sets the root and saves it for next time
rayap-musik --help
rayap-musik --version
```

- A leading `~` or `~/` in the path is expanded using `$HOME`.
- The music directory is remembered, so you usually pass it only once.
- Startup opens the Library view and **never auto-plays**.

On first launch the library is scanned in the background; the status line shows
progress and the library populates as files are indexed.

---

## Configuration

Settings live in a commented TOML file:

```
$XDG_CONFIG_HOME/rayap-musik/config.toml      # default ~/.config/rayap-musik/
```

The database (library index + playlists) lives in:

```
$XDG_DATA_HOME/rayap-musik/library.db          # default ~/.local/share/rayap-musik/
```

The config file is human-editable: comments and unknown keys are preserved
across saves, and a single malformed value falls back to its default without
discarding the rest of the file.

```toml
# Music root. A leading ~ is expanded to $HOME.
root = "~/Music"

# One of: rust-walnut, catppuccin, dracula, nord
theme = "rust-walnut"

# Startup volume, 0-100.
volume = 70

[advanced]
# Where the library index + playlists are stored. Leave as-is unless you know
# what you are doing.
data_dir = "/home/you/.local/share/rayap-musik"
```

`s` (shuffle), `R` (repeat), and the queue are session-only and are not
persisted; the volume and theme are saved when the app exits.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including `--help` and `--version`) |
| `1` | Runtime error (e.g. terminal initialization failure) |
| `2` | Usage error (missing, unknown, or invalid directory) |

---

## The interface

The main screen is split into panels:

<img width="1893" height="1105" alt="image" src="https://github.com/user-attachments/assets/935d1c05-e9b7-4c40-b771-a739ec99059a" />



- **Library panel** (left): the active view — Songs, Albums, Artists,
  Playlists, a detail view, or Search.
- **Queue panel** (right): the current playback queue; the active track is
  marked with `>`.
- **Keys panel** (bottom): a context-aware helper. It changes as you move
  between views and focus, so it always lists the most useful shortcuts.
- `Tab` toggles focus between the Library and Queue panels.

---

## Navigation

- `↑` / `↓` move the selection and **wrap around** at both ends.
- `Tab` switches focus between the Library and Queue panels.
- `1` / `2` / `3` / `4` switch views (Songs / Albums / Artists / Playlists).
- `Esc` / `Backspace` goes back (leaves a detail view, cancels a picker).
- Pressing `?` anywhere opens the searchable Help screen.

Selecting an item:
- In **Songs**, `Enter` plays the selected song.
- In **Albums**, **Artists**, and **Playlists**, `Enter` **opens the detail
  view** listing that collection's tracks.
- Inside a detail view, `Enter` plays the selected track, and `Esc` returns.

Navigation is remembered: opening an album from Search and pressing `Esc`
returns you to Search with the query, category, and selection intact, whereas
opening it from the Albums view returns you to Albums.

---

## Global search

Press `/` to open the Search interface. It has four categories — Songs, Albums,
Artists, Playlists — selected with `1`–`4`. One query applies to all categories
and results update live as you type.

Search is **case-insensitive** and **multi-word**: every word must match at
least one field of an item (title, artist, album, or name). For example
`radiohead kid a` matches the album *Kid A* by Radiohead even though the words
match different fields.

Search has two modes, toggled with `/`:

1. **Edit query** (default): typing edits the query and filters results live.
   `↑`/`↓` still move the selection. Press `/` (or `Enter`) to switch to
   browsing results.
2. **Browse results**: the query is frozen. `↑`/`↓` navigate the results, and
   `a`, `A`, `N`, `P`, `Enter`, and the usual shortcuts act on the selected
   result. Press `/` to return to editing.

In both modes `1`–`4` switch category and `Esc` closes Search.

What `Enter` does in results depends on the category:

| Category | `Enter` |
|---|---|
| Song | Plays the song |
| Album | Opens the album detail view |
| Artist | Opens the artist detail view |
| Playlist | Opens the playlist detail view |

Empty states: `Search your library…` before you type, and
`No results found.` when a non-empty query matches nothing.

---

## Queue and playback

- `Space` plays/pauses.
- `n` / `p` go to the next / previous track. Previous **restarts** the current
  track if it has played for more than 3 seconds.
- `+` / `-` (or `=`) adjust the volume in 5% steps (clamped 0–100).
- `s` toggles shuffle; the active track stays active when the order changes.
- `R` cycles repeat: Off → All → One.
- In the Queue panel (`Tab`), `Enter` plays the highlighted entry and `d`
  removes it.

Adding to the queue (works from the Library panel):

| Key | Adds |
|---|---|
| `a` | The selection (song, album, artist, or playlist) |
| `A` | Everything currently in the panel |
| `N` | The selection, inserted right after the active track (Play Next) |

For `A`, "the panel" means: all songs in Songs/Albums/Artists, all tracks of the
open detail view, or every match in the active Search category. Duplicates are
removed by file path.

A failed track is skipped automatically; if every track fails, playback stops
safely rather than looping.

---

## Playlists

Playlists are stored in the local database and persist between sessions.

| Key | Action |
|---|---|
| `c` | Create a playlist |
| `e` | Rename the selected playlist |
| `D` | Duplicate the selected playlist |
| `x` | Delete a playlist (press twice to confirm), or remove a track inside a playlist |
| `P` | Add the current selection to a playlist (opens a picker) |
| `I` | Import an M3U playlist |
| `E` | Export the selected playlist to M3U |

Inside an open playlist:

| Key | Action |
|---|---|
| `J` / `K` | Move the selected track down / up |
| `S` | Sort by artist → album → track |
| `x` | Remove the selected track from the playlist |

M3U import/export preserves order where the format allows, resolves relative and
absolute paths, and reports how many entries were imported, skipped, or
unavailable. Importing adds found paths to the index; tracks outside the library
root are marked unavailable on the next rescan. Tracks that become unavailable
are removed from playlists automatically.

---

## Themes, Play Mode, and Help

- `T` cycles the theme: **Rust & Walnut** (default) → Catppuccin → Dracula →
  Nord. The chosen theme is saved on exit.
- `m` toggles the full-screen **Play Mode**, which shows the current track, an
  animated equalizer, and the one-line rayap mascot `--[ d(◕_◕)b ]--` (the eyes
  blink). Playback continues uninterrupted while Play Mode is open.
- `?` opens the searchable **Help** page: type to filter keybindings by key,
  action, or description; `Backspace` edits the query; `Esc` clears it, then
  closes; `?` closes.

---

## Keyboard reference

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection (wraps around) |
| `Tab` | Switch focus (Library / Queue) |
| `1` / `2` / `3` / `4` | Songs / Albums / Artists / Playlists (Search: category) |
| `/` | Open Search; `/` again toggles edit ↔ browse results |
| `Enter` | Play song / open a detail view / play the selected result |
| `Esc` / `Backspace` | Back (leave a detail view, cancel a picker) |
| `Space` | Play / Pause |
| `n` / `p` | Next / previous track (previous restarts if played > 3s) |
| `+` / `=` / `-` | Volume up / down (5% step) |
| `a` | Add selection to the end of the queue |
| `A` | Add everything in the current panel to the queue |
| `N` | Play Next (insert after the active track) |
| `d` | Remove the selected queue entry (Queue panel) |
| `s` | Toggle shuffle |
| `R` | Cycle repeat (Off → All → One → Off) |
| `c` / `e` / `D` / `x` | Create / rename / duplicate / delete a playlist |
| `P` | Add the selection to a playlist |
| `I` / `E` | Import / export an M3U playlist |
| `J` / `K` / `S` | Move track down / up / sort (inside a playlist) |
| `m` | Toggle Play Mode |
| `T` | Cycle theme |
| `?` | Toggle searchable Help |
| `r` | Rescan the library |
| `q` | Quit and stop playback |

Inside Search (edit mode) printable keys edit the query; inside Search (browse
mode) the normal shortcuts act on the highlighted result. A prompt (playlist
name or M3U path) captures text until `Enter` confirms or `Esc` cancels.

---

## Supported formats

Indexed extensions: `.mp3`, `.flac`, `.wav`, `.ogg`, `.oga`, `.opus`, `.m4a`,
`.mp4`, `.aac`.

- Decoding is handled by Symphonia through Rodio. **MP3, FLAC, WAV, OGG Vorbis,
  and M4A/AAC are decoded by the pinned stack.**
- **Opus is a known gap:** the pinned Rodio/Symphonia stack does not decode it,
  so `.opus` files are indexed but skipped on playback.
- Non-audio files and symlinks are ignored.
- Metadata fallback: the title uses the file stem; album and artist fall back to
  the containing folder's name when tags are missing.

---

## Manual test plan

The UI and audio need a real terminal and an audio device, so these are manual:

1. **Formats** — play a real file of each type: MP3, FLAC, WAV, OGG Vorbis,
   M4A/AAC. (Opus is a known gap.)
2. **Persistence** — set a root, quit, relaunch; root, index, playlists, theme,
   and volume are restored without auto-play.
3. **Watcher** — add, rename, and delete files while the app runs; the library
   updates without restarting. Use `r` to force a rescan.
4. **Views** — switch Songs / Albums / Artists / Playlists; open album and
   artist details; confirm `Esc` returns correctly.
5. **Search** — search multi-word; switch categories with `1`–`4`; toggle with
   `/`; confirm actions (`a`, `A`, `N`, `P`, `Enter`) work on results while the
   query stays; confirm Back returns to Search with state intact.
6. **Queue** — Play Now / Next / Add / Add-all / remove; shuffle on/off keeps
   the active track; repeat Off/All/One behave correctly.
7. **Playlists** — create / rename / duplicate / delete, reorder, sort, and M3U
   import/export.
8. **Play Mode / themes** — toggle `m`, cycle `T`; the mascot and equalizer
   animate while playing and freeze when paused.
9. **Help / Keys panel** — `?` filters keys live; the Keys panel changes per
   context (e.g. shuffle/repeat/remove appear in the Queue).
10. **Error handling** — corrupt or unavailable files are skipped without
    crashing.
11. **Terminal restore** — quit normally and via handled errors; the terminal is
    always restored.

---

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

Automated tests cover configuration, database persistence/migrations, scanning,
catalog grouping/sorting/search, queue operations (including shuffle/repeat),
M3U parsing, theme selection, keyboard mapping, keybinding metadata, and
navigation helpers. Tests requiring an audio device or a real terminal are
manual (above).

### Releasing

Releases are automated. Bump `version` in `Cargo.toml`, commit, then push a
matching tag:

```sh
git tag v0.2.0
git push origin v0.2.0
```

The `.github/workflows/release.yml` workflow builds inside a `debian:11`
container (glibc 2.31) and attaches a `.tar.gz`, a `.deb`, and `SHA256SUMS` to a
**draft** GitHub Release. It fails early if the tag does not match `Cargo.toml`
(`vX.Y.Z`) or if the built binary requires a glibc newer than 2.31 (checked with
`objdump`). Review the draft, then publish it.

## Project structure

```text
src/
  main.rs      lifecycle, event loop, terminal setup/cleanup, CLI parsing
  lib.rs       library crate root (enables integration testing)
  app.rs       application state and decisions (queue, playback, navigation)
  input.rs     key event -> InputMode-specific Action mapping
  keys.rs      centralized keybindings (dispatch + helper + help + search)
  ui.rs        rendering only (views, detail, Play Mode, Help, Keys panel)
  library.rs   filesystem, formats, metadata, scanning
  catalog.rs   grouping/sorting of the index + categorized global search
  indexer.rs   background scan worker + filesystem watcher
  db.rs        SQLite persistence (index + playlists) and migrations
  config.rs    commented TOML preferences
  m3u.rs       M3U import/export
  theme.rs     color themes
  player.rs    audio playback
  models.rs    shared types (Track, Queue, Action, search, wrap helpers, ...)
docs/
  PRD.md           v0.1 product requirements (baseline)
  QA.md            v0.1 quality assurance report
upgrade/
  prd_upgrade.md   v0.2 requirements
  prd_upgrade_2.md 2.2 navigation & search requirements
packaging/
  build-deb.sh     builds a Debian package into dist/
tests/
  library_tests.rs  queue_tests.rs  cli_tests.rs  config_tests.rs
  db_tests.rs  m3u_tests.rs  format_verify.rs
```

## Known limitations

- **Opus is not decoded** by the pinned Rodio/Symphonia stack.
- **No image rendering**: Play Mode uses an ASCII equalizer and mascot instead
  of real album artwork.
- The filesystem watcher triggers a full rescan (debounced); very large
  libraries may see a brief pause while metadata is read in the background.
- M3U import adds found paths to the index; tracks outside the library root are
  marked unavailable on the next rescan.
- The queue, shuffle, and repeat are not persisted between sessions.
