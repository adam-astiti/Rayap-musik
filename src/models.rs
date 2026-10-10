//! Shared data types used across modules.

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
