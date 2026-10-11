//! Rayap Musik library crate.
//!
//! The binary (`src/main.rs`) is a thin wrapper; all logic lives here so it can
//! be exercised by integration tests in `tests/`.

pub mod app;
pub mod config;
pub mod db;
pub mod input;
pub mod library;
pub mod models;
pub mod player;
pub mod ui;
