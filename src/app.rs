//! Application state and decision making.

use std::path::PathBuf;
use std::time::Duration;

use crate::catalog::{self, Catalog};
use crate::db::Db;
use crate::indexer::{IndexEvent, Indexer};
use crate::models::{
    previous_action, Action, Focus, LibraryView, PlaybackState, Playlist, PreviousAction, Queue,
    Track,
};
use crate::player::Player;

pub const DEFAULT_VOLUME: u8 = 70;
pub const VOLUME_STEP: u8 = 5;

/// Apply a volume change, clamping the result to the 0..=100 range.
pub fn change_volume(current: u8, delta: i8) -> u8 {
    let value = if delta >= 0 {
        current.saturating_add(delta as u8)
    } else {
        current.saturating_sub(delta.unsigned_abs())
    };
    value.min(100)
}
/// Threshold used by the previous/restart rule (FR-07).
const RESTART_THRESHOLD: Duration = Duration::from_secs(3);

pub struct App {
    /// Validated root directory.
    pub root: PathBuf,
    /// Active library section.
    pub view: LibraryView,
    /// Sorted and grouped view of the indexed tracks.
    pub catalog: Catalog,
    /// Persisted playlists (names only; membership is read on demand).
    pub playlists: Vec<Playlist>,
    /// Current global-search query (empty when not searching).
    pub search_query: String,
    /// Results for the current search query.
    pub search_results: Vec<catalog::SearchEntry>,
    /// Selection index within the library panel.
    pub selected: usize,
    pub focus: Focus,
    /// In-memory playback queue.
    pub queue: Queue,
    /// Selection index within the queue panel.
    pub queue_selected: usize,
    pub playback: PlaybackState,
    pub volume: u8,
    pub should_quit: bool,
    pub status: String,
    player: Option<Player>,
    db: Option<Db>,
    indexer: Option<Indexer>,
}

impl App {
    pub fn new(root: PathBuf, volume: u8, db: Option<Db>) -> Self {
        let volume = volume.min(100);
        let (player, player_error) = match Player::new() {
            Ok(player) => {
                player.set_volume_percent(volume);
                (Some(player), None)
            }
            Err(error) => (None, Some(error.to_string())),
        };

        let indexer = if db.is_some() {
            Some(Indexer::new(root.clone()))
        } else {
            None
        };

        let mut app = Self {
            root,
            view: LibraryView::Songs,
            catalog: Catalog::default(),
            playlists: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            selected: 0,
            focus: Focus::Library,
            queue: Queue::default(),
            queue_selected: 0,
            playback: PlaybackState::default(),
            volume,
            should_quit: false,
            status: String::new(),
            player,
            db,
            indexer,
        };
        app.reload_catalog();
        if let Some(error) = player_error {
            // Surface the audio failure instead of the directory status.
            app.status = format!("Audio unavailable: {error}");
        }
        app
    }

    pub fn audio_available(&self) -> bool {
        self.player.is_some()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue.current()
    }

    pub fn position(&self) -> Option<Duration> {
        self.player
            .as_ref()
            .filter(|player| player.has_track())
            .map(Player::position)
    }

    pub fn duration(&self) -> Option<Duration> {
        self.player.as_ref().and_then(Player::duration)
    }

    /// Number of selectable items in the active library view.
    pub fn library_len(&self) -> usize {
        match self.view {
            LibraryView::Songs => self.catalog.songs.len(),
            LibraryView::Albums => self.catalog.albums.len(),
            LibraryView::Artists => self.catalog.artists.len(),
            LibraryView::Playlists => self.playlists.len(),
            LibraryView::Search => self.search_results.len(),
        }
    }

    /// Whether the app is currently capturing text for the search query.
    pub fn is_searching(&self) -> bool {
        self.view == LibraryView::Search
    }

    /// The selected library entry as a single song, when it is one.
    fn selected_song(&self) -> Option<Track> {
        match self.view {
            LibraryView::Songs => self.catalog.songs.get(self.selected).cloned(),
            LibraryView::Search => match self.search_results.get(self.selected) {
                Some(catalog::SearchEntry::Song(track)) => Some(track.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Apply an action to the state.
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectUp => self.move_selection(-1),
            Action::SelectDown => self.move_selection(1),
            Action::NextFocus => {
                self.focus = self.focus.next();
                self.status = format!("Focus: {}", self.focus.label());
            }
            Action::Enter => self.enter(),
            Action::TogglePlay => self.toggle_play(),
            Action::NextTrack => self.next(),
            Action::PreviousTrack => self.previous(),
            Action::VolumeUp => self.set_volume(change_volume(self.volume, VOLUME_STEP as i8)),
            Action::VolumeDown => self.set_volume(change_volume(self.volume, -(VOLUME_STEP as i8))),
            Action::Rescan => self.rescan_library(),
            Action::SwitchView(view) => self.switch_view(view),
            Action::StartSearch => self.start_search(),
            Action::SearchChar(ch) => self.type_search_char(ch),
            Action::SearchBackspace => self.search_backspace(),
            Action::SearchCancel => self.search_cancel(),
            Action::Quit => self.should_quit = true,
            Action::Resize => {}
        }
    }

    /// Poll the player for state changes and drain indexer events. Called once
    /// per UI frame.
    pub fn tick(&mut self) {
        self.drain_indexer();
        if self.queue.is_empty() {
            return;
        }
        let (finished, paused) = match &self.player {
            Some(player) => (player.is_finished(), player.is_paused()),
            None => return,
        };

        if finished {
            if self.playback != PlaybackState::Stopped {
                // Mark stopped first so a finished track is only reported once.
                self.playback = PlaybackState::Stopped;
                self.advance_or_stop();
            }
        } else if paused {
            self.playback = PlaybackState::Paused;
        } else {
            self.playback = PlaybackState::Playing;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let (selected, len) = match self.focus {
            Focus::Library => (self.selected, self.library_len()),
            Focus::Queue => (self.queue_selected, self.queue.len()),
        };
        if len == 0 {
            return;
        }

        let next = (selected as i32 + delta).clamp(0, len as i32 - 1) as usize;
        match self.focus {
            Focus::Library => self.selected = next,
            Focus::Queue => self.queue_selected = next,
        }
    }

    fn switch_view(&mut self, view: LibraryView) {
        self.view = view;
        self.selected = 0;
        self.search_query.clear();
        self.search_results.clear();
        self.status = view.label().to_string();
    }

    fn start_search(&mut self) {
        self.view = LibraryView::Search;
        self.selected = 0;
        self.search_query.clear();
        self.search_results.clear();
        self.status = "Search: type to filter".to_string();
    }

    fn type_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.selected = 0;
        self.refresh_search();
    }

    fn search_backspace(&mut self) {
        self.search_query.pop();
        self.selected = 0;
        self.refresh_search();
    }

    fn search_cancel(&mut self) {
        self.view = LibraryView::Songs;
        self.search_query.clear();
        self.search_results.clear();
        self.selected = 0;
        self.status = "Search cancelled".to_string();
    }

    fn refresh_search(&mut self) {
        self.search_results = catalog::search(&self.catalog, &self.playlists, &self.search_query);
    }

    /// Rebuild the in-memory catalog and playlist list from the database.
    fn reload_catalog(&mut self) {
        self.catalog = match &self.db {
            Some(db) => db
                .list_available_tracks()
                .map(catalog::build_catalog)
                .unwrap_or_default(),
            None => Catalog::default(),
        };
        self.playlists = match &self.db {
            Some(db) => db.list_playlists().unwrap_or_default(),
            None => Vec::new(),
        };
        self.selected = self.selected.min(self.library_len().saturating_sub(1));
        self.refresh_search();
    }

    fn rescan_library(&mut self) {
        match &self.indexer {
            Some(indexer) => {
                indexer.rescan();
                self.status = "Rescanning library...".to_string();
            }
            None => self.status = "Library index unavailable".to_string(),
        }
    }

    /// Consume indexer events and apply them to the database and status line.
    fn drain_indexer(&mut self) {
        let mut events = Vec::new();
        if let Some(indexer) = &self.indexer {
            while let Some(event) = indexer.poll() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                IndexEvent::ScanStarted => self.status = "Indexing library...".to_string(),
                IndexEvent::ScanCompleted { tracks } => {
                    let count = tracks.len();
                    match &mut self.db {
                        Some(db) => match db.sync_tracks(&tracks) {
                            Ok(()) => self.status = format!("Indexed {count} track(s)"),
                            Err(error) => self.status = format!("Index write failed: {error}"),
                        },
                        None => self.status = format!("Indexed {count} track(s) (not persisted)"),
                    }
                    self.reload_catalog();
                }
                IndexEvent::WatcherError { error } => {
                    self.status = format!("Filesystem watcher unavailable: {error}");
                }
            }
        }
    }

    fn enter(&mut self) {
        match self.focus {
            Focus::Library => self.enter_library(),
            Focus::Queue => {
                if self.queue_selected < self.queue.len() {
                    self.queue.set_current(self.queue_selected);
                    self.start_playback();
                }
            }
        }
    }

    fn enter_library(&mut self) {
        match self.view {
            LibraryView::Songs => {
                if let Some(track) = self.catalog.songs.get(self.selected).cloned() {
                    self.play_now(vec![track]);
                }
            }
            LibraryView::Albums => {
                if let Some(album) = self.catalog.albums.get(self.selected) {
                    self.play_now(album.tracks.clone());
                }
            }
            LibraryView::Artists => {
                if let Some(artist) = self.catalog.artists.get(self.selected) {
                    self.play_now(artist.tracks.clone());
                }
            }
            LibraryView::Playlists => {
                if let Some(id) = self.playlists.get(self.selected).map(|p| p.id) {
                    let tracks = self
                        .db
                        .as_ref()
                        .and_then(|db| db.playlist_tracks(id).ok())
                        .unwrap_or_default();
                    self.play_now(tracks);
                }
            }
            LibraryView::Search => match self.search_results.get(self.selected).cloned() {
                Some(catalog::SearchEntry::Song(track)) => self.play_now(vec![track]),
                Some(catalog::SearchEntry::Album(album)) => self.play_now(album.tracks),
                Some(catalog::SearchEntry::Artist(artist)) => self.play_now(artist.tracks),
                Some(catalog::SearchEntry::Playlist(playlist)) => {
                    let tracks = self
                        .db
                        .as_ref()
                        .and_then(|db| db.playlist_tracks(playlist.id).ok())
                        .unwrap_or_default();
                    self.play_now(tracks);
                }
                None => {}
            },
        }
    }

    /// Replace the queue with the given collection and start playing (FR-PLAY-03).
    ///
    /// An empty source leaves the current queue untouched and shows a message.
    fn play_now(&mut self, tracks: Vec<Track>) {
        if tracks.is_empty() {
            self.status = "No playable tracks".to_string();
            return;
        }
        self.queue = Queue::from_tracks(tracks);
        self.queue_selected = 0;
        self.start_playback();
    }

    /// Play the current queue item, skipping any that fail to play (FR-06).
    ///
    /// The search is bounded by the queue length, so a queue where every track
    /// fails terminates instead of looping forever.
    fn start_playback(&mut self) {
        let len = self.queue.len();
        let mut index = self.queue.current_index();

        while index < len {
            self.queue.set_current(index);
            let Some(track) = self.queue.current().cloned() else {
                break;
            };

            match self.try_play(&track) {
                Ok(()) => {
                    self.playback = PlaybackState::Playing;
                    self.status = format!("Playing: {} ({}/{})", track.title, index + 1, len);
                    return;
                }
                Err(message) => {
                    self.status = format!("Skipped {}: {message}", track.title);
                }
            }
            index += 1;
        }

        self.stop_playback("All tracks in queue failed");
    }

    fn try_play(&mut self, track: &Track) -> Result<(), String> {
        match self.player.as_mut() {
            Some(player) => {
                player
                    .play_path(&track.path)
                    .map_err(|error| error.to_string())?;
                player.set_volume_percent(self.volume);
                Ok(())
            }
            None => Err("audio backend unavailable".to_string()),
        }
    }

    fn stop_playback(&mut self, message: &str) {
        if let Some(player) = &mut self.player {
            player.stop();
        }
        self.playback = PlaybackState::Stopped;
        self.status = message.to_string();
    }

    fn advance_or_stop(&mut self) {
        if self.queue.advance() {
            self.queue_selected = self.queue.current_index();
            self.start_playback();
        } else {
            self.stop_playback("End of queue");
        }
    }

    fn next(&mut self) {
        if self.queue.is_empty() {
            self.status = "Queue is empty".to_string();
            return;
        }
        self.advance_or_stop();
    }

    /// Previous/restart behaviour (FR-07).
    fn previous(&mut self) {
        if self.queue.is_empty() {
            self.status = "Queue is empty".to_string();
            return;
        }

        let position = self
            .player
            .as_ref()
            .map(Player::position)
            .unwrap_or_default();
        let has_previous = self.queue.previous_index().is_some();

        match previous_action(position, has_previous, RESTART_THRESHOLD) {
            PreviousAction::Restart => {
                self.start_playback();
            }
            PreviousAction::Previous => {
                self.queue.retreat();
                self.queue_selected = self.queue.current_index();
                self.start_playback();
            }
        }
    }

    fn toggle_play(&mut self) {
        let finished = self
            .player
            .as_ref()
            .map(|player| player.is_finished())
            .unwrap_or(false);
        let has_track = self.player.as_ref().map(Player::has_track).unwrap_or(false);

        if !has_track || finished {
            if !self.queue.is_empty() {
                self.start_playback();
            } else if let Some(track) = self.selected_song() {
                self.play_now(vec![track]);
            }
            return;
        }

        if let Some(player) = &self.player {
            if player.is_paused() {
                player.play();
                self.playback = PlaybackState::Playing;
                self.status = "Resumed".to_string();
            } else {
                player.pause();
                self.playback = PlaybackState::Paused;
                self.status = "Paused".to_string();
            }
        }
    }

    fn set_volume(&mut self, value: u8) {
        self.volume = value.min(100);
        if let Some(player) = &self.player {
            player.set_volume_percent(self.volume);
        }
        self.status = format!("Volume: {}%", self.volume);
    }
}

#[cfg(test)]
mod tests {
    use super::{change_volume, DEFAULT_VOLUME, VOLUME_STEP};

    #[test]
    fn volume_clamps_to_zero() {
        let mut volume = DEFAULT_VOLUME;
        for _ in 0..100 {
            volume = change_volume(volume, -(VOLUME_STEP as i8));
        }
        assert_eq!(volume, 0);
    }

    #[test]
    fn volume_clamps_to_one_hundred() {
        let mut volume = DEFAULT_VOLUME;
        for _ in 0..100 {
            volume = change_volume(volume, VOLUME_STEP as i8);
        }
        assert_eq!(volume, 100);
    }

    #[test]
    fn volume_step_is_five() {
        assert_eq!(change_volume(70, 5), 75);
        assert_eq!(change_volume(70, -5), 65);
        assert_eq!(change_volume(3, -5), 0, "downwards step clamps at zero");
        assert_eq!(change_volume(98, 5), 100, "upwards step clamps at 100");
    }
}
