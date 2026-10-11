//! Background library indexer.
//!
//! A worker thread performs full scans (filesystem traversal + metadata), and a
//! filesystem watcher triggers debounced rescans when the library changes. The
//! slow metadata work never blocks the UI: results arrive through a channel that
//! the app drains once per frame.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use crate::library;
use crate::models::Track;

/// Quiet period after the last filesystem event before a rescan is triggered.
const DEBOUNCE: Duration = Duration::from_millis(750);
/// How often the watcher loop checks for a shutdown signal while idle.
const WATCH_POLL: Duration = Duration::from_millis(250);

/// Events delivered from the indexer to the app.
pub enum IndexEvent {
    ScanStarted,
    ScanCompleted { tracks: Vec<Track> },
    WatcherError { error: String },
}

enum IndexCommand {
    Rescan,
}

pub struct Indexer {
    command_tx: Sender<IndexCommand>,
    event_rx: Receiver<IndexEvent>,
    watcher_stop_tx: Sender<()>,
}

impl Indexer {
    pub fn new(root: PathBuf) -> Indexer {
        let (event_tx, event_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();
        let (watcher_stop_tx, watcher_stop_rx) = mpsc::channel();

        let worker_root = root.clone();
        let worker_event_tx = event_tx.clone();
        thread::spawn(move || scan_worker(&worker_root, command_rx, worker_event_tx));

        let watcher_command_tx = command_tx.clone();
        thread::spawn(move || watcher_loop(&root, watcher_command_tx, watcher_stop_rx, event_tx));

        Indexer {
            command_tx,
            event_rx,
            watcher_stop_tx,
        }
    }

    /// Drain pending events without blocking.
    pub fn poll(&self) -> Option<IndexEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Ask the worker to rescan the library.
    pub fn rescan(&self) {
        let _ = self.command_tx.send(IndexCommand::Rescan);
    }
}

impl Drop for Indexer {
    fn drop(&mut self) {
        let _ = self.watcher_stop_tx.send(());
    }
}

fn scan_worker(root: &Path, command_rx: Receiver<IndexCommand>, event_tx: Sender<IndexEvent>) {
    scan_once(root, &event_tx);
    while command_rx.recv().is_ok() {
        scan_once(root, &event_tx);
    }
}

fn scan_once(root: &Path, event_tx: &Sender<IndexEvent>) {
    let _ = event_tx.send(IndexEvent::ScanStarted);
    let tracks = library::scan_tracks(root);
    let _ = event_tx.send(IndexEvent::ScanCompleted { tracks });
}

fn watcher_loop(
    root: &Path,
    command_tx: Sender<IndexCommand>,
    stop_rx: Receiver<()>,
    event_tx: Sender<IndexEvent>,
) {
    let (signal_tx, signal_rx) = mpsc::channel::<()>();

    let mut watcher =
        match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            if result.is_ok() {
                let _ = signal_tx.send(());
            }
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                let _ = event_tx.send(IndexEvent::WatcherError {
                    error: error.to_string(),
                });
                return;
            }
        };

    if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
        let _ = event_tx.send(IndexEvent::WatcherError {
            error: error.to_string(),
        });
        return;
    }

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }
        match signal_rx.recv_timeout(WATCH_POLL) {
            Ok(()) => {
                // Coalesce the burst of events into a single rescan.
                while signal_rx.recv_timeout(DEBOUNCE).is_ok() {}
                if command_tx.send(IndexCommand::Rescan).is_err() {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(watcher);
}
