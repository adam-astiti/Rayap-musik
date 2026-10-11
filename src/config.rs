//! Persistent preferences stored in a human-editable TOML file.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Directory name used under the XDG base directories.
pub const APP_DIR: &str = "rayap-musik";
/// File name of the configuration file.
pub const CONFIG_FILE: &str = "config.toml";
/// File name of the SQLite database.
pub const DB_FILE: &str = "library.db";

pub const DEFAULT_THEME: &str = "rust-walnut";
pub const DEFAULT_VOLUME: u8 = 70;

/// The base directory for the configuration file, honoring `$XDG_CONFIG_HOME`.
pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
        .join(APP_DIR)
}

/// The base directory for the database, honoring `$XDG_DATA_HOME`.
pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share"))
        .join(APP_DIR)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Music root directory. `None` until the user has chosen one.
    pub root: Option<String>,
    pub theme: String,
    pub volume: u8,
    #[serde(default)]
    pub advanced: Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advanced {
    /// Directory that holds the SQLite database.
    pub data_dir: String,
}

impl Default for Advanced {
    fn default() -> Self {
        Self {
            data_dir: data_dir().to_string_lossy().into_owned(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: None,
            theme: DEFAULT_THEME.to_string(),
            volume: DEFAULT_VOLUME,
            advanced: Advanced::default(),
        }
    }
}

impl Config {
    /// Load the configuration, falling back to defaults when the file is
    /// missing or unreadable. Never fails: a corrupt config is treated as empty.
    pub fn load() -> Config {
        Self::load_from_path(&config_dir().join(CONFIG_FILE))
    }

    pub fn load_from_path(path: &Path) -> Config {
        match fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    /// Write the configuration, creating parent directories as needed.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&config_dir().join(CONFIG_FILE))
    }

    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.render())
    }

    /// Volume clamped to the 0..=100 range, tolerating hand-edited values.
    pub fn volume(&self) -> u8 {
        self.volume.min(100)
    }

    /// Render the configuration as commented TOML so it stays easy to edit.
    fn render(&self) -> String {
        let root_line = match &self.root {
            Some(root) => format!("root = \"{}\"", escape(root)),
            None => "# root = \"~/Music\"   (uncomment this line and set your music directory)"
                .to_string(),
        };

        format!(
            r#"# =============================================================================
#  Rayap Musik configuration
#
#  This file is read once at startup. Changes take effect the next time the
#  app launches. Lines beginning with '#' are comments and are ignored.
#
#  A '~' at the start of a path is expanded to your home directory ($HOME).
# =============================================================================

# -----------------------------------------------------------------------------
#  BASIC SETTINGS  (safe to edit)
# -----------------------------------------------------------------------------

# The music directory that Rayap Musik scans and browses.
# Example: root = "~/Music"
{root_line}

# Color theme used by the interface.
# Available themes: rust-walnut, catppuccin, dracula, nord
theme = "{theme}"

# Startup volume as a percentage (0-100). Applied when the app launches.
volume = {volume}

# -----------------------------------------------------------------------------
#  ADVANCED SETTINGS
# -----------------------------------------------------------------------------
#  WARNING: do not change the values below unless you know what you are doing.
#  They point at Rayap Musik's internal data. Editing them can make the app
#  unable to find its library index and playlists.

[advanced]
# Directory where the local database (library index + playlists) is stored.
data_dir = "{data_dir}"
"#,
            root_line = root_line,
            theme = self.theme,
            volume = self.volume,
            data_dir = escape(&self.advanced.data_dir),
        )
    }
}

/// Escape a value for use inside a TOML basic string.
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{escape, Config, DEFAULT_THEME, DEFAULT_VOLUME};

    #[test]
    fn escape_handles_backslashes_and_quotes() {
        assert_eq!(escape(r#""#), r#""#);
        assert_eq!(escape(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape(r"a\b"), r"a\\b");
    }

    #[test]
    fn render_contains_guidance_and_values() {
        let config = Config {
            root: Some("~/Music".to_string()),
            theme: "nord".to_string(),
            volume: 55,
            ..Config::default()
        };

        let text = config.render();
        assert!(text.contains("root = \"~/Music\""));
        assert!(text.contains("theme = \"nord\""));
        assert!(text.contains("volume = 55"));
        assert!(text.contains("BASIC SETTINGS"));
        assert!(text.contains("ADVANCED SETTINGS"));
        assert!(text.contains("do not change"));
    }

    #[test]
    fn render_without_root_shows_example_comment() {
        let config = Config::default();
        let text = config.render();
        assert!(text.contains("# root = \"~/Music\""));
        assert!(!text.lines().any(|line| line.starts_with("root =")));
    }

    #[test]
    fn defaults_are_sane() {
        let config = Config::default();
        assert_eq!(config.theme, DEFAULT_THEME);
        assert_eq!(config.volume, DEFAULT_VOLUME);
        assert!(config.root.is_none());
    }
}
