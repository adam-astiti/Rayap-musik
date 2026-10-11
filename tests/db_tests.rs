//! Integration tests for SQLite persistence and migrations.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rayap_musik::db::Db;
use rayap_musik::models::Track;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-db-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn track(path: &str, title: &str, number: Option<u32>) -> Track {
    Track {
        path: PathBuf::from(path),
        title: title.to_string(),
        artist: Some("Some Artist".to_string()),
        album: Some("Some Album".to_string()),
        track_number: number,
        duration: Some(Duration::from_secs(123)),
    }
}

#[test]
fn track_round_trips_through_database() {
    let db = Db::open_in_memory().expect("open in-memory db");
    let inserted = track("/music/a.mp3", "A", Some(1));

    let id = db.upsert_track(&inserted).expect("insert track");
    let tracks = db.all_tracks().expect("list tracks");

    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0], inserted);
    assert_eq!(db.track_id_by_path(&inserted.path).unwrap(), Some(id));
}

#[test]
fn upsert_updates_existing_track_by_path() {
    let db = Db::open_in_memory().expect("open in-memory db");
    let original = track("/music/a.mp3", "Old Title", Some(1));

    db.upsert_track(&original).expect("insert");
    let mut updated = original.clone();
    updated.title = "New Title".to_string();
    updated.track_number = Some(2);
    db.upsert_track(&updated).expect("upsert");

    let tracks = db.all_tracks().expect("list");
    assert_eq!(tracks.len(), 1, "path identity prevents duplicates");
    assert_eq!(tracks[0].title, "New Title");
    assert_eq!(tracks[0].track_number, Some(2));
}

#[test]
fn data_survives_close_and_reopen() {
    let temp = TempDir::new();
    let db_path = temp.path.join("library.db");

    {
        let db = Db::open(&db_path).expect("open db");
        db.upsert_track(&track("/music/a.mp3", "A", Some(1)))
            .expect("insert");
    }

    let db = Db::open(&db_path).expect("reopen db");
    let tracks = db.all_tracks().expect("list");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].title, "A");
}

#[test]
fn playlist_crud_and_ordering() {
    let db = Db::open_in_memory().expect("open in-memory db");
    let a = db
        .upsert_track(&track("/music/a.mp3", "A", Some(1)))
        .unwrap();
    let b = db
        .upsert_track(&track("/music/b.mp3", "B", Some(2)))
        .unwrap();

    let playlist = db.create_playlist("Roadtrip").expect("create playlist");
    assert_eq!(db.list_playlists().unwrap().len(), 1);

    db.add_track_to_playlist(playlist, a).expect("add a");
    db.add_track_to_playlist(playlist, b).expect("add b");

    let tracks = db.playlist_tracks(playlist).expect("playlist tracks");
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].title, "A");
    assert_eq!(tracks[1].title, "B");

    db.rename_playlist(playlist, "Long Drive").expect("rename");
    assert_eq!(db.list_playlists().unwrap()[0].name, "Long Drive");

    db.remove_track_from_playlist(playlist, a)
        .expect("remove a");
    assert_eq!(db.playlist_tracks(playlist).unwrap().len(), 1);

    db.delete_playlist(playlist).expect("delete playlist");
    assert!(db.list_playlists().unwrap().is_empty());
}

#[test]
fn deleting_playlist_leaves_tracks_intact() {
    let db = Db::open_in_memory().expect("open in-memory db");
    let id = db
        .upsert_track(&track("/music/a.mp3", "A", Some(1)))
        .unwrap();
    let playlist = db.create_playlist("P").unwrap();
    db.add_track_to_playlist(playlist, id).unwrap();

    db.delete_playlist(playlist).unwrap();
    assert_eq!(
        db.all_tracks().unwrap().len(),
        1,
        "audio file identity survives"
    );
}
