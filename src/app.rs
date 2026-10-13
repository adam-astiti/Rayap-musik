//! Application state and decision making.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::catalog::{self, Catalog};
use crate::db::Db;
use crate::indexer::{IndexEvent, Indexer};
use crate::keys::{self, Context};
use crate::library;
use crate::m3u;
use crate::models::{
    previous_action, wrap_selection, Action, Focus, InputMode, LibraryView, PlaybackState,
    Playlist, PreviousAction, Queue, RepeatMode, SearchCategory, Track,
};
use crate::player::Player;
use crate::theme::{self, Theme};

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

/// A pending text prompt (playlist name, or M3U import/export path).
#[derive(Debug, Clone, PartialEq, Eq)]
enum PromptKind {
    New,
    Rename(i64),
    Export(i64),
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Prompt {
    kind: PromptKind,
    buffer: String,
}

/// A saved navigation point, pushed onto the back stack before entering a
/// detail view or the Search interface so Back can restore it.
#[derive(Debug, Clone)]
struct NavLocation {
    view: LibraryView,
    selected: usize,
    search_query: String,
    search_category: SearchCategory,
    search_editing: bool,
}

pub struct App {
    /// Validated root directory.
    pub root: PathBuf,
    /// Active library section.
    pub view: LibraryView,
    /// Sorted and grouped view of the indexed tracks.
    pub catalog: Catalog,
    /// Persisted playlists (names only; membership is read on demand).
    pub playlists: Vec<Playlist>,
    /// The playlist currently being browsed, if any.
    pub open_playlist: Option<Playlist>,
    /// The album detail currently being browsed, if any.
    pub open_album: Option<catalog::Album>,
    /// The artist detail currently being browsed, if any.
    pub open_artist: Option<catalog::Artist>,
    /// Tracks of the currently browsed detail view (playlist/album/artist).
    pub open_tracks: Vec<Track>,
    /// Current global-search query (empty when not searching).
    pub search_query: String,
    /// The active result category in the Search interface.
    pub search_category: SearchCategory,
    /// Whether the search query is being edited (`true`) or results navigated.
    pub search_editing: bool,
    /// Results for the current search query, grouped by category.
    pub search_results: catalog::SearchResults,
    /// Selection index within the library panel.
    pub selected: usize,
    pub focus: Focus,
    /// In-memory playback queue.
    pub queue: Queue,
    /// Selection index within the queue panel.
    pub queue_selected: usize,
    pub playback: PlaybackState,
    pub volume: u8,
    pub repeat: RepeatMode,
    pub play_mode: bool,
    pub show_help: bool,
    pub theme: Theme,
    pub should_quit: bool,
    pub status: String,
    pub help_query: String,
    pub help_selected: usize,
    player: Option<Player>,
    db: Option<Db>,
    indexer: Option<Indexer>,
    pending_add: Option<Vec<Track>>,
    picker_return: Option<LibraryView>,
    confirm_delete: Option<i64>,
    prompt: Option<Prompt>,
    back_stack: Vec<NavLocation>,
}

impl App {
    pub fn new(root: PathBuf, volume: u8, theme_name: &str, db: Option<Db>) -> Self {
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
            open_playlist: None,
            open_album: None,
            open_artist: None,
            open_tracks: Vec::new(),
            search_query: String::new(),
            search_category: SearchCategory::default(),
            search_editing: true,
            search_results: catalog::SearchResults::default(),
            selected: 0,
            focus: Focus::Library,
            queue: Queue::default(),
            queue_selected: 0,
            playback: PlaybackState::default(),
            volume,
            repeat: RepeatMode::default(),
            play_mode: false,
            show_help: false,
            theme: theme::theme_from_name(theme_name),
            should_quit: false,
            status: String::new(),
            help_query: String::new(),
            help_selected: 0,
            player,
            db,
            indexer,
            pending_add: None,
            picker_return: None,
            confirm_delete: None,
            prompt: None,
            back_stack: Vec::new(),
        };
        app.reload_catalog();
        if let Some(error) = player_error {
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

    /// The input mode that determines how key events are interpreted.
    pub fn input_mode(&self) -> InputMode {
        if self.show_help {
            InputMode::Help
        } else if self.view == LibraryView::Search {
            if self.search_editing {
                InputMode::SearchEdit
            } else {
                InputMode::SearchResults
            }
        } else if self.prompt.is_some() {
            InputMode::Prompt
        } else {
            InputMode::Normal
        }
    }

    /// The interaction context used by the contextual keybinding helper.
    pub fn context(&self) -> Context {
        if self.show_help {
            Context::Help
        } else if self.play_mode {
            Context::PlayMode
        } else if self.prompt.is_some() {
            Context::TextInput
        } else if self.picker_active() {
            Context::Playlists
        } else if self.detail_open() {
            Context::OpenPlaylist
        } else if self.focus == Focus::Queue {
            Context::Queue
        } else {
            match self.view {
                LibraryView::Songs => Context::Songs,
                LibraryView::Albums => Context::Albums,
                LibraryView::Artists => Context::Artists,
                LibraryView::Playlists => Context::Playlists,
                LibraryView::Search => Context::Search,
            }
        }
    }

    /// The help entries visible for the current help query.
    pub fn help_entries(&self) -> Vec<&'static keys::Binding> {
        keys::search(&self.help_query)
    }

    /// Number of entries in the (filtered) help list.
    pub fn help_len(&self) -> usize {
        self.help_entries().len()
    }

    /// Whether a playlist picker is active (choosing a target playlist).
    pub fn picker_active(&self) -> bool {
        self.pending_add.is_some()
    }

    /// Text shown while a prompt is active, for the UI.
    pub fn prompt_display(&self) -> Option<String> {
        self.prompt
            .as_ref()
            .map(|prompt| format!("{}: {}\u{2588}", prompt_label(&prompt.kind), prompt.buffer))
    }

    /// Number of selectable items in the active library panel.
    pub fn library_len(&self) -> usize {
        if self.picker_active() {
            return self.playlists.len();
        }
        if self.detail_open() {
            return self.open_tracks.len();
        }
        match self.view {
            LibraryView::Songs => self.catalog.songs.len(),
            LibraryView::Albums => self.catalog.albums.len(),
            LibraryView::Artists => self.catalog.artists.len(),
            LibraryView::Playlists => self.playlists.len(),
            LibraryView::Search => self.search_results.count(self.search_category),
        }
    }

    /// The selected library entry as a single song, when it is one.
    fn selected_song(&self) -> Option<Track> {
        if self.detail_open() {
            return self.open_tracks.get(self.selected).cloned();
        }
        match self.view {
            LibraryView::Songs => self.catalog.songs.get(self.selected).cloned(),
            LibraryView::Search => {
                if self.search_category == SearchCategory::Songs {
                    self.search_results.songs.get(self.selected).cloned()
                } else {
                    None
                }
            }
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
            Action::Enter => {
                if self.prompt.is_some() {
                    self.commit_prompt();
                } else {
                    self.enter();
                }
            }
            Action::Back => self.go_back(),
            Action::TogglePlay => self.toggle_play(),
            Action::NextTrack => self.next(),
            Action::PreviousTrack => self.previous(),
            Action::VolumeUp => self.set_volume(change_volume(self.volume, VOLUME_STEP as i8)),
            Action::VolumeDown => self.set_volume(change_volume(self.volume, -(VOLUME_STEP as i8))),
            Action::Rescan => self.rescan_library(),
            Action::SwitchView(view) => self.switch_view(view),
            Action::StartSearch => self.start_search(),
            Action::SwitchSearchCategory(category) => self.switch_search_category(category),
            Action::ToggleSearchFocus => self.toggle_search_focus(),
            Action::TypeChar(ch) => self.type_char(ch),
            Action::DeleteChar => self.delete_char(),
            Action::Cancel => self.cancel_text_input(),
            Action::PlayNext => self.play_next(),
            Action::AddToQueue => self.add_to_queue(),
            Action::AddAllToQueue => self.add_all_to_queue(),
            Action::RemoveFromQueue => self.remove_from_queue(),
            Action::ToggleShuffle => self.toggle_shuffle(),
            Action::CycleRepeat => self.cycle_repeat(),
            Action::TogglePlayMode => self.toggle_play_mode(),
            Action::ToggleHelp => self.toggle_help(),
            Action::CycleTheme => self.cycle_theme(),
            Action::NewPlaylist => self.begin_prompt(PromptKind::New),
            Action::Rename => self.begin_rename(),
            Action::Duplicate => self.duplicate_selected(),
            Action::Delete => self.delete_selected(),
            Action::AddToPlaylist => self.begin_add_to_playlist(),
            Action::Import => self.begin_prompt(PromptKind::Import),
            Action::Export => self.begin_export(),
            Action::MoveDown => self.move_track(1),
            Action::MoveUp => self.move_track(-1),
            Action::Sort => self.sort_open_playlist(),
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
                self.on_finished();
            }
        } else if paused {
            self.playback = PlaybackState::Paused;
        } else {
            self.playback = PlaybackState::Playing;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.show_help {
            self.help_selected = wrap_selection(self.help_selected, delta, self.help_len());
            return;
        }

        let (selected, len) = match self.focus {
            Focus::Library => (self.selected, self.library_len()),
            Focus::Queue => (self.queue_selected, self.queue.len()),
        };
        if len == 0 {
            return;
        }

        let next = wrap_selection(selected, delta, len);
        match self.focus {
            Focus::Library => {
                self.selected = next;
                self.confirm_delete = None;
            }
            Focus::Queue => self.queue_selected = next,
        }
    }

    fn switch_view(&mut self, view: LibraryView) {
        self.view = view;
        self.selected = 0;
        self.close_detail();
        self.pending_add = None;
        self.picker_return = None;
        self.confirm_delete = None;
        self.prompt = None;
        self.search_query.clear();
        self.search_category = SearchCategory::default();
        self.search_editing = true;
        self.search_results = catalog::SearchResults::default();
        self.back_stack.clear();
        self.status = view.label().to_string();
    }

    fn start_search(&mut self) {
        self.push_location();
        self.view = LibraryView::Search;
        self.focus = Focus::Library;
        self.selected = 0;
        self.close_detail();
        self.pending_add = None;
        self.picker_return = None;
        self.confirm_delete = None;
        self.prompt = None;
        self.search_query.clear();
        self.search_category = SearchCategory::default();
        self.search_editing = true;
        self.search_results = catalog::SearchResults::default();
        self.status = "Search: type to filter ( / to browse results )".to_string();
    }

    fn switch_search_category(&mut self, category: SearchCategory) {
        if self.view != LibraryView::Search {
            return;
        }
        self.search_category = category;
        self.selected = 0;
    }

    /// Toggle between editing the search query and navigating the results.
    fn toggle_search_focus(&mut self) {
        if self.view != LibraryView::Search {
            return;
        }
        self.search_editing = !self.search_editing;
        self.status = if self.search_editing {
            "Search: editing query".to_string()
        } else {
            "Search: press / to edit, Enter/a/N/P/A on results".to_string()
        };
    }

    fn type_char(&mut self, ch: char) {
        if self.show_help {
            self.help_query.push(ch);
            self.help_selected = 0;
        } else if self.view == LibraryView::Search {
            self.search_query.push(ch);
            self.selected = 0;
            self.refresh_search();
        } else if let Some(prompt) = &mut self.prompt {
            prompt.buffer.push(ch);
        }
    }

    fn delete_char(&mut self) {
        if self.show_help {
            self.help_query.pop();
            self.help_selected = 0;
        } else if self.view == LibraryView::Search {
            self.search_query.pop();
            self.selected = 0;
            self.refresh_search();
        } else if let Some(prompt) = &mut self.prompt {
            prompt.buffer.pop();
        }
    }

    fn cancel_text_input(&mut self) {
        if self.show_help {
            if self.help_query.is_empty() {
                self.show_help = false;
            } else {
                self.help_query.clear();
                self.help_selected = 0;
            }
        } else if self.view == LibraryView::Search {
            self.restore_location();
            self.status = "Search closed".to_string();
        } else if self.prompt.is_some() {
            self.prompt = None;
            self.status = "Cancelled".to_string();
        }
    }

    fn refresh_search(&mut self) {
        self.search_results = catalog::search(&self.catalog, &self.playlists, &self.search_query);
    }

    /// Rebuild the in-memory catalog from the database.
    fn reload_catalog(&mut self) {
        self.catalog = match &self.db {
            Some(db) => db
                .list_available_tracks()
                .map(catalog::build_catalog)
                .unwrap_or_default(),
            None => Catalog::default(),
        };
        self.reload_playlists();
    }

    /// Reload playlist names and, if a playlist is open, its tracks.
    fn reload_playlists(&mut self) {
        self.playlists = match &self.db {
            Some(db) => db.list_playlists().unwrap_or_default(),
            None => Vec::new(),
        };
        if let Some(playlist) = self.open_playlist.clone() {
            self.open_tracks = self
                .db
                .as_ref()
                .and_then(|db| db.playlist_tracks(playlist.id).ok())
                .unwrap_or_default();
        }
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
                        Some(db) => {
                            match db.sync_tracks(&tracks) {
                                Ok(()) => self.status = format!("Indexed {count} track(s)"),
                                Err(error) => self.status = format!("Index write failed: {error}"),
                            }
                            if let Err(error) = db.remove_unavailable_from_playlists() {
                                self.status = format!("Playlist cleanup failed: {error}");
                            }
                        }
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
        if let Some(tracks) = self.pending_add.clone() {
            self.commit_add(tracks);
            return;
        }
        if self.detail_open() {
            if let Some(track) = self.open_tracks.get(self.selected).cloned() {
                self.play_now(vec![track]);
            }
            return;
        }
        match self.view {
            LibraryView::Songs => {
                if let Some(track) = self.catalog.songs.get(self.selected).cloned() {
                    self.play_now(vec![track]);
                }
            }
            LibraryView::Albums => {
                if let Some(album) = self.catalog.albums.get(self.selected).cloned() {
                    self.push_location();
                    self.open_album_detail(album);
                }
            }
            LibraryView::Artists => {
                if let Some(artist) = self.catalog.artists.get(self.selected).cloned() {
                    self.push_location();
                    self.open_artist_detail(artist);
                }
            }
            LibraryView::Playlists => {
                if let Some(playlist) = self.playlists.get(self.selected).cloned() {
                    self.push_location();
                    self.open_playlist_detail(playlist);
                }
            }
            LibraryView::Search => self.enter_search(),
        }
    }

    fn enter_search(&mut self) {
        match self.search_category {
            SearchCategory::Songs => {
                if let Some(track) = self.search_results.songs.get(self.selected).cloned() {
                    self.play_now(vec![track]);
                }
            }
            SearchCategory::Albums => {
                if let Some(album) = self.search_results.albums.get(self.selected).cloned() {
                    self.push_location();
                    self.open_album_detail(album);
                }
            }
            SearchCategory::Artists => {
                if let Some(artist) = self.search_results.artists.get(self.selected).cloned() {
                    self.push_location();
                    self.open_artist_detail(artist);
                }
            }
            SearchCategory::Playlists => {
                if let Some(playlist) = self.search_results.playlists.get(self.selected).cloned() {
                    self.push_location();
                    self.open_playlist_detail(playlist);
                }
            }
        }
    }

    /// The tracks represented by the current selection (single song, whole
    /// album/artist/playlist), in deterministic order.
    fn selected_tracks(&self) -> Option<Vec<Track>> {
        if self.picker_active() {
            return None;
        }
        if self.detail_open() {
            return self
                .open_tracks
                .get(self.selected)
                .map(|track| vec![track.clone()]);
        }
        match self.view {
            LibraryView::Songs => self
                .catalog
                .songs
                .get(self.selected)
                .map(|track| vec![track.clone()]),
            LibraryView::Albums => self
                .catalog
                .albums
                .get(self.selected)
                .map(|album| album.tracks.clone()),
            LibraryView::Artists => self
                .catalog
                .artists
                .get(self.selected)
                .map(|artist| artist.tracks.clone()),
            LibraryView::Playlists => self
                .playlists
                .get(self.selected)
                .and_then(|playlist| self.playlist_tracks(playlist.id)),
            LibraryView::Search => self.search_selected_tracks(),
        }
    }

    fn search_selected_tracks(&self) -> Option<Vec<Track>> {
        match self.search_category {
            SearchCategory::Songs => self
                .search_results
                .songs
                .get(self.selected)
                .map(|track| vec![track.clone()]),
            SearchCategory::Albums => self
                .search_results
                .albums
                .get(self.selected)
                .map(|album| album.tracks.clone()),
            SearchCategory::Artists => self
                .search_results
                .artists
                .get(self.selected)
                .map(|artist| artist.tracks.clone()),
            SearchCategory::Playlists => self
                .search_results
                .playlists
                .get(self.selected)
                .and_then(|playlist| self.playlist_tracks(playlist.id)),
        }
    }

    fn playlist_tracks(&self, id: i64) -> Option<Vec<Track>> {
        self.db.as_ref().and_then(|db| db.playlist_tracks(id).ok())
    }

    fn open_playlist_detail(&mut self, playlist: Playlist) {
        self.open_tracks = self
            .db
            .as_ref()
            .and_then(|db| db.playlist_tracks(playlist.id).ok())
            .unwrap_or_default();
        self.open_playlist = Some(playlist);
        self.open_album = None;
        self.open_artist = None;
        self.selected = 0;
    }

    fn open_album_detail(&mut self, album: catalog::Album) {
        self.open_tracks = album.tracks.clone();
        self.open_album = Some(album);
        self.open_artist = None;
        self.open_playlist = None;
        self.selected = 0;
    }

    fn open_artist_detail(&mut self, artist: catalog::Artist) {
        self.open_tracks = artist.tracks.clone();
        self.open_artist = Some(artist);
        self.open_album = None;
        self.open_playlist = None;
        self.selected = 0;
    }

    /// Whether any detail view (playlist/album/artist) is open.
    pub fn detail_open(&self) -> bool {
        self.open_playlist.is_some() || self.open_album.is_some() || self.open_artist.is_some()
    }

    fn close_detail(&mut self) {
        self.open_playlist = None;
        self.open_album = None;
        self.open_artist = None;
        self.open_tracks.clear();
    }

    /// The title of the open detail view, for the UI.
    pub fn detail_label(&self) -> Option<String> {
        self.open_playlist
            .as_ref()
            .map(|playlist| format!("Playlist: {}", playlist.name))
            .or_else(|| {
                self.open_album
                    .as_ref()
                    .map(|album| format!("Album: {}", album.title))
            })
            .or_else(|| {
                self.open_artist
                    .as_ref()
                    .map(|artist| format!("Artist: {}", artist.name))
            })
    }

    fn push_location(&mut self) {
        self.back_stack.push(NavLocation {
            view: self.view,
            selected: self.selected,
            search_query: self.search_query.clone(),
            search_category: self.search_category,
            search_editing: self.search_editing,
        });
    }

    fn restore_location(&mut self) {
        self.close_detail();
        if let Some(location) = self.back_stack.pop() {
            self.view = location.view;
            self.selected = location.selected;
            self.search_query = location.search_query;
            self.search_category = location.search_category;
            self.search_editing = location.search_editing;
            if self.view == LibraryView::Search {
                self.refresh_search();
            }
        } else {
            self.view = LibraryView::Songs;
            self.selected = 0;
        }
        self.selected = self.selected.min(self.library_len().saturating_sub(1));
    }

    fn go_back(&mut self) {
        self.confirm_delete = None;
        if self.detail_open() {
            self.restore_location();
        } else if self.pending_add.is_some() {
            self.pending_add = None;
            if let Some(view) = self.picker_return.take() {
                self.view = view;
            }
            self.selected = 0;
        } else if !self.back_stack.is_empty() {
            self.restore_location();
        }
    }

    /// Append every track currently represented by the panel to the queue.
    ///
    /// Songs/Albums/Artists panels represent the whole library; an open detail
    /// represents its own tracks; Search represents the active category's
    /// matches. Duplicates (by path) are removed.
    fn add_all_to_queue(&mut self) {
        if self.picker_active() {
            self.status = "Bulk add works in a library view".to_string();
            return;
        }

        let tracks: Vec<Track> = if self.detail_open() {
            self.open_tracks.clone()
        } else {
            match self.view {
                LibraryView::Songs | LibraryView::Albums | LibraryView::Artists => {
                    self.catalog.songs.clone()
                }
                LibraryView::Playlists => {
                    self.status = "Open a playlist first (Enter)".to_string();
                    return;
                }
                LibraryView::Search => self.search_all_tracks(),
            }
        };

        let tracks = crate::models::dedupe_by_path(tracks);
        if tracks.is_empty() {
            self.status = "No songs to add".to_string();
            return;
        }
        let count = tracks.len();
        self.queue.append(tracks);
        self.status = format!("Added {count} song(s) to queue");
    }

    /// All tracks matchable by the current search category.
    fn search_all_tracks(&self) -> Vec<Track> {
        match self.search_category {
            SearchCategory::Songs => self.search_results.songs.clone(),
            SearchCategory::Albums => self
                .search_results
                .albums
                .iter()
                .flat_map(|album| album.tracks.clone())
                .collect(),
            SearchCategory::Artists => self
                .search_results
                .artists
                .iter()
                .flat_map(|artist| artist.tracks.clone())
                .collect(),
            SearchCategory::Playlists => self
                .search_results
                .playlists
                .iter()
                .filter_map(|playlist| self.playlist_tracks(playlist.id))
                .flatten()
                .collect(),
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

    /// Insert the selected library entry after the active track (FR-PLAY-04).
    fn play_next(&mut self) {
        let Some(tracks) = self.selected_tracks() else {
            return;
        };
        if tracks.is_empty() {
            self.status = "No playable tracks".to_string();
            return;
        }
        if self.queue.is_empty() {
            self.play_now(tracks);
            return;
        }
        self.queue.insert_after_current(tracks);
        self.status = "Play Next".to_string();
    }

    /// Append the selected library entry to the end of the queue (FR-PLAY-02).
    fn add_to_queue(&mut self) {
        let Some(tracks) = self.selected_tracks() else {
            return;
        };
        if tracks.is_empty() {
            self.status = "No playable tracks".to_string();
            return;
        }
        self.queue.append(tracks);
        self.status = "Added to queue".to_string();
    }

    /// Remove the selected queue entry (FR-PLAY-02).
    fn remove_from_queue(&mut self) {
        if self.focus != Focus::Queue {
            self.status = "Remove works in the Queue view".to_string();
            return;
        }
        if self.queue.is_empty() {
            return;
        }
        let was_active = self.queue_selected == self.queue.current_index();
        self.queue.remove(self.queue_selected);
        self.queue_selected = self.queue_selected.min(self.queue.len().saturating_sub(1));
        if was_active {
            self.stop_playback("Track removed from queue");
        } else {
            self.status = "Removed from queue".to_string();
        }
    }

    fn toggle_shuffle(&mut self) {
        self.queue.set_shuffle(!self.queue.is_shuffled());
        self.status = if self.queue.is_shuffled() {
            "Shuffle: on".to_string()
        } else {
            "Shuffle: off".to_string()
        };
    }

    fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.next();
        self.status = format!("Repeat: {}", self.repeat.label());
    }

    fn toggle_play_mode(&mut self) {
        self.play_mode = !self.play_mode;
        self.status = if self.play_mode {
            "Play mode".to_string()
        } else {
            "Library".to_string()
        };
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.help_query.clear();
        self.help_selected = 0;
    }

    fn cycle_theme(&mut self) {
        let next = match self.theme.name.as_str() {
            "rust-walnut" => theme::catppuccin(),
            "catppuccin" => theme::dracula(),
            "dracula" => theme::nord(),
            _ => theme::rust_walnut(),
        };
        self.status = format!("Theme: {}", next.name);
        self.theme = next;
    }

    fn begin_prompt(&mut self, kind: PromptKind) {
        self.prompt = Some(Prompt {
            kind,
            buffer: String::new(),
        });
        self.status = "Type and press Enter".to_string();
    }

    fn begin_rename(&mut self) {
        if self.open_playlist.is_some() || self.view != LibraryView::Playlists {
            self.status = "Rename works in the Playlists list".to_string();
            return;
        }
        if let Some(playlist) = self.playlists.get(self.selected).cloned() {
            self.begin_prompt(PromptKind::Rename(playlist.id));
        }
    }

    fn begin_export(&mut self) {
        if self.open_playlist.is_some() || self.view != LibraryView::Playlists {
            self.status = "Export works in the Playlists list".to_string();
            return;
        }
        if let Some(playlist) = self.playlists.get(self.selected).cloned() {
            self.begin_prompt(PromptKind::Export(playlist.id));
        }
    }

    fn duplicate_selected(&mut self) {
        if self.open_playlist.is_some() || self.view != LibraryView::Playlists {
            self.status = "Duplicate works in the Playlists list".to_string();
            return;
        }
        let Some(playlist) = self.playlists.get(self.selected).cloned() else {
            return;
        };
        let name = format!("{} (copy)", playlist.name);
        match &mut self.db {
            Some(db) => match db.duplicate_playlist(playlist.id, &name) {
                Ok(_) => {
                    self.reload_playlists();
                    self.status = format!("Duplicated as {name}");
                }
                Err(error) => self.status = format!("Duplicate failed: {error}"),
            },
            None => self.status = "Database unavailable".to_string(),
        }
    }

    fn delete_selected(&mut self) {
        if self.open_playlist.is_some() {
            self.remove_track_from_open_playlist();
            return;
        }
        if self.view != LibraryView::Playlists {
            self.status = "Delete works in the Playlists list".to_string();
            return;
        }
        let Some(playlist) = self.playlists.get(self.selected).cloned() else {
            return;
        };
        if self.confirm_delete == Some(playlist.id) {
            self.confirm_delete = None;
            match &mut self.db {
                Some(db) => {
                    if let Err(error) = db.delete_playlist(playlist.id) {
                        self.status = format!("Delete failed: {error}");
                        return;
                    }
                }
                None => self.status = "Database unavailable".to_string(),
            }
            self.reload_playlists();
            self.status = format!("Deleted playlist {}", playlist.name);
        } else {
            self.confirm_delete = Some(playlist.id);
            self.status = format!("Delete '{}'? Press x again to confirm", playlist.name);
        }
    }

    fn remove_track_from_open_playlist(&mut self) {
        let Some(playlist) = self.open_playlist.clone() else {
            return;
        };
        let Some(track) = self.open_tracks.get(self.selected).cloned() else {
            return;
        };
        let Some(db) = &mut self.db else {
            return;
        };
        if let Some(id) = db.track_id_by_path(&track.path).ok().flatten() {
            if let Err(error) = db.remove_track_from_playlist(playlist.id, id) {
                self.status = format!("Remove failed: {error}");
                return;
            }
        }
        self.reload_playlists();
        self.status = "Removed from playlist".to_string();
    }

    fn begin_add_to_playlist(&mut self) {
        if self.open_playlist.is_some() || self.view == LibraryView::Playlists {
            self.status = "Select a song/album/artist to add".to_string();
            return;
        }
        let Some(tracks) = self.selected_tracks() else {
            return;
        };
        if tracks.is_empty() {
            self.status = "No playable tracks".to_string();
            return;
        }
        if self.playlists.is_empty() {
            self.status = "No playlists; press c to create one".to_string();
            return;
        }
        self.picker_return = Some(self.view);
        self.pending_add = Some(tracks);
        self.view = LibraryView::Playlists;
        self.selected = 0;
        self.status = "Pick a playlist to add to".to_string();
    }

    fn commit_add(&mut self, tracks: Vec<Track>) {
        let Some(playlist) = self.playlists.get(self.selected).cloned() else {
            return;
        };
        let Some(db) = &mut self.db else {
            return;
        };
        let ids: Vec<i64> = tracks
            .iter()
            .filter_map(|track| db.track_id_by_path(&track.path).ok().flatten())
            .collect();
        if ids.is_empty() {
            self.status = "No tracks added".to_string();
        } else {
            match db.add_tracks_to_playlist(playlist.id, &ids) {
                Ok(()) => {
                    self.status = format!("Added {} track(s) to {}", ids.len(), playlist.name)
                }
                Err(error) => self.status = format!("Add failed: {error}"),
            }
        }
        self.pending_add = None;
        if let Some(view) = self.picker_return.take() {
            self.view = view;
        }
        self.selected = 0;
    }

    fn move_track(&mut self, delta: i32) {
        if self.open_playlist.is_none() {
            self.status = "Move works inside a playlist".to_string();
            return;
        }
        let len = self.open_tracks.len();
        if len < 2 {
            return;
        }
        let from = self.selected;
        let to = (from as i32 + delta).clamp(0, len as i32 - 1) as usize;
        if from == to {
            return;
        }
        self.open_tracks.swap(from, to);
        self.selected = to;
        self.persist_open_order();
    }

    fn sort_open_playlist(&mut self) {
        if self.open_playlist.is_none() {
            self.status = "Sort works inside a playlist".to_string();
            return;
        }
        self.open_tracks.sort_by(|a, b| {
            a.artist_label()
                .to_lowercase()
                .cmp(&b.artist_label().to_lowercase())
                .then_with(|| {
                    a.album_label()
                        .to_lowercase()
                        .cmp(&b.album_label().to_lowercase())
                })
                .then_with(|| library::compare_tracks(a, b))
        });
        self.selected = 0;
        self.persist_open_order();
        self.status = "Playlist sorted".to_string();
    }

    fn persist_open_order(&mut self) {
        let Some(playlist) = self.open_playlist.clone() else {
            return;
        };
        let Some(db) = &mut self.db else {
            return;
        };
        let ids: Vec<i64> = self
            .open_tracks
            .iter()
            .filter_map(|track| db.track_id_by_path(&track.path).ok().flatten())
            .collect();
        let _ = db.reorder_playlist(playlist.id, &ids);
    }

    fn commit_prompt(&mut self) {
        let Some(prompt) = self.prompt.take() else {
            return;
        };
        let value = prompt.buffer.trim().to_string();
        match prompt.kind {
            PromptKind::New => {
                if value.is_empty() {
                    self.status = "Empty name".to_string();
                    return;
                }
                match &mut self.db {
                    Some(db) => match db.create_playlist(&value) {
                        Ok(_) => {
                            self.reload_playlists();
                            self.status = format!("Created playlist {value}");
                        }
                        Err(error) => self.status = format!("Create failed: {error}"),
                    },
                    None => self.status = "Database unavailable".to_string(),
                }
            }
            PromptKind::Rename(id) => {
                if value.is_empty() {
                    self.status = "Empty name".to_string();
                    return;
                }
                match &mut self.db {
                    Some(db) => match db.rename_playlist(id, &value) {
                        Ok(_) => {
                            self.reload_playlists();
                            self.status = format!("Renamed to {value}");
                        }
                        Err(error) => self.status = format!("Rename failed: {error}"),
                    },
                    None => self.status = "Database unavailable".to_string(),
                }
            }
            PromptKind::Export(id) => {
                if value.is_empty() {
                    self.status = "Empty path".to_string();
                    return;
                }
                self.export_playlist(id, &value);
            }
            PromptKind::Import => {
                if value.is_empty() {
                    self.status = "Empty path".to_string();
                    return;
                }
                self.import_playlist(&value);
            }
        }
    }

    fn export_playlist(&mut self, id: i64, path: &str) {
        let expanded = library::expand_tilde(path);
        let tracks = match &self.db {
            Some(db) => match db.playlist_tracks(id) {
                Ok(tracks) => tracks,
                Err(error) => {
                    self.status = format!("Export failed: {error}");
                    return;
                }
            },
            None => {
                self.status = "Database unavailable".to_string();
                return;
            }
        };
        match m3u::export_m3u_file(Path::new(&expanded), &tracks) {
            Ok(()) => self.status = format!("Exported {} track(s) to {expanded}", tracks.len()),
            Err(error) => self.status = format!("Export failed: {error}"),
        }
    }

    fn import_playlist(&mut self, path: &str) {
        let expanded = library::expand_tilde(path);
        let outcome = match m3u::import_m3u_file(Path::new(&expanded)) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.status = format!("Import failed: {error}");
                return;
            }
        };

        let name = Path::new(&expanded)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Imported".to_string());

        let Some(db) = &mut self.db else {
            self.status = "Database unavailable".to_string();
            return;
        };
        let mut ids = Vec::new();
        for path in &outcome.imported {
            let track = library::read_track(path);
            if let Ok(id) = db.upsert_track(&track) {
                ids.push(id);
            }
        }
        let playlist_id = match db.create_playlist(&name) {
            Ok(id) => id,
            Err(error) => {
                self.status = format!("Import failed: {error}");
                return;
            }
        };
        let _ = db.add_tracks_to_playlist(playlist_id, &ids);

        self.reload_catalog();
        self.status = format!(
            "Imported {} (skipped {}, missing {})",
            outcome.imported.len(),
            outcome.skipped.len(),
            outcome.unavailable.len()
        );
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

    /// Advance to the next track, honouring Repeat All but not Repeat One.
    fn advance_to_next(&mut self) {
        if self.queue.advance() {
            self.queue_selected = self.queue.current_index();
            self.start_playback();
        } else if self.repeat == RepeatMode::All {
            self.queue.set_current(0);
            self.queue_selected = 0;
            self.start_playback();
        } else {
            self.stop_playback("End of queue");
        }
    }

    /// Handle the natural end of the current track (FR-PLAY-06, FR-PLAY-07).
    fn on_finished(&mut self) {
        if self.repeat == RepeatMode::One {
            self.start_playback();
        } else {
            self.advance_to_next();
        }
    }

    fn next(&mut self) {
        if self.queue.is_empty() {
            self.status = "Queue is empty".to_string();
            return;
        }
        self.advance_to_next();
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

fn prompt_label(kind: &PromptKind) -> &'static str {
    match kind {
        PromptKind::New => "New playlist name",
        PromptKind::Rename(_) => "New name",
        PromptKind::Export(_) => "Export M3U path",
        PromptKind::Import => "Import M3U path",
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
