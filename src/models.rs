//! Shared data types used across modules.

use std::path::PathBuf;
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
/// A persisted playlist. Track membership and ordering are stored in the
/// database; this is the lightweight identity used by the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

/// In-memory playback queue. The queue is not persisted between sessions.
#[derive(Debug, Clone, Default)]
pub struct Queue {
    tracks: Vec<Track>,
    current: usize,
}

impl Queue {
    pub fn from_tracks(tracks: Vec<Track>) -> Self {
        Self { tracks, current: 0 }
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn current(&self) -> Option<&Track> {
        self.tracks.get(self.current)
    }

    pub fn set_current(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.current = index;
        }
    }

    pub fn next_index(&self) -> Option<usize> {
        (self.current + 1 < self.tracks.len()).then_some(self.current + 1)
    }

    pub fn previous_index(&self) -> Option<usize> {
        self.current.checked_sub(1)
    }

    /// Move to the next track. Returns `false` when already at the end.
    pub fn advance(&mut self) -> bool {
        match self.next_index() {
            Some(index) => {
                self.current = index;
                true
            }
            None => false,
        }
    }

    /// Move to the previous track. Returns `false` when already at the start.
    pub fn retreat(&mut self) -> bool {
        match self.previous_index() {
            Some(index) => {
                self.current = index;
                true
            }
            None => false,
        }
    }
}

/// Decision for the previous button (FR-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviousAction {
    /// Restart the current track from the beginning.
    Restart,
    /// Move to the previous track.
    Previous,
}

/// Decide what the previous button should do.
///
/// Restart when the track has played for more than `threshold`, or when there
/// is no previous track. Otherwise move to the previous track. `position` is
/// the actual playback position, so pausing does not affect the decision.
pub fn previous_action(
    position: Duration,
    has_previous: bool,
    threshold: Duration,
) -> PreviousAction {
    if position > threshold || !has_previous {
        PreviousAction::Restart
    } else {
        PreviousAction::Previous
    }
}

/// A user intent derived from keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SelectUp,
    SelectDown,
    NextFocus,
    Enter,
    TogglePlay,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    Rescan,
    SwitchView(LibraryView),
    StartSearch,
    SearchChar(char),
    SearchBackspace,
    SearchCancel,
    Quit,
    Resize,
}

/// Which section of the library is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryView {
    #[default]
    Songs,
    Albums,
    Artists,
    Playlists,
    Search,
}

impl LibraryView {
    pub fn label(self) -> &'static str {
        match self {
            LibraryView::Songs => "Songs",
            LibraryView::Albums => "Albums",
            LibraryView::Artists => "Artists",
            LibraryView::Playlists => "Playlists",
            LibraryView::Search => "Search",
        }
    }
}
