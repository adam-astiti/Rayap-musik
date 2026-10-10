# QA Report — Rayap Musik

- **Version:** 0.1.0 (MVP)
- **Date:** 2026-10-10
- **Status:** PASS — all MVP acceptance criteria met
- **Author:** QA verification pass

## 1. Environment under test

| Item | Value |
|---|---|
| OS | Debian GNU/Linux 13 (trixie) |
| Rust | 1.85.1 (system toolchain) |
| Audio backend | ALSA (cpal 0.16); PipeWire present on host |
| Key dependencies | rodio 0.21.1, symphonia 0.5.5, lofty 0.21.1, ratatui 0.28.1, crossterm 0.28.1 |

Tooling verified present: `cargo`, `rustfmt`, `clippy`, `pkg-config`,
`libasound2-dev`.

## 2. Test summary

| Suite | Result |
|---|---|
| Unit tests (lib) | 3/3 |
| CLI integration tests | 9/9 |
| Library integration tests | 11/11 |
| Queue integration tests | 7/7 |
| **Total automated** | **30/30** |

Automated commands:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

All commands pass with no warnings or errors.

## 3. Manual / terminal tests (via pseudo-terminal)

| Scenario | Result |
|---|---|
| UI shell renders all five panels | PASS |
| Quit restores terminal (raw mode, alternate screen, cursor) | PASS |
| Small terminal (<60x20) shows warning | PASS |
| Browse folders, enter subdirectory, back to parent | PASS |
| Empty directory shows informative message | PASS |
| Unreadable directory shows error without crash | PASS |
| M4A file decodes and plays | PASS |
| WAV file plays and end-of-playback is detected | PASS |
| Volume up from 70% → 75% | PASS |
| Play/pause/resume | PASS |
| Queue built from subtree, auto-next to end, "End of queue" | PASS |
| Corrupt file skipped, next track plays | PASS |
| All-failed queue stops with "All tracks in queue failed" | PASS |
| Queue item selection jumps playback | PASS |
| Full integration run (navigate → play → pause → volume → queue → next/prev → quit) with no panic | PASS |

## 4. Functional requirements traceability

| ID | Requirement | Status | Evidence |
|---|---|---|---|
| FR-01 | Filesystem browser | PASS | `list_directory`, folder navigation, empty/unreadable handling |
| FR-02 | Audio metadata + fallback | PASS | `read_track` fallback; `library_tests` |
| FR-03 | Deterministic sorting | PASS | `sort_items`; track-number parsing; stability test |
| FR-04 | Audio playback | PASS | `player.rs`; M4A + WAV manual test |
| FR-05 | Queue management | PASS | `build_queue`; `queue_tests` |
| FR-06 | Completion & error handling | PASS | skip-on-failure, end-of-queue, all-fail stop |
| FR-07 | Previous/restart (3s rule) | PASS | `previous_action`; unit tests |
| FR-08 | Volume (70% start, 5% step, 0–100) | PASS | `change_volume`; unit tests + manual |
| FR-09 | User interface (5 panels, focus, small terminal) | PASS | `ui.rs`; manual tests |
| FR-10 | Keyboard mapping | PASS | `input.rs`; manual tests |
| FR-11 | Symlink & filesystem safety | PASS | symlink ignored; read-only; tests |
| FR-12 | Lifecycle & terminal restore | PASS | `TerminalGuard`; manual tests |

## 5. Acceptance criteria

All 21 acceptance criteria (AC-01 … AC-21) pass. Notably:

- **AC-20** (audio actually audible) and **AC-21** (M4A actually plays) were
  confirmed by the product owner on hardware.
- **AC-02** (invalid path → non-zero exit) is now covered by automated CLI
  tests asserting exit code `2`.
- **AC-18** (build, formatter, tests) is enforced by `clippy -D warnings`,
  `cargo fmt --check`, and 30 passing tests.

## 6. CLI verification

| Command | Exit code | Expected output |
|---|---|---|
| `rayap-musik --help` | 0 | usage, args, options, keys, formats, exit codes |
| `rayap-musik -h` | 0 | same as `--help` |
| `rayap-musik --version` | 0 | `rayap-musik 0.1.0` |
| `rayap-musik` | 2 | "missing root directory argument" |
| `rayap-musik <nonexistent>` | 2 | "path not found" |
| `rayap-musik <file>` | 2 | "not a directory" |
| `rayap-musik --bogus` | 2 | "unknown option" |
| `rayap-musik a b` | 2 | "too many arguments" |

## 7. Known issues and limitations

1. **Opus not supported.** The pinned Rodio/Symphonia stack does not decode
   Opus. `.opus`/`.oga` files are listed but skipped on playback. Accepted as a
   known gap (Opus is a "target" format, not required).
2. **Synchronous metadata reads.** Listing a directory reads metadata for every
   audio file synchronously; very large directories may cause a brief pause.
3. **Space after end of queue** replays the last track rather than restarting
   the queue from the first item. Not a requirement violation; minor UX note.
4. **No `.gitignore`.** A `.git` repository exists but build artifacts are not
   ignored.

## 8. Verdict

Rayap Musik v0.1.0 meets the MVP requirements in `docs/PRD.md`. All automated
tests pass, all manual scenarios pass, and the required M4A format is verified
to play on real hardware.
