//! Integration tests for queue navigation and previous rules.

use std::path::PathBuf;
use std::time::Duration;

use rayap_musik::models::{previous_action, PreviousAction, Queue, Track};

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
