//! Shared data types used across modules.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Fallback text shown when a metadata field is unavailable.
pub const UNKNOWN_ARTIST: &str = "Unknown artist";
pub const UNKNOWN_ALBUM: &str = "Unknown album";

/// Which panel currently owns keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Library,
    Queue,
}

impl Focus {
    /// Cycle focus to the next panel (Library -> Queue -> Library).
    pub fn next(self) -> Self {
        match self {
            Focus::Library => Focus::Queue,
            Focus::Queue => Focus::Library,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Focus::Library => "Library",
            Focus::Queue => "Queue",
        }
    }
}

/// Playback status shown to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

impl PlaybackState {
    pub fn label(self) -> &'static str {
        match self {
            PlaybackState::Stopped => "Stopped",
            PlaybackState::Playing => "Playing",
            PlaybackState::Paused => "Paused",
        }
    }
}

/// A single audio track. The file path is the identity of a track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub duration: Option<Duration>,
}

impl Track {
    /// Raw file name including extension, used as a sorting tie-breaker.
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn artist_label(&self) -> &str {
        self.artist.as_deref().unwrap_or(UNKNOWN_ARTIST)
    }

    pub fn album_label(&self) -> &str {
        self.album.as_deref().unwrap_or(UNKNOWN_ALBUM)
    }
}

/// An entry in the filesystem browser: either a subdirectory or an audio track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryItem {
    Directory { path: PathBuf, name: String },
    Track(Track),
}

impl LibraryItem {
    pub fn name(&self) -> &str {
        match self {
            LibraryItem::Directory { name, .. } => name,
            LibraryItem::Track(track) => &track.title,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            LibraryItem::Directory { path, .. } => path,
            LibraryItem::Track(track) => &track.path,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, LibraryItem::Directory { .. })
    }

    pub fn as_track(&self) -> Option<&Track> {
        match self {
            LibraryItem::Track(track) => Some(track),
            LibraryItem::Directory { .. } => None,
        }
    }
}

/// A user intent derived from keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SelectUp,
    SelectDown,
    NextFocus,
    Enter,
    GoUp,
    TogglePlay,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    Quit,
    Resize,
}
