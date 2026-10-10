//! Application state and decision making.
//!
//! Milestone 1 is a UI shell: it holds focus, volume, and a status line but
//! does not yet scan the library or play audio.

use crate::models::{Action, Focus, PlaybackState};

pub const DEFAULT_VOLUME: u8 = 70;
pub const VOLUME_STEP: u8 = 5;

pub struct App {
    /// Root directory passed on the command line, if any.
    pub root: Option<String>,
    pub focus: Focus,
    pub playback: PlaybackState,
    pub volume: u8,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    pub fn new(root: Option<String>) -> Self {
        let status = match &root {
            Some(path) => format!("Root: {path}"),
            None => "No root directory provided".to_string(),
        };

        Self {
            root,
            focus: Focus::default(),
            playback: PlaybackState::default(),
            volume: DEFAULT_VOLUME,
            should_quit: false,
            status,
        }
    }

    /// Apply an action to the state.
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectUp | Action::SelectDown => {
                // Navigation is wired once the library module exists.
            }
            Action::NextFocus => {
                self.focus = self.focus.next();
                self.status = format!("Focus: {}", self.focus.label());
            }
            Action::Enter | Action::GoUp => {
                // Folder navigation and playback selection land in later milestones.
            }
            Action::TogglePlay => {
                self.playback = match self.playback {
                    PlaybackState::Playing => PlaybackState::Paused,
                    _ => PlaybackState::Playing,
                };
            }
            Action::NextTrack | Action::PreviousTrack => {
                // Playback control lands in later milestones.
            }
            Action::VolumeUp => self.set_volume(self.volume.saturating_add(VOLUME_STEP)),
            Action::VolumeDown => self.set_volume(self.volume.saturating_sub(VOLUME_STEP)),
            Action::Quit => self.should_quit = true,
            Action::Resize => {}
        }
    }

    fn set_volume(&mut self, value: u8) {
        self.volume = value.min(100);
        self.status = format!("Volume: {}%", self.volume);
    }
}
