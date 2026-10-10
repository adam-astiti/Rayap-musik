//! Integration tests for queue building, navigation, and previous rules.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rayap_musik::library::build_queue;
use rayap_musik::models::{previous_action, PreviousAction, Queue, Track};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-queue-{}-{}",
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

fn track(name: &str) -> Track {
    Track {
        path: PathBuf::from(name),
        title: name.to_string(),
        artist: None,
        album: None,
        track_number: None,
        duration: None,
    }
}

fn file_names(tracks: &[Track]) -> Vec<String> {
    tracks
        .iter()
        .map(|track| track.file_name())
        .collect::<Vec<_>>()
}

#[test]
fn build_queue_puts_selected_first_and_scans_subtree_only() {
    let temp = TempDir::new();
    let selected = temp.file("Album A/03.mp3");
    temp.file("Album A/01.mp3");
    temp.file("Album A/02.mp3");
    temp.file("Other/x.mp3");

    let queue = build_queue(&selected);
    assert_eq!(file_names(&queue), vec!["03.mp3", "01.mp3", "02.mp3"]);
}

#[test]
fn build_queue_has_no_duplicates() {
    let temp = TempDir::new();
    let selected = temp.file("Album A/02.mp3");
    temp.file("Album A/01.mp3");
    temp.file("Album A/03.mp3");

    let queue = build_queue(&selected);
    let mut paths: Vec<&PathBuf> = queue.iter().map(|track| &track.path).collect();
    let before = paths.len();
    paths.sort();
    paths.dedup();
    assert_eq!(before, paths.len(), "selected track must not appear twice");
    assert_eq!(before, 3);
}

#[test]
fn build_queue_single_track_directory_returns_only_selected() {
    let temp = TempDir::new();
    let selected = temp.file("Solo/only.mp3");

    let queue = build_queue(&selected);
    assert_eq!(file_names(&queue), vec!["only.mp3"]);
}

#[test]
fn queue_navigation_is_bounded() {
    let mut queue = Queue::from_tracks(vec![track("a.mp3"), track("b.mp3"), track("c.mp3")]);

    assert_eq!(queue.current_index(), 0);
    assert!(queue.advance());
    assert_eq!(queue.current_index(), 1);
    assert!(queue.advance());
    assert_eq!(queue.current_index(), 2);
    // At the end: no auto-repeat, no further advance.
    assert!(!queue.advance());
    assert_eq!(queue.current_index(), 2);

    assert!(queue.retreat());
    assert_eq!(queue.current_index(), 1);
    assert!(queue.retreat());
    assert_eq!(queue.current_index(), 0);
    assert!(!queue.retreat());
    assert_eq!(queue.current_index(), 0);

    queue.set_current(1);
    assert_eq!(queue.current_index(), 1);
    queue.set_current(99);
    assert_eq!(queue.current_index(), 1, "out-of-range index is ignored");
}

#[test]
fn advancing_through_all_tracks_terminates() {
    let mut queue = Queue::from_tracks((0..5).map(|i| track(&format!("{i}.mp3"))).collect());

    let mut steps = 0;
    while queue.advance() {
        steps += 1;
        assert!(steps < 1000, "queue traversal must terminate");
    }

    assert_eq!(steps, 4);
    assert_eq!(queue.current_index(), 4);
}

#[test]
fn empty_queue_is_safe() {
    let queue = Queue::default();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.current(), None);
    assert_eq!(queue.next_index(), None);
    assert_eq!(queue.previous_index(), None);
}

#[test]
fn previous_action_follows_three_second_rule() {
    let threshold = Duration::from_secs(3);

    assert_eq!(
        previous_action(Duration::from_secs(4), true, threshold),
        PreviousAction::Restart
    );
    assert_eq!(
        previous_action(Duration::from_secs(2), true, threshold),
        PreviousAction::Previous
    );
    // Exactly 3 seconds counts as "3 seconds or less".
    assert_eq!(
        previous_action(Duration::from_secs(3), true, threshold),
        PreviousAction::Previous
    );
    // No previous track always restarts, regardless of position.
    assert_eq!(
        previous_action(Duration::from_secs(1), false, threshold),
        PreviousAction::Restart
    );
    assert_eq!(
        previous_action(Duration::from_secs(10), false, threshold),
        PreviousAction::Restart
    );
}
