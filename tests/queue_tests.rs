//! Integration tests for queue navigation and previous rules.

use std::path::PathBuf;
use std::time::Duration;

use rayap_musik::models::{previous_action, PreviousAction, Queue, RepeatMode, Track};

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

fn file_names(queue: &Queue) -> Vec<String> {
    queue.tracks().iter().map(Track::file_name).collect()
}

#[test]
fn append_adds_to_end() {
    let mut queue = Queue::from_tracks(vec![track("a.mp3"), track("b.mp3")]);
    queue.append(vec![track("c.mp3")]);
    assert_eq!(file_names(&queue), vec!["a.mp3", "b.mp3", "c.mp3"]);
}

#[test]
fn insert_after_current_places_after_active() {
    let mut queue = Queue::from_tracks(vec![track("a.mp3"), track("b.mp3"), track("c.mp3")]);
    queue.set_current(1);
    assert!(queue.insert_after_current(vec![track("x.mp3"), track("y.mp3")]));
    assert_eq!(
        file_names(&queue),
        vec!["a.mp3", "b.mp3", "x.mp3", "y.mp3", "c.mp3"]
    );
    assert_eq!(queue.current_index(), 1, "active track unchanged");
}

#[test]
fn insert_after_current_on_empty_fails() {
    let mut queue = Queue::default();
    assert!(!queue.insert_after_current(vec![track("a.mp3")]));
}

#[test]
fn remove_adjusts_current() {
    let mut queue = Queue::from_tracks(vec![
        track("a.mp3"),
        track("b.mp3"),
        track("c.mp3"),
        track("d.mp3"),
    ]);
    queue.set_current(2);

    queue.remove(0);
    assert_eq!(
        queue.current_index(),
        1,
        "removing before active shifts it down"
    );

    queue.remove(2);
    assert_eq!(file_names(&queue), vec!["b.mp3", "c.mp3"]);
    assert_eq!(queue.current_index(), 1);
}

#[test]
fn shuffle_preserves_active_and_restores_order() {
    let tracks: Vec<Track> = (0..5).map(|i| track(&format!("{i}.mp3"))).collect();
    let mut queue = Queue::from_tracks(tracks.clone());
    queue.set_current(2);
    let active = queue.current().expect("active").path.clone();

    queue.set_shuffle(true);
    assert!(queue.is_shuffled());
    assert_eq!(
        queue.current().expect("active").path,
        active,
        "active stays active"
    );

    let mut shuffled = file_names(&queue);
    let mut original = tracks.iter().map(Track::file_name).collect::<Vec<_>>();
    shuffled.sort();
    original.sort();
    assert_eq!(shuffled, original, "shuffle is a permutation");

    queue.set_shuffle(false);
    assert!(!queue.is_shuffled());
    assert_eq!(
        file_names(&queue),
        tracks.iter().map(Track::file_name).collect::<Vec<_>>()
    );
    assert_eq!(
        queue.current().expect("active").path,
        active,
        "active after restore"
    );
}

#[test]
fn insert_after_current_when_shuffled_is_deterministic() {
    let mut queue = Queue::from_tracks(vec![track("a.mp3"), track("b.mp3"), track("c.mp3")]);
    queue.set_current(0);
    queue.set_shuffle(true);
    let active = queue.current().expect("active").path.clone();

    queue.insert_after_current(vec![track("new.mp3")]);

    assert_eq!(queue.current().expect("active").path, active);
    let names = file_names(&queue);
    let pos = names.iter().position(|name| name == "new.mp3").unwrap();
    assert_eq!(
        pos,
        queue.current_index() + 1,
        "inserted right after active"
    );

    queue.set_shuffle(false);
    assert_eq!(
        file_names(&queue),
        vec!["a.mp3", "new.mp3", "b.mp3", "c.mp3"]
    );
}

#[test]
fn repeat_mode_cycles() {
    assert_eq!(RepeatMode::Off.next(), RepeatMode::All);
    assert_eq!(RepeatMode::All.next(), RepeatMode::One);
    assert_eq!(RepeatMode::One.next(), RepeatMode::Off);
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
