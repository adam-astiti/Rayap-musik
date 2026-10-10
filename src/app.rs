//! Application state and decision making.

use std::path::PathBuf;

use crate::library;
use crate::models::{Action, Focus, LibraryItem, PlaybackState};

pub const DEFAULT_VOLUME: u8 = 70;
pub const VOLUME_STEP: u8 = 5;

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
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let mut app = Self {
            current_dir: root.clone(),
            root,
            items: Vec::new(),
            selected: 0,
            focus: Focus::Library,
            playback: PlaybackState::default(),
            volume: DEFAULT_VOLUME,
            should_quit: false,
            status: String::new(),
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
            Action::TogglePlay => {
                self.playback = match self.playback {
                    PlaybackState::Playing => PlaybackState::Paused,
                    _ => PlaybackState::Playing,
                };
            }
            Action::NextTrack | Action::PreviousTrack => {
                // Playback control lands in a later milestone.
            }
            Action::VolumeUp => self.set_volume(self.volume.saturating_add(VOLUME_STEP)),
            Action::VolumeDown => self.set_volume(self.volume.saturating_sub(VOLUME_STEP)),
            Action::Quit => self.should_quit = true,
            Action::Resize => {}
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
            return;
        }
        match self.selected_item() {
            Some(LibraryItem::Directory { path, .. }) => {
                self.current_dir = path.clone();
                self.reload_directory();
            }
            Some(LibraryItem::Track(track)) => {
                self.status = format!("Selected track: {} (playback not implemented)", track.title);
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

    fn set_volume(&mut self, value: u8) {
        self.volume = value.min(100);
        self.status = format!("Volume: {}%", self.volume);
    }
}
