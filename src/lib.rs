//! Rayap Musik library crate.
//!
//! The binary (`src/main.rs`) is a thin wrapper; all logic lives here so it can
//! be exercised by integration tests in `tests/`.

pub mod app;
pub mod catalog;
pub mod config;
pub mod db;
pub mod indexer;
pub mod input;
pub mod keys;
pub mod library;
pub mod m3u;
pub mod models;
pub mod player;
pub mod theme;
pub mod ui;
