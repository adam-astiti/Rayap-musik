//! Integration tests for the library module.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayap_musik::library::{
    expand_tilde, is_supported_audio, parse_track_number, read_track, scan_tracks, validate_root,
    LibraryError,
};
use rayap_musik::models::Track;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Minimal temporary directory helper that removes itself on drop.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn file(&self, relative: &str) -> PathBuf {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&path, b"").expect("write file");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn supported_extensions_include_m4a_case_insensitively() {
    assert!(is_supported_audio(Path::new("song.m4a")));
    assert!(is_supported_audio(Path::new("song.M4A")));
    assert!(is_supported_audio(Path::new("song.mp3")));
    assert!(is_supported_audio(Path::new("song.FLAC")));
    assert!(!is_supported_audio(Path::new("notes.txt")));
    assert!(!is_supported_audio(Path::new("no_extension")));
}

#[test]
fn parse_track_number_handles_common_formats() {
    assert_eq!(parse_track_number("1"), Some(1));
    assert_eq!(parse_track_number("01"), Some(1));
    assert_eq!(parse_track_number("1/12"), Some(1));
    assert_eq!(parse_track_number("12/12"), Some(12));
    assert_eq!(parse_track_number(" 3 "), Some(3));
    assert_eq!(parse_track_number(""), None);
    assert_eq!(parse_track_number("A1"), None);
    assert_eq!(parse_track_number("/12"), None);
}

#[test]
fn scan_tracks_collects_metadata_and_order() {
    let temp = TempDir::new();
    temp.file("A/01.mp3");
    temp.file("A/02.mp3");
    temp.file("root.mp3");

    let tracks = scan_tracks(temp.path());
    let names: Vec<String> = tracks.iter().map(Track::file_name).collect();

    assert_eq!(names, vec!["01.mp3", "02.mp3", "root.mp3"]);
    assert_eq!(tracks[0].title, "01", "fallback title is the file stem");
}

#[test]
fn scan_tracks_skips_symlinked_directories() {
    let temp = TempDir::new();
    let outside = TempDir::new();
    temp.file("real.mp3");
    fs::write(outside.path().join("hidden.mp3"), b"").expect("write outside file");
    std::os::unix::fs::symlink(outside.path(), temp.path().join("linked")).expect("symlink dir");

    let tracks = scan_tracks(temp.path());
    let names: Vec<String> = tracks.iter().map(Track::file_name).collect();

    assert_eq!(names, vec!["real.mp3"]);
}

#[test]
fn read_track_falls_back_to_folder_name() {
    let temp = TempDir::new();
    let path = temp.file("Album Name/01.mp3");

    let track = read_track(&path);
    assert_eq!(track.title, "01", "title falls back to the file stem");
    assert_eq!(
        track.artist.as_deref(),
        Some("Album Name"),
        "artist uses folder"
    );
    assert_eq!(
        track.album.as_deref(),
        Some("Album Name"),
        "album uses folder"
    );
}

#[test]
fn validate_root_reports_missing_and_wrong_type() {
    let temp = TempDir::new();
    let file = temp.file("plain.txt");

    assert!(matches!(
        validate_root(Path::new("/definitely/not/here/rayap")),
        Err(LibraryError::NotFound(_))
    ));
    assert!(matches!(
        validate_root(&file),
        Err(LibraryError::NotDirectory(_))
    ));

    let root = validate_root(temp.path()).expect("valid root");
    assert!(root.is_dir());
}

#[test]
fn expand_tilde_leaves_other_paths_untouched() {
    assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    assert_eq!(expand_tilde("relative/path"), "relative/path");
    assert_eq!(expand_tilde("~user/music"), "~user/music");
}

#[test]
fn expand_tilde_uses_home_when_set() {
    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy().trim_end_matches('/').to_string();
        assert_eq!(expand_tilde("~/Music"), format!("{home}/Music"));
        assert_eq!(expand_tilde("~"), home);
    }
}
