//! Audio playback via Rodio.
//!
//! The player owns the output stream and a single sink. It never blocks: the
//! UI polls playback state through [`Player`] methods.

use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

/// Errors produced by the player.
#[derive(Debug)]
pub enum PlayerError {
    /// The audio backend could not be initialized.
    AudioInit(String),
    /// The file could not be opened.
    Open { path: PathBuf, message: String },
    /// The file could not be decoded.
    Decode { path: PathBuf, message: String },
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::AudioInit(message) => write!(f, "audio backend error: {message}"),
            PlayerError::Open { path, message } => {
                write!(f, "cannot open {}: {message}", path.display())
            }
            PlayerError::Decode { path, message } => {
                write!(f, "cannot decode {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for PlayerError {}

/// Metadata about the track currently loaded into the sink.
#[derive(Debug, Clone)]
struct Loaded {
    path: PathBuf,
    duration: Option<Duration>,
}

pub struct Player {
    /// Kept alive for the lifetime of the player; dropping it stops audio.
    _stream: OutputStream,
    sink: Sink,
    loaded: Option<Loaded>,
}

impl Player {
    /// Initialize the default audio output stream.
    pub fn new() -> Result<Self, PlayerError> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|error| PlayerError::AudioInit(error.to_string()))?;
        let sink = Sink::connect_new(stream.mixer());
        Ok(Self {
            _stream: stream,
            sink,
            loaded: None,
        })
    }

    /// Stop any current playback and start playing `path`.
    pub fn play_path(&mut self, path: &Path) -> Result<(), PlayerError> {
        let file = File::open(path).map_err(|error| PlayerError::Open {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        let decoder = Decoder::try_from(file).map_err(|error| PlayerError::Decode {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

        let duration = decoder.total_duration();
        self.sink.stop();
        self.sink.append(decoder);
        self.sink.play();
        self.loaded = Some(Loaded {
            path: path.to_path_buf(),
            duration,
        });
        Ok(())
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.loaded = None;
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn set_volume_percent(&self, percent: u8) {
        self.sink.set_volume(f32::from(percent) / 100.0);
    }

    pub fn has_track(&self) -> bool {
        self.loaded.is_some()
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.loaded.as_ref().map(|loaded| loaded.path.as_path())
    }

    pub fn duration(&self) -> Option<Duration> {
        self.loaded.as_ref().and_then(|loaded| loaded.duration)
    }

    /// Current playback position within the loaded track.
    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }

    /// Whether the loaded track has finished playing naturally.
    ///
    /// A paused sink is never considered finished.
    pub fn is_finished(&self) -> bool {
        self.loaded.is_some() && !self.sink.is_paused() && self.sink.empty()
    }
}
