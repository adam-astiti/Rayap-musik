//! Translates keyboard events into application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::models::Action;

/// Map a crossterm key event to an [`Action`].
///
/// Returns `None` for keys that are not bound or for key release/repeat
/// events that should be ignored.
pub fn map_key(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    let action = match key.code {
        KeyCode::Up => Action::SelectUp,
        KeyCode::Down => Action::SelectDown,
        KeyCode::Tab => Action::NextFocus,
        KeyCode::Enter => Action::Enter,
        KeyCode::Backspace => Action::GoUp,
        KeyCode::Char(' ') => Action::TogglePlay,
        KeyCode::Char('n') => Action::NextTrack,
        KeyCode::Char('p') => Action::PreviousTrack,
        KeyCode::Char('+') | KeyCode::Char('=') => Action::VolumeUp,
        KeyCode::Char('-') => Action::VolumeDown,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            Action::Quit
        }
        _ => return None,
    };

    Some(action)
}
