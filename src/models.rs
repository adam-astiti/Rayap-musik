//! Shared data types used across modules.

use std::path::PathBuf;
use std::time::Duration;

/// Fallback text shown when a metadata field is unavailable.
pub const UNKNOWN_ARTIST: &str = "Unknown artist";
pub const UNKNOWN_ALBUM: &str = "Unknown album";

/// How raw keyboard events are interpreted for the active interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation and shortcuts.
    #[default]
    Normal,
    /// Editing the global search query.
    SearchEdit,
    /// Navigating the search results (query retained but frozen).
    SearchResults,
    /// Editing a prompt (playlist name, M3U path).
    Prompt,
    /// Searching the Help page.
    Help,
}

/// Wrap-around selection movement for a list of `len` items.
///
/// Returns `0` for empty and single-item lists. Otherwise moves `delta` steps
/// from `current`, wrapping around both ends.
pub fn wrap_selection(current: usize, delta: i32, len: usize) -> usize {
    if len < 2 {
        return 0;
    }
    let len = len as i32;
    let mut next = (current as i32 + delta) % len;
    if next < 0 {
        next += len;
    }
    next as usize
}

/// Remove duplicate tracks (by file path), keeping the first occurrence.
pub fn dedupe_by_path(tracks: Vec<Track>) -> Vec<Track> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::with_capacity(tracks.len());
    for track in tracks {
        if seen.insert(track.path.clone()) {
            result.push(track);
        }
    }
    result
}

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

/// Playback repeat behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    /// Cycle to the next mode (Off -> All -> One -> Off).
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "One",
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
///
/// The queue keeps two orderings: `original` (the canonical, unshuffled order
/// that mutations are applied to) and `order` (the playback order, equal to
/// `original` unless shuffle is enabled). `current` indexes into `order`.
#[derive(Debug, Clone, Default)]
pub struct Queue {
    original: Vec<Track>,
    order: Vec<Track>,
    current: usize,
    shuffled: bool,
}

impl Queue {
    pub fn from_tracks(tracks: Vec<Track>) -> Self {
        Self {
            original: tracks.clone(),
            order: tracks,
            current: 0,
            shuffled: false,
        }
    }

    pub fn tracks(&self) -> &[Track] {
        &self.order
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn is_shuffled(&self) -> bool {
        self.shuffled
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn current(&self) -> Option<&Track> {
        self.order.get(self.current)
    }

    pub fn set_current(&mut self, index: usize) {
        if index < self.order.len() {
            self.current = index;
        }
    }

    pub fn next_index(&self) -> Option<usize> {
        (self.current + 1 < self.order.len()).then_some(self.current + 1)
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

    /// Append tracks to the end of the queue (Add to Queue).
    pub fn append(&mut self, tracks: Vec<Track>) {
        self.original.extend(tracks.iter().cloned());
        self.order.extend(tracks);
    }

    /// Insert tracks directly after the active track (Play Next).
    ///
    /// Returns `false` when the queue is empty and there is no active track.
    pub fn insert_after_current(&mut self, tracks: Vec<Track>) -> bool {
        if self.order.is_empty() {
            return false;
        }

        let active_path = self.order[self.current].path.clone();
        self.order
            .splice(self.current + 1..self.current + 1, tracks.iter().cloned());

        if let Some(pos) = self
            .original
            .iter()
            .position(|track| track.path == active_path)
        {
            self.original.splice(pos + 1..pos + 1, tracks);
        } else {
            self.original.extend(tracks);
        }
        true
    }

    /// Remove the track at the given playback index, returning it.
    pub fn remove(&mut self, index: usize) -> Option<Track> {
        if index >= self.order.len() {
            return None;
        }

        let removed = self.order.remove(index);
        if let Some(pos) = self
            .original
            .iter()
            .position(|track| track.path == removed.path)
        {
            self.original.remove(pos);
        }

        match index.cmp(&self.current) {
            std::cmp::Ordering::Less => self.current -= 1,
            std::cmp::Ordering::Equal => {
                self.current = self.current.min(self.order.len().saturating_sub(1));
            }
            std::cmp::Ordering::Greater => {}
        }
        Some(removed)
    }

    /// Enable or disable shuffle.
    ///
    /// Enabling shuffles the whole queue while keeping the active track in
    /// place. Disabling restores the original order without changing the
    /// active track.
    pub fn set_shuffle(&mut self, on: bool) {
        if on == self.shuffled {
            return;
        }

        if on {
            let seed = time_seed();
            self.order = shuffle_keep_current(&self.order, self.current, seed);
            self.shuffled = true;
        } else {
            let active = self.current().map(|track| track.path.clone());
            self.order = self.original.clone();
            if let Some(active) = active {
                if let Some(pos) = self.order.iter().position(|track| track.path == active) {
                    self.current = pos;
                }
            }
            self.shuffled = false;
        }
    }
}

/// Shuffle a playback order while keeping the track at `current` in place.
fn shuffle_keep_current(order: &[Track], current: usize, seed: u64) -> Vec<Track> {
    let len = order.len();
    let mut result = order.to_vec();
    if len <= 1 {
        return result;
    }

    let mut state = seed;
    for i in (1..len).rev() {
        let r = next_random(&mut state) % (i as u64 + 1);
        result.swap(i, r as usize);
    }

    let active_path = order[current].path.clone();
    if let Some(pos) = result.iter().position(|track| track.path == active_path) {
        result.swap(pos, current);
    }
    result
}

fn time_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

/// xorshift64* pseudo-random generator, sufficient for shuffle.
fn next_random(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
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
    Back,
    TogglePlay,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    Rescan,
    SwitchView(LibraryView),
    StartSearch,
    SwitchSearchCategory(SearchCategory),
    ToggleSearchFocus,
    TypeChar(char),
    DeleteChar,
    Cancel,
    PlayNext,
    AddToQueue,
    AddAllToQueue,
    RemoveFromQueue,
    ToggleShuffle,
    CycleRepeat,
    TogglePlayMode,
    ToggleHelp,
    CycleTheme,
    NewPlaylist,
    Rename,
    Duplicate,
    Delete,
    AddToPlaylist,
    Import,
    Export,
    MoveDown,
    MoveUp,
    Sort,
    Quit,
    Resize,
}

/// The active result category within the Search interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchCategory {
    #[default]
    Songs,
    Albums,
    Artists,
    Playlists,
}

impl SearchCategory {
    pub fn label(self) -> &'static str {
        match self {
            SearchCategory::Songs => "Songs",
            SearchCategory::Albums => "Albums",
            SearchCategory::Artists => "Artists",
            SearchCategory::Playlists => "Playlists",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{dedupe_by_path, wrap_selection};
    use crate::models::Track;
    use std::path::PathBuf;

    fn track(path: &str) -> Track {
        Track {
            path: PathBuf::from(path),
            title: path.to_string(),
            artist: None,
            album: None,
            track_number: None,
            duration: None,
        }
    }

    #[test]
    fn dedupe_keeps_first_occurrence() {
        let tracks = vec![track("a.mp3"), track("b.mp3"), track("a.mp3")];
        let result = dedupe_by_path(tracks);
        let paths: Vec<String> = result.iter().map(|t| t.title.clone()).collect();
        assert_eq!(paths, vec!["a.mp3", "b.mp3"]);
    }
    #[test]
    fn wrap_handles_empty_and_single() {
        assert_eq!(wrap_selection(0, -1, 0), 0);
        assert_eq!(wrap_selection(0, 1, 1), 0);
    }

    #[test]
    fn wrap_moves_forward_and_backward() {
        assert_eq!(wrap_selection(0, 1, 3), 1);
        assert_eq!(wrap_selection(2, 1, 3), 0, "last wraps to first");
        assert_eq!(wrap_selection(0, -1, 3), 2, "first wraps to last");
        assert_eq!(wrap_selection(1, -1, 3), 0);
    }

    #[test]
    fn wrap_handles_multi_step() {
        assert_eq!(wrap_selection(0, 5, 3), 2);
        assert_eq!(wrap_selection(1, -3, 3), 1);
    }
}
