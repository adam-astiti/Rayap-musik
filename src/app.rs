//! Application state and decision making.

use std::path::PathBuf;
use std::time::Duration;

use crate::library;
use crate::models::{Action, Focus, LibraryItem, PlaybackState, Track};
use crate::player::Player;

pub const DEFAULT_VOLUME: u8 = 70;
pub const VOLUME_STEP: u8 = 5;
/// Threshold used by the previous/restart rule (FR-07).
const RESTART_THRESHOLD: Duration = Duration::from_secs(3);

pub struct App {
    /// Validated root directory; parent navigation never goes above it.
    pub root: PathBuf,
    /// Directory currently displayed in the browser.
    pub current_dir: PathBuf,
    pub items: Vec<LibraryItem>,
    pub selected: usize,
    pub focus: Focus,
    pub playback: PlaybackState,
    pub volume: u8,
    pub should_quit: bool,
    pub status: String,
    /// Track currently loaded into the player, if any.
    pub now_playing: Option<Track>,
    player: Option<Player>,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let (player, status) = match Player::new() {
            Ok(player) => {
                player.set_volume_percent(DEFAULT_VOLUME);
                (Some(player), String::new())
            }
            Err(error) => (None, error.to_string()),
        };

        let mut app = Self {
            current_dir: root.clone(),
            root,
            items: Vec::new(),
            selected: 0,
            focus: Focus::Library,
            playback: PlaybackState::default(),
            volume: DEFAULT_VOLUME,
            should_quit: false,
            status,
            now_playing: None,
            player,
        };
        app.reload_directory();
        app
    }

    /// Re-read the current directory, resetting selection.
    pub fn reload_directory(&mut self) {
        match library::list_directory(&self.current_dir) {
            Ok(items) => {
                self.items = items;
                self.selected = 0;
                self.status = if self.items.is_empty() {
                    format!("Empty: no audio files in {}", self.current_dir.display())
                } else {
                    format!(
                        "{} item(s) in {}",
                        self.items.len(),
                        self.current_dir.display()
                    )
                };
            }
            Err(error) => {
                self.items.clear();
                self.selected = 0;
                self.status = error.to_string();
            }
        }
    }

    pub fn selected_item(&self) -> Option<&LibraryItem> {
        self.items.get(self.selected)
    }

    pub fn audio_available(&self) -> bool {
        self.player.is_some()
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
            Action::GoUp => self.go_to_parent(),
            Action::TogglePlay => self.toggle_play(),
            Action::PreviousTrack => self.previous(),
            Action::NextTrack => {
                self.status = "Next track needs a queue (Milestone 4)".to_string();
            }
            Action::VolumeUp => self.set_volume(self.volume.saturating_add(VOLUME_STEP)),
            Action::VolumeDown => self.set_volume(self.volume.saturating_sub(VOLUME_STEP)),
            Action::Quit => self.should_quit = true,
            Action::Resize => {}
        }
    }

    /// Poll the player for state changes. Called once per UI frame.
    pub fn tick(&mut self) {
        if self.now_playing.is_none() {
            return;
        }
        let (finished, paused) = match &self.player {
            Some(player) => (player.is_finished(), player.is_paused()),
            None => return,
        };

        if finished {
            if self.playback != PlaybackState::Stopped {
                self.playback = PlaybackState::Stopped;
                let title = self
                    .now_playing
                    .as_ref()
                    .map(|track| track.title.clone())
                    .unwrap_or_default();
                self.status = format!("Finished: {title}");
            }
        } else if paused {
            self.playback = PlaybackState::Paused;
        } else {
            self.playback = PlaybackState::Playing;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.focus == Focus::Library && !self.items.is_empty() {
            let last = self.items.len() - 1;
            let next = (self.selected as i32 + delta).clamp(0, last as i32);
            self.selected = next as usize;
        }
    }

    fn enter(&mut self) {
        if self.focus != Focus::Library {
            // Queue focus acts once the queue exists (Milestone 4).
            return;
        }
        match self.selected_item() {
            Some(LibraryItem::Directory { path, .. }) => {
                self.current_dir = path.clone();
                self.reload_directory();
            }
            Some(LibraryItem::Track(track)) => {
                let track = track.clone();
                self.play_track(&track);
            }
            None => {}
        }
    }

    fn go_to_parent(&mut self) {
        if self.focus != Focus::Library {
            return;
        }
        if self.current_dir == self.root {
            self.status = "Already at root".to_string();
            return;
        }
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.reload_directory();
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
            // Start (or restart) the selected track.
            if let Some(LibraryItem::Track(track)) = self.selected_item().cloned() {
                self.play_track(&track);
            } else if let Some(track) = self.now_playing.clone() {
                self.play_track(&track);
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

    /// Previous/restart behaviour (FR-07).
    ///
    /// With a single track there is no previous item, so both branches restart
    /// the current track.
    fn previous(&mut self) {
        let Some(track) = self.now_playing.clone() else {
            self.status = "Nothing playing".to_string();
            return;
        };

        let position = self
            .player
            .as_ref()
            .map(Player::position)
            .unwrap_or_default();

        if position > RESTART_THRESHOLD {
            self.play_track(&track);
            self.status = format!("Restarted: {}", track.title);
        } else {
            self.play_track(&track);
            self.status = format!("Restarted: {} (no previous track)", track.title);
        }
    }

    fn play_track(&mut self, track: &Track) {
        let result = self
            .player
            .as_mut()
            .map(|player| player.play_path(&track.path));

        match result {
            None => {
                self.status = "Audio backend unavailable".to_string();
            }
            Some(Ok(())) => {
                if let Some(player) = &self.player {
                    player.set_volume_percent(self.volume);
                }
                self.now_playing = Some(track.clone());
                self.playback = PlaybackState::Playing;
                self.status = format!("Playing: {}", track.title);
            }
            Some(Err(error)) => {
                self.now_playing = None;
                self.playback = PlaybackState::Stopped;
                self.status = error.to_string();
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
