//! Translates keyboard events into application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::models::{Action, LibraryView};

/// Map a crossterm key event to an [`Action`].
///
/// When `text_mode` is true (search or a prompt is active), printable
/// characters feed the text buffer instead of triggering normal bindings.
/// Returns `None` for unbound keys and for key release/repeat events.
pub fn map_key(key: KeyEvent, text_mode: bool) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

    if text_mode {
        return match key.code {
            KeyCode::Up => Some(Action::SelectUp),
            KeyCode::Down => Some(Action::SelectDown),
            KeyCode::Char(ch) => Some(Action::TypeChar(ch)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Esc => Some(Action::Cancel),
            KeyCode::Enter => Some(Action::Enter),
            _ => None,
        };
    }

    let action = match key.code {
        KeyCode::Up => Action::SelectUp,
        KeyCode::Down => Action::SelectDown,
        KeyCode::Tab => Action::NextFocus,
        KeyCode::Enter => Action::Enter,
        KeyCode::Esc | KeyCode::Backspace => Action::Back,
        KeyCode::Char(' ') => Action::TogglePlay,
        KeyCode::Char('n') => Action::NextTrack,
        KeyCode::Char('p') => Action::PreviousTrack,
        KeyCode::Char('+') | KeyCode::Char('=') => Action::VolumeUp,
        KeyCode::Char('-') => Action::VolumeDown,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('r') => Action::Rescan,
        KeyCode::Char('/') => Action::StartSearch,
        KeyCode::Char('1') => Action::SwitchView(LibraryView::Songs),
        KeyCode::Char('2') => Action::SwitchView(LibraryView::Albums),
        KeyCode::Char('3') => Action::SwitchView(LibraryView::Artists),
        KeyCode::Char('4') => Action::SwitchView(LibraryView::Playlists),
        KeyCode::Char('a') => Action::AddToQueue,
        KeyCode::Char('N') => Action::PlayNext,
        KeyCode::Char('d') => Action::RemoveFromQueue,
        KeyCode::Char('s') => Action::ToggleShuffle,
        KeyCode::Char('R') => Action::CycleRepeat,
        KeyCode::Char('m') => Action::TogglePlayMode,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('T') => Action::CycleTheme,
        KeyCode::Char('c') => Action::NewPlaylist,
        KeyCode::Char('e') => Action::Rename,
        KeyCode::Char('D') => Action::Duplicate,
        KeyCode::Char('x') => Action::Delete,
        KeyCode::Char('P') => Action::AddToPlaylist,
        KeyCode::Char('I') => Action::Import,
        KeyCode::Char('E') => Action::Export,
        KeyCode::Char('J') => Action::MoveDown,
        KeyCode::Char('K') => Action::MoveUp,
        KeyCode::Char('S') => Action::Sort,
        _ => return None,
    };

    Some(action)
}

#[cfg(test)]
mod tests {
    use super::map_key;
    use crate::models::{Action, LibraryView};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn q_quits_in_normal_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event, false), Some(Action::Quit));
    }

    #[test]
    fn q_types_in_text_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event, true), Some(Action::TypeChar('q')));
    }

    #[test]
    fn view_shortcuts_map_to_views() {
        let event = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, false),
            Some(Action::SwitchView(LibraryView::Albums))
        );
    }
}
