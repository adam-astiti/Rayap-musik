//! Integration tests for configuration persistence.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rayap_musik::config::Config;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = format!(
            "rayap-musik-config-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn save_and_load_round_trips() {
    let temp = TempDir::new();
    let path = temp.path.join("config.toml");

    let config = Config {
        root: Some("~/Music".to_string()),
        theme: "catppuccin".to_string(),
        volume: 42,
        advanced: rayap_musik::config::Advanced {
            data_dir: "/custom/data".to_string(),
        },
    };

    config.save_to_path(&path).expect("save config");
    let loaded = Config::load_from_path(&path);

    assert_eq!(loaded, config);
}

#[test]
fn load_missing_file_returns_defaults() {
    let temp = TempDir::new();
    let loaded = Config::load_from_path(&temp.path.join("nope.toml"));

    assert!(loaded.root.is_none());
    assert_eq!(loaded.theme, rayap_musik::config::DEFAULT_THEME);
    assert_eq!(loaded.volume, rayap_musik::config::DEFAULT_VOLUME);
}

#[test]
fn corrupt_file_falls_back_to_defaults() {
    let temp = TempDir::new();
    let path = temp.path.join("config.toml");
    fs::write(&path, "this is not [valid toml").expect("write corrupt config");

    let loaded = Config::load_from_path(&path);
    assert!(loaded.root.is_none());
}

#[test]
fn written_file_is_human_readable() {
    let temp = TempDir::new();
    let path = temp.path.join("config.toml");

    Config::default().save_to_path(&path).expect("save default");

    let text = fs::read_to_string(&path).expect("read config");
    assert!(text.contains("Rayap Musik configuration"));
    assert!(text.contains("safe to edit"));
    assert!(text.contains("do not change"));
    assert!(text.contains("[advanced]"));
}
