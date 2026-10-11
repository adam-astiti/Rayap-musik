//! Persistent preferences stored in a human-editable TOML file.
//!
//! The file is deliberately forgiving: unknown keys are kept, comments are
//! preserved, and a single malformed value falls back to its default instead
//! of discarding the whole file.

use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{value, DocumentMut, Item, Table};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Music root directory. `None` until the user has chosen one.
    pub root: Option<String>,
    pub theme: String,
    pub volume: u8,
    pub advanced: Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// missing, unreadable, or malformed.
    pub fn load() -> Config {
        Self::load_from_path(&config_dir().join(CONFIG_FILE))
    }

    pub fn load_from_path(path: &Path) -> Config {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => return Config::default(),
        };
        parse(&text)
    }

    /// Write the configuration, creating parent directories as needed.
    ///
    /// An existing, well-formed file is edited in place so comments and
    /// unknown keys are preserved. A missing or malformed file is replaced by
    /// the commented template.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&config_dir().join(CONFIG_FILE))
    }

    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let text = match fs::read_to_string(path) {
            Ok(existing) => update_in_place(&existing, self).unwrap_or_else(|| self.render()),
            Err(_) => self.render(),
        };

        fs::write(path, text)
    }

    /// Volume clamped to the 0..=100 range, tolerating hand-edited values.
    pub fn volume(&self) -> u8 {
        self.volume.min(100)
    }

    /// Render the configuration as commented TOML so it stays easy to edit.
    ///
    /// Used only for first-time creation (or recovery of a corrupt file).
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

/// Parse a config file, applying per-field defaults for anything missing or
/// malformed. Never fails.
fn parse(text: &str) -> Config {
    let Ok(doc) = text.parse::<DocumentMut>() else {
        return Config::default();
    };

    let mut config = Config::default();

    if let Some(root) = doc.get("root").and_then(Item::as_str) {
        config.root = Some(root.to_string());
    }
    if let Some(theme) = doc.get("theme").and_then(Item::as_str) {
        if !theme.is_empty() {
            config.theme = theme.to_string();
        }
    }
    if let Some(volume) = doc.get("volume").and_then(Item::as_integer) {
        config.volume = volume.clamp(0, 100) as u8;
    }
    if let Some(data_dir) = doc
        .get("advanced")
        .and_then(Item::as_table)
        .and_then(|table| table.get("data_dir"))
        .and_then(Item::as_str)
    {
        if !data_dir.is_empty() {
            config.advanced.data_dir = data_dir.to_string();
        }
    }

    config
}

/// Update the known keys of an existing config file in place, preserving
/// comments and unknown keys. Returns `None` when the file cannot be parsed.
fn update_in_place(existing: &str, config: &Config) -> Option<String> {
    let mut doc: DocumentMut = existing.parse().ok()?;

    if let Some(root) = &config.root {
        doc["root"] = value(root.clone());
    }
    doc["theme"] = value(config.theme.clone());
    doc["volume"] = value(i64::from(config.volume));

    if doc.get("advanced").and_then(Item::as_table).is_none() {
        doc["advanced"] = Item::Table(Table::new());
    }
    if let Some(table) = doc["advanced"].as_table_mut() {
        table["data_dir"] = value(config.advanced.data_dir.clone());
    }

    Some(doc.to_string())
}

/// Escape a value for use inside a TOML basic string.
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{escape, parse, update_in_place, Config, DEFAULT_THEME, DEFAULT_VOLUME};

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

    #[test]
    fn parse_applies_per_field_defaults() {
        let text = r#"
root = "/music"
volume = "loud"
theme = 123
"#;
        let config = parse(text);
        assert_eq!(config.root.as_deref(), Some("/music"));
        assert_eq!(config.theme, DEFAULT_THEME, "malformed theme falls back");
        assert_eq!(config.volume, DEFAULT_VOLUME, "malformed volume falls back");
    }

    #[test]
    fn parse_clamps_out_of_range_volume() {
        let config = parse("volume = 999");
        assert_eq!(config.volume, 100);
        let config = parse("volume = -5");
        assert_eq!(config.volume, 0);
    }

    #[test]
    fn update_preserves_comments_and_unknown_keys() {
        let existing = r#"# my personal note
root = "/old"
volume = 70

[advanced]
data_dir = "/old/data"

[my_custom]
foo = "bar"
"#;
        let config = Config {
            root: Some("/new".to_string()),
            theme: "catppuccin".to_string(),
            volume: 80,
            advanced: super::Advanced {
                data_dir: "/new/data".to_string(),
            },
        };

        let updated = update_in_place(existing, &config).expect("update");

        assert!(updated.contains("# my personal note"));
        assert!(updated.contains("root = \"/new\""));
        assert!(updated.contains("theme = \"catppuccin\""));
        assert!(updated.contains("volume = 80"));
        assert!(updated.contains("data_dir = \"/new/data\""));
        assert!(updated.contains("[my_custom]"));
        assert!(updated.contains("foo = \"bar\""));
    }
}
