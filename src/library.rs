//! Filesystem scanning, audio detection, metadata, and deterministic sorting.

use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};

use crate::models::Track;

/// File extensions treated as supported audio for the MVP.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "ogg", "oga", "opus", "m4a", "mp4", "aac",
];

/// Errors produced by library operations.
#[derive(Debug)]
pub enum LibraryError {
    NotFound(PathBuf),
    NotDirectory(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::NotFound(path) => write!(f, "path not found: {}", path.display()),
            LibraryError::NotDirectory(path) => write!(f, "not a directory: {}", path.display()),
            LibraryError::Io { path, source } => {
                write!(f, "cannot read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for LibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LibraryError::Io { source, .. } => Some(source),
            LibraryError::NotFound(_) | LibraryError::NotDirectory(_) => None,
        }
    }
}

/// Expand a leading `~` or `~/` using `$HOME`.
///
/// If `$HOME` is not set the input is returned unchanged.
pub fn expand_tilde(input: &str) -> String {
    match (
        input.strip_prefix("~/"),
        input == "~",
        std::env::var_os("HOME"),
    ) {
        (Some(rest), _, Some(home)) => {
            format!("{}/{}", home.to_string_lossy().trim_end_matches('/'), rest)
        }
        (None, true, Some(home)) => home.to_string_lossy().into_owned(),
        _ => input.to_string(),
    }
}

/// Validate that `path` exists and is a directory.
///
/// Returns the canonicalized path on success.
pub fn validate_root(path: &Path) -> Result<PathBuf, LibraryError> {
    if !path.exists() {
        return Err(LibraryError::NotFound(path.to_path_buf()));
    }
    let canonical = path.canonicalize().map_err(|source| LibraryError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !canonical.is_dir() {
        return Err(LibraryError::NotDirectory(path.to_path_buf()));
    }
    Ok(canonical)
}

/// Whether `path` has a supported audio extension (case-insensitive).
pub fn is_supported_audio(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => SUPPORTED_EXTENSIONS
            .iter()
            .any(|supported| ext.eq_ignore_ascii_case(supported)),
        None => false,
    }
}

/// Parse a track number from a metadata string.
///
/// Handles plain values (`"1"`, `"01"`) and `number/total` forms (`"1/12"`).
/// Returns `None` for empty or non-numeric values.
pub fn parse_track_number(raw: &str) -> Option<u32> {
    let head = raw.split('/').next()?.trim();
    if head.is_empty() {
        return None;
    }
    head.parse::<u32>().ok()
}

fn name_key(name: &str) -> String {
    name.to_lowercase()
}

fn path_name_key(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Compare two tracks by track number, falling back to file name (FR-03).
pub fn compare_tracks(a: &Track, b: &Track) -> Ordering {
    match (a.track_number, b.track_number) {
        (Some(x), Some(y)) => x
            .cmp(&y)
            .then_with(|| name_key(&a.file_name()).cmp(&name_key(&b.file_name()))),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => name_key(&a.file_name()).cmp(&name_key(&b.file_name())),
    }
}

/// Read metadata for `path`, falling back to sensible defaults (FR-02, FR-LIB-05).
///
/// The title falls back to the file stem; album and artist fall back to the
/// containing folder's name. A file with missing or corrupt metadata still
/// produces a usable `Track`; this function never fails.
pub fn read_track(path: &Path) -> Track {
    let fallback_title = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder_name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned());

    let mut track = Track {
        path: path.to_path_buf(),
        title: fallback_title,
        artist: folder_name.clone(),
        album: folder_name,
        track_number: None,
        duration: None,
    };

    if let Ok(tagged) = lofty::read_from_path(path) {
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if let Some(title) = non_empty(tag.title().as_deref()) {
                track.title = title;
            }
            if let Some(artist) = non_empty(tag.artist().as_deref()) {
                track.artist = Some(artist);
            }
            if let Some(album) = non_empty(tag.album().as_deref()) {
                track.album = Some(album);
            }
            track.track_number = tag
                .get_string(&ItemKey::TrackNumber)
                .and_then(parse_track_number);
        }

        let duration = tagged.properties().duration();
        if !duration.is_zero() {
            track.duration = Some(duration);
        }
    }

    track
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Recursively scan `root`, returning all supported audio tracks with metadata.
///
/// Order is deterministic: depth-first, directories before files, tracks sorted
/// by number then file name. This is the source for the persistent library index.
pub fn scan_tracks(root: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();
    collect_tracks(root, &mut tracks);
    tracks
}

/// List the directories and audio tracks directly inside `dir`, each sorted
/// deterministically. Symlinks are ignored.
fn list_dir_entries(dir: &Path) -> Result<(Vec<PathBuf>, Vec<Track>), LibraryError> {
    let entries = fs::read_dir(dir).map_err(|source| LibraryError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    let mut dirs = Vec::new();
    let mut tracks = Vec::new();

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        if file_type.is_dir() {
            dirs.push(path);
        } else if file_type.is_file() && is_supported_audio(&path) {
            tracks.push(read_track(&path));
        }
    }

    dirs.sort_by_key(|path| path_name_key(path));
    tracks.sort_by(compare_tracks);
    Ok((dirs, tracks))
}

/// Recursively collect tracks under `dir`: subdirectories before files, names
/// sorted case-insensitively, tracks sorted by number then file name.
fn collect_tracks(dir: &Path, out: &mut Vec<Track>) {
    let Ok((dirs, tracks)) = list_dir_entries(dir) else {
        return;
    };
    for subdir in dirs {
        collect_tracks(&subdir, out);
    }
    out.extend(tracks);
}
