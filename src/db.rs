//! Local SQLite persistence for the library index and playlists.
//!
//! The database is embedded (no server). Schema changes are applied through a
//! versioned migration path driven by `PRAGMA user_version`.

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::{Playlist, Track};

/// Current schema version. Bump this when adding migrations below.
const SCHEMA_VERSION: i64 = 1;

/// Statements that bring a database from any earlier version to v1.
const SCHEMA_V1: &str = r#"
CREATE TABLE tracks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT NOT NULL UNIQUE,
    title        TEXT NOT NULL,
    artist       TEXT,
    album        TEXT,
    track_number INTEGER,
    duration_ms  INTEGER,
    available    INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    PRIMARY KEY (playlist_id, position)
);

CREATE INDEX idx_tracks_path ON tracks(path);
CREATE INDEX idx_playlist_tracks_playlist ON playlist_tracks(playlist_id);
"#;

#[derive(Debug)]
pub struct DbError(String);

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        DbError(error.to_string())
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (or create) the database at `path`, applying migrations.
    pub fn open(path: &Path) -> Result<Db, DbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                DbError(format!(
                    "cannot create database directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        let conn = Connection::open(path).map_err(|error| {
            DbError(format!("cannot open database {}: {error}", path.display()))
        })?;
        Db::from_connection(conn)
    }

    /// Open an in-memory database, used by tests.
    pub fn open_in_memory() -> Result<Db, DbError> {
        Db::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Db, DbError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let mut db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&mut self) -> Result<(), DbError> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if version < 1 {
            self.conn.execute_batch(SCHEMA_V1)?;
        }

        if version < SCHEMA_VERSION {
            self.conn
                .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(())
    }

    /// Insert a track, or refresh its metadata if it already exists by path.
    ///
    /// Returns the row id.
    pub fn upsert_track(&self, track: &Track) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT INTO tracks (path, title, artist, album, track_number, duration_ms, available)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
             ON CONFLICT(path) DO UPDATE SET
               title        = excluded.title,
               artist       = excluded.artist,
               album        = excluded.album,
               track_number = excluded.track_number,
               duration_ms  = excluded.duration_ms,
               available    = 1",
            params![
                track.path.to_string_lossy().into_owned(),
                track.title,
                track.artist,
                track.album,
                track.track_number.map(i64::from),
                track.duration.map(duration_ms),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Mark a track as unavailable (missing on disk) without deleting it.
    pub fn mark_unavailable(&self, id: i64) -> Result<(), DbError> {
        self.conn
            .execute("UPDATE tracks SET available = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_track(&self, id: i64) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn track_id_by_path(&self, path: &Path) -> Result<Option<i64>, DbError> {
        self.conn
            .query_row(
                "SELECT id FROM tracks WHERE path = ?1",
                params![path.to_string_lossy().into_owned()],
                |row| row.get(0),
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn all_tracks(&self) -> Result<Vec<Track>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT path, title, artist, album, track_number, duration_ms
             FROM tracks ORDER BY id",
        )?;
        let rows = stmt.query_map([], track_from_row)?;
        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row?);
        }
        Ok(tracks)
    }

    pub fn create_playlist(&self, name: &str) -> Result<i64, DbError> {
        self.conn
            .execute("INSERT INTO playlists (name) VALUES (?1)", params![name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE playlists SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        Ok(())
    }

    pub fn delete_playlist(&self, id: i64) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_playlists(&self) -> Result<Vec<Playlist>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM playlists ORDER BY name COLLATE NOCASE, id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut playlists = Vec::new();
        for row in rows {
            playlists.push(row?);
        }
        Ok(playlists)
    }

    /// Append a track to the end of a playlist.
    pub fn add_track_to_playlist(&self, playlist_id: i64, track_id: i64) -> Result<(), DbError> {
        let position: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO playlist_tracks (playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
            params![playlist_id, position, track_id],
        )?;
        Ok(())
    }

    pub fn remove_track_from_playlist(
        &self,
        playlist_id: i64,
        track_id: i64,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    /// Tracks of a playlist in their stored order.
    pub fn playlist_tracks(&self, playlist_id: i64) -> Result<Vec<Track>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.path, t.title, t.artist, t.album, t.track_number, t.duration_ms
             FROM playlist_tracks pt
             JOIN tracks t ON t.id = pt.track_id
             WHERE pt.playlist_id = ?1
             ORDER BY pt.position",
        )?;
        let rows = stmt.query_map(params![playlist_id], track_from_row)?;
        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row?);
        }
        Ok(tracks)
    }
}

fn duration_ms(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

fn track_from_row(row: &Row<'_>) -> rusqlite::Result<Track> {
    let path: String = row.get("path")?;
    let title: String = row.get("title")?;
    let artist: Option<String> = row.get("artist")?;
    let album: Option<String> = row.get("album")?;
    let track_number: Option<i64> = row.get("track_number")?;
    let duration_ms: Option<i64> = row.get("duration_ms")?;

    Ok(Track {
        path: PathBuf::from(path),
        title,
        artist,
        album,
        track_number: track_number.and_then(|value| u32::try_from(value).ok()),
        duration: duration_ms
            .filter(|value| *value >= 0)
            .map(|value| Duration::from_millis(value as u64)),
    })
}
