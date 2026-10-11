//! M3U playlist import and export.
//!
//! M3U is an interchange format only; the source of truth is the local
//! database. Import/export never modifies audio files.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::library::is_supported_audio;
use crate::models::Track;

/// Result of parsing an M3U file.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportOutcome {
    /// Existing, supported audio files found in the file.
    pub imported: Vec<PathBuf>,
    /// Paths that point at non-audio files or directories.
    pub skipped: Vec<String>,
    /// Paths that do not exist on disk.
    pub unavailable: Vec<String>,
}

/// Render tracks as M3U text (with an `#EXTM3U` header and `#EXTINF` records).
pub fn render_m3u(tracks: &[Track]) -> String {
    let mut out = String::from("#EXTM3U\n");
    for track in tracks {
        let seconds = track
            .duration
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        out.push_str(&format!("#EXTINF:{seconds},{}\n", m3u_label(track)));
        out.push_str(&track.path.display().to_string());
        out.push('\n');
    }
    out
}

/// Parse M3U text, resolving relative paths against `base`.
pub fn parse_m3u(text: &str, base: &Path) -> ImportOutcome {
    let mut outcome = ImportOutcome::default();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let raw = PathBuf::from(line);
        let resolved = if raw.is_absolute() {
            raw
        } else {
            base.join(&raw)
        };

        if !resolved.exists() {
            outcome.unavailable.push(line.to_string());
        } else if !resolved.is_file() || !is_supported_audio(&resolved) {
            outcome.skipped.push(line.to_string());
        } else {
            outcome.imported.push(resolved);
        }
    }

    outcome
}

/// Export tracks to an M3U file at `path`.
pub fn export_m3u_file(path: &Path, tracks: &[Track]) -> io::Result<()> {
    fs::write(path, render_m3u(tracks))
}

/// Import an M3U file, resolving relative paths against its own directory.
pub fn import_m3u_file(path: &Path) -> io::Result<ImportOutcome> {
    let text = fs::read_to_string(path)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_m3u(&text, base))
}

fn m3u_label(track: &Track) -> String {
    format!("{} - {}", track.artist_label(), track.title)
}
