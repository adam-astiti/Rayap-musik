//! Translates keyboard events into application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::keys;
use crate::models::{Action, InputMode};

/// Map a crossterm key event to an [`Action`], honouring the active input mode.
///
/// Normal mode delegates to the centralized binding table in [`keys`]. Text
/// mode (search query or prompt) and Help mode capture printable characters
/// and only allow a small set of editing/navigation keys.
///
/// Returns `None` for unbound keys and for key release/repeat events.
pub fn map_key(key: KeyEvent, mode: InputMode) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

    match mode {
        InputMode::Normal => keys::lookup(&key),
        InputMode::Text => text_keys(key.code),
        InputMode::Help => help_keys(key.code),
    }
}

fn text_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up => Some(Action::SelectUp),
        KeyCode::Down => Some(Action::SelectDown),
        KeyCode::Char(ch) => Some(Action::TypeChar(ch)),
        KeyCode::Backspace => Some(Action::DeleteChar),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::Enter),
        _ => None,
    }
}

fn help_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up => Some(Action::SelectUp),
        KeyCode::Down => Some(Action::SelectDown),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Backspace => Some(Action::DeleteChar),
        KeyCode::Char(ch) => Some(Action::TypeChar(ch)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::map_key;
    use crate::models::{Action, InputMode, LibraryView};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn q_quits_in_normal_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event, InputMode::Normal), Some(Action::Quit));
    }

    #[test]
    fn q_types_in_text_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event, InputMode::Text), Some(Action::TypeChar('q')));
    }

    #[test]
    fn view_shortcuts_map_to_views() {
        let event = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::Normal),
            Some(Action::SwitchView(LibraryView::Albums))
        );
    }

    #[test]
    fn ctrl_c_quits_in_any_mode() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(event, InputMode::Text), Some(Action::Quit));
    }

    #[test]
    fn help_mode_feeds_query_not_shortcuts() {
        let event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(map_key(event, InputMode::Help), Some(Action::TypeChar('n')));
    }
}
