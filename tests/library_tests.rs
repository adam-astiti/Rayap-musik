//! Integration tests for the library module.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayap_musik::library::{
    expand_tilde, is_supported_audio, list_directory, parse_track_number, scan_recursive,
    sort_items, validate_root, LibraryError,
};
use rayap_musik::models::{LibraryItem, Track};

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

    fn dir(&self, relative: &str) -> PathBuf {
        let path = self.path.join(relative);
        fs::create_dir_all(&path).expect("create dir");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn track_item(name: &str, number: Option<u32>) -> LibraryItem {
    LibraryItem::Track(Track {
        path: PathBuf::from(name),
        title: name.to_string(),
        artist: None,
        album: None,
        track_number: number,
        duration: None,
    })
}

fn item_name(item: &LibraryItem) -> String {
    match item {
        LibraryItem::Directory { name, .. } => name.clone(),
        LibraryItem::Track(track) => track.file_name(),
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
fn sort_items_is_deterministic() {
    let mut items = vec![
        track_item("track10.mp3", Some(10)),
        track_item("track2.mp3", Some(2)),
        track_item("track1.mp3", Some(1)),
        track_item("b-side.mp3", None),
        track_item("a-side.mp3", None),
        LibraryItem::Directory {
            path: PathBuf::from("beta"),
            name: "beta".to_string(),
        },
        LibraryItem::Directory {
            path: PathBuf::from("Alpha"),
            name: "Alpha".to_string(),
        },
    ];

    sort_items(&mut items);
    let names: Vec<String> = items.iter().map(item_name).collect();

    assert_eq!(
        names,
        vec![
            "Alpha", // directories first, case-insensitive alphabetical
            "beta",
            "track1.mp3", // then numbered tracks by number
            "track2.mp3",
            "track10.mp3",
            "a-side.mp3", // unnumbered tracks after numbered, by file name
            "b-side.mp3",
        ]
    );
}

#[test]
fn numbered_tracks_are_stable_under_repeated_sorting() {
    let build = || {
        vec![
            track_item("12-twelve.mp3", Some(12)),
            track_item("02-two.mp3", Some(2)),
            track_item("01-one.mp3", Some(1)),
            track_item("99-ninety.mp3", None),
        ]
    };

    let mut first = build();
    sort_items(&mut first);
    let mut second = build();
    sort_items(&mut second);

    let names = |items: &[LibraryItem]| items.iter().map(item_name).collect::<Vec<_>>();
    assert_eq!(names(&first), names(&second));
}

#[test]
fn list_directory_filters_and_falls_back() {
    let temp = TempDir::new();
    temp.dir("Sub");
    temp.file("audio.m4a");
    temp.file("Song.mp3");
    temp.file("notes.txt");
    temp.file("cover.png");

    let items = list_directory(temp.path()).expect("list directory");
    let names: Vec<String> = items.iter().map(item_name).collect();

    // Directories are listed before audio files.
    assert_eq!(names, vec!["Sub", "audio.m4a", "Song.mp3"]);
    assert!(items[0].is_dir());

    let m4a = items
        .iter()
        .find(|item| item_name(item) == "audio.m4a")
        .and_then(LibraryItem::as_track)
        .expect("m4a track present");
    assert_eq!(m4a.title, "audio", "fallback title is the file stem");
    assert_eq!(m4a.artist_label(), rayap_musik::models::UNKNOWN_ARTIST);
    assert_eq!(m4a.album_label(), rayap_musik::models::UNKNOWN_ALBUM);
    assert_eq!(m4a.track_number, None);
}

#[test]
fn list_directory_ignores_symlinks() {
    let temp = TempDir::new();
    temp.file("real.mp3");
    let link = temp.path().join("link.mp3");
    let dir_link = temp.path().join("link_dir");

    std::os::unix::fs::symlink(temp.path().join("missing.mp3"), &link).expect("symlink file");
    std::os::unix::fs::symlink(temp.path(), &dir_link).expect("symlink dir");

    let items = list_directory(temp.path()).expect("list directory");
    let names: Vec<String> = items.iter().map(item_name).collect();

    assert_eq!(names, vec!["real.mp3"]);
}

#[test]
fn scan_recursive_is_depth_first_and_deterministic() {
    let temp = TempDir::new();
    temp.file("z-root.mp3");
    temp.file("A_dir/01.mp3");
    temp.file("b_dir/01.mp3");
    temp.file("b_dir/02.mp3");
    temp.file("b_dir/notes.txt");
    temp.file("A_dir/nested/deep.mp3");

    let files = scan_recursive(temp.path());
    let relative: Vec<String> = files
        .iter()
        .map(|path| {
            path.strip_prefix(temp.path())
                .expect("strip prefix")
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    // "nested" is a directory inside A_dir, so it is traversed before A_dir's
    // own files (directories before files at every level).
    assert_eq!(
        relative,
        vec![
            "A_dir/nested/deep.mp3",
            "A_dir/01.mp3",
            "b_dir/01.mp3",
            "b_dir/02.mp3",
            "z-root.mp3",
        ]
    );
}

#[test]
fn scan_recursive_skips_symlinked_directories() {
    let temp = TempDir::new();
    let outside = TempDir::new();
    temp.file("real.mp3");
    fs::write(outside.path().join("hidden.mp3"), b"").expect("write outside file");
    std::os::unix::fs::symlink(outside.path(), temp.path().join("linked")).expect("symlink dir");

    let files = scan_recursive(temp.path());
    let relative: Vec<String> = files
        .iter()
        .map(|path| {
            path.strip_prefix(temp.path())
                .expect("strip prefix")
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert_eq!(relative, vec!["real.mp3"]);
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
