//! Integration tests for artwork resolution.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rayap_musik::artwork::{resolve, Artwork};
use rayap_musik::models::Track;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-art-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
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

fn track(path: PathBuf) -> Track {
    Track {
        path,
        title: "01".to_string(),
        artist: None,
        album: None,
        track_number: None,
        duration: None,
    }
}

#[test]
fn resolves_folder_image() {
    let temp = TempDir::new();
    let audio = temp.file("Album/01.mp3");
    temp.file("Album/cover.jpg");

    assert!(matches!(resolve(&track(audio)), Artwork::FolderImage(_)));
}

#[test]
fn placeholder_when_no_artwork() {
    let temp = TempDir::new();
    let audio = temp.file("Album/01.mp3");

    assert_eq!(resolve(&track(audio)), Artwork::Placeholder);
}

#[test]
fn folder_image_prefers_cover_over_others() {
    let temp = TempDir::new();
    let audio = temp.file("Album/01.mp3");
    temp.file("Album/front.jpg");
    temp.file("Album/cover.png");

    match resolve(&track(audio)) {
        Artwork::FolderImage(path) => {
            assert_eq!(path.file_name().unwrap(), "cover.png");
        }
        other => panic!("expected folder image, got {other:?}"),
    }
}
