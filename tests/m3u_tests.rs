//! Integration tests for M3U import/export.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rayap_musik::m3u::{parse_m3u, render_m3u};
use rayap_musik::models::Track;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-m3u-{}-{}",
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
        fs::write(&path, b"").expect("write file");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn track(path: PathBuf) -> Track {
    Track {
        path,
        title: "A".to_string(),
        artist: Some("Artist".to_string()),
        album: Some("Album".to_string()),
        track_number: Some(1),
        duration: Some(Duration::from_secs(123)),
    }
}

#[test]
fn render_m3u_writes_header_and_paths() {
    let temp = TempDir::new();
    let a = temp.file("a.mp3");
    let text = render_m3u(&[track(a)]);

    assert!(text.starts_with("#EXTM3U\n"));
    assert!(text.contains("#EXTINF:123,Artist - A\n"));
}

#[test]
fn parse_resolves_relative_paths_against_base() {
    let temp = TempDir::new();
    let a = temp.file("a.mp3");
    let text = format!("#EXTM3U\na.mp3\n{}\n/missing/file.mp3\n", a.display());

    let outcome = parse_m3u(&text, temp.path());
    assert_eq!(
        outcome.imported.len(),
        2,
        "relative + absolute existing file"
    );
    assert_eq!(outcome.unavailable.len(), 1, "missing path reported");
}

#[test]
fn parse_skips_comments_and_non_audio() {
    let temp = TempDir::new();
    temp.file("notes.txt");
    let text = "#EXTM3U\n#EXTINF:123,foo\nnotes.txt\n";

    let outcome = parse_m3u(text, temp.path());
    assert_eq!(outcome.imported.len(), 0);
    assert_eq!(outcome.skipped.len(), 1, "non-audio file skipped");
}

#[test]
fn render_then_parse_round_trips() {
    let temp = TempDir::new();
    let a = temp.file("a.mp3");
    let rendered = render_m3u(&[track(a)]);

    let outcome = parse_m3u(&rendered, temp.path());
    assert_eq!(outcome.imported.len(), 1);
    assert_eq!(outcome.unavailable.len(), 0);
    assert_eq!(outcome.skipped.len(), 0);
}
