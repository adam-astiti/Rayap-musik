//! Translates keyboard events into application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::models::{Action, LibraryView};

/// Map a crossterm key event to an [`Action`].
///
/// When `searching` is true, printable characters feed the search query instead
/// of triggering normal bindings. Returns `None` for unbound keys and for key
/// release/repeat events that should be ignored.
pub fn map_key(key: KeyEvent, searching: bool) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.code == KeyCode::Char('q') {
        return Some(Action::Quit);
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

    if searching {
        return match key.code {
            KeyCode::Up => Some(Action::SelectUp),
            KeyCode::Down => Some(Action::SelectDown),
            KeyCode::Char(ch) => Some(Action::SearchChar(ch)),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Esc => Some(Action::SearchCancel),
            KeyCode::Enter => Some(Action::Enter),
            _ => None,
        };
    }

    let action = match key.code {
        KeyCode::Up => Action::SelectUp,
        KeyCode::Down => Action::SelectDown,
        KeyCode::Tab => Action::NextFocus,
        KeyCode::Enter => Action::Enter,
        KeyCode::Char(' ') => Action::TogglePlay,
        KeyCode::Char('n') => Action::NextTrack,
        KeyCode::Char('p') => Action::PreviousTrack,
        KeyCode::Char('+') | KeyCode::Char('=') => Action::VolumeUp,
        KeyCode::Char('-') => Action::VolumeDown,
        KeyCode::Char('r') => Action::Rescan,
        KeyCode::Char('/') => Action::StartSearch,
        KeyCode::Char('1') => Action::SwitchView(LibraryView::Songs),
        KeyCode::Char('2') => Action::SwitchView(LibraryView::Albums),
        KeyCode::Char('3') => Action::SwitchView(LibraryView::Artists),
        KeyCode::Char('4') => Action::SwitchView(LibraryView::Playlists),
        _ => return None,
    };

    Some(action)
}
