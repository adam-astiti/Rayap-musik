//! Filesystem browsing, audio detection, metadata, and deterministic sorting.

use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};

use crate::models::{LibraryItem, Track};

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

/// List the directories and supported audio files inside `dir`.
///
/// Symlinks are ignored. The result is sorted deterministically by
/// [`sort_items`]. An unreadable directory is reported as [`LibraryError::Io`].
pub fn list_directory(dir: &Path) -> Result<Vec<LibraryItem>, LibraryError> {
    let entries = fs::read_dir(dir).map_err(|source| LibraryError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        // FR-11: symlinks are ignored.
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if file_type.is_dir() {
            items.push(LibraryItem::Directory { path, name });
        } else if file_type.is_file() && is_supported_audio(&path) {
            items.push(LibraryItem::Track(read_track(&path)));
        }
    }

    sort_items(&mut items);
    Ok(items)
}

/// Sort browser items deterministically (FR-03).
pub fn sort_items(items: &mut [LibraryItem]) {
    items.sort_by(compare_items);
}

fn compare_items(a: &LibraryItem, b: &LibraryItem) -> Ordering {
    match (a, b) {
        (LibraryItem::Directory { name: na, .. }, LibraryItem::Directory { name: nb, .. }) => {
            name_key(na).cmp(&name_key(nb))
        }
        (LibraryItem::Directory { .. }, LibraryItem::Track(_)) => Ordering::Less,
        (LibraryItem::Track(_), LibraryItem::Directory { .. }) => Ordering::Greater,
        (LibraryItem::Track(ta), LibraryItem::Track(tb)) => compare_tracks(ta, tb),
    }
}

fn compare_tracks(a: &Track, b: &Track) -> Ordering {
    match (a.track_number, b.track_number) {
        (Some(x), Some(y)) => x
            .cmp(&y)
            .then_with(|| name_key(&a.file_name()).cmp(&name_key(&b.file_name()))),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => name_key(&a.file_name()).cmp(&name_key(&b.file_name())),
    }
}

/// Read metadata for `path`, falling back to sensible defaults (FR-02).
///
/// A file with missing or corrupt metadata still produces a usable `Track`;
/// this function never fails.
pub fn read_track(path: &Path) -> Track {
    let fallback_title = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut track = Track {
        path: path.to_path_buf(),
        title: fallback_title,
        artist: None,
        album: None,
        track_number: None,
        duration: None,
    };

    if let Ok(tagged) = lofty::read_from_path(path) {
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if let Some(title) = tag.title() {
                let title = title.trim();
                if !title.is_empty() {
                    track.title = title.to_string();
                }
            }
            track.artist = non_empty(tag.artist().as_deref());
            track.album = non_empty(tag.album().as_deref());
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

/// Recursively collect supported audio files under `root` (FR-03, FR-05).
///
/// Order is deterministic: depth-first, subdirectories before the current
/// directory's files, with names sorted case-insensitively at every level.
/// Symlinks are not followed.
pub fn scan_recursive(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    scan_dir(root, &mut files);
    files
}

/// Build a playback queue for a selected track (FR-05).
///
/// The selected track is first, followed by every other audio file in the
/// selected track's directory subtree, in deterministic traversal order, with
/// no duplicates. Only the subtree is scanned; unrelated directories are
/// excluded.
pub fn build_queue(selected: &Path) -> Vec<Track> {
    let selected_track = read_track(selected);
    let mut tracks = vec![selected_track.clone()];

    let Some(parent) = selected_track.path.parent() else {
        return tracks;
    };

    let mut rest = Vec::new();
    collect_tracks(parent, &mut rest);
    for track in rest {
        if track.path != selected_track.path {
            tracks.push(track);
        }
    }
    tracks
}

/// Recursively scan `root`, returning all supported audio tracks with metadata.
///
/// Order is deterministic (directories before files, depth-first). This is the
/// source for the persistent library index.
pub fn scan_tracks(root: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();
    collect_tracks(root, &mut tracks);
    tracks
}

/// Recursively collect tracks under `dir` in traversal order: subdirectories
/// before files, names sorted case-insensitively, tracks sorted by number then
/// file name (via [`list_directory`]).
fn collect_tracks(dir: &Path, out: &mut Vec<Track>) {
    let Ok(items) = list_directory(dir) else {
        return;
    };
    for item in items {
        match item {
            LibraryItem::Directory { path, .. } => collect_tracks(&path, out),
            LibraryItem::Track(track) => out.push(track),
        }
    }
}

fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut dirs = Vec::new();
    let mut audio = Vec::new();

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            dirs.push(path);
        } else if file_type.is_file() && is_supported_audio(&path) {
            audio.push(path);
        }
    }

    dirs.sort_by_key(|path| path_name_key(path));
    audio.sort_by_key(|path| path_name_key(path));

    for subdir in dirs {
        scan_dir(&subdir, files);
    }
    files.extend(audio);
}
