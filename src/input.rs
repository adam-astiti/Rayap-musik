//! Translates keyboard events into application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::keys;
use crate::models::{Action, InputMode, SearchCategory};

/// Map a crossterm key event to an [`Action`], honouring the active input mode.
///
/// Normal mode delegates to the centralized binding table in [`keys`]. Search
/// mode captures printable characters but routes `1`-`4` to category switches.
/// Prompt and Help modes capture printable characters and only allow a small
/// set of editing/navigation keys.
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
        InputMode::SearchEdit => search_edit_keys(key.code),
        InputMode::SearchResults => search_results_keys(key.code),
        InputMode::Prompt => prompt_keys(key.code),
        InputMode::Help => help_keys(key.code),
    }
}

fn search_edit_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up => Some(Action::SelectUp),
        KeyCode::Down => Some(Action::SelectDown),
        KeyCode::Char('1') => Some(Action::SwitchSearchCategory(SearchCategory::Songs)),
        KeyCode::Char('2') => Some(Action::SwitchSearchCategory(SearchCategory::Albums)),
        KeyCode::Char('3') => Some(Action::SwitchSearchCategory(SearchCategory::Artists)),
        KeyCode::Char('4') => Some(Action::SwitchSearchCategory(SearchCategory::Playlists)),
        KeyCode::Char('/') => Some(Action::ToggleSearchFocus),
        KeyCode::Char(ch) => Some(Action::TypeChar(ch)),
        KeyCode::Backspace => Some(Action::DeleteChar),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::ToggleSearchFocus),
        _ => None,
    }
}

fn search_results_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up => Some(Action::SelectUp),
        KeyCode::Down => Some(Action::SelectDown),
        KeyCode::Char('1') => Some(Action::SwitchSearchCategory(SearchCategory::Songs)),
        KeyCode::Char('2') => Some(Action::SwitchSearchCategory(SearchCategory::Albums)),
        KeyCode::Char('3') => Some(Action::SwitchSearchCategory(SearchCategory::Artists)),
        KeyCode::Char('4') => Some(Action::SwitchSearchCategory(SearchCategory::Playlists)),
        KeyCode::Char('/') => Some(Action::ToggleSearchFocus),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::Enter),
        _ => keys::lookup_code(&code),
    }
}

fn prompt_keys(code: KeyCode) -> Option<Action> {
    match code {
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
    use crate::models::{Action, InputMode, LibraryView, SearchCategory};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn q_quits_in_normal_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event, InputMode::Normal), Some(Action::Quit));
    }

    #[test]
    fn q_types_in_search_mode() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::SearchEdit),
            Some(Action::TypeChar('q'))
        );
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
        assert_eq!(map_key(event, InputMode::SearchEdit), Some(Action::Quit));
    }

    #[test]
    fn help_mode_feeds_query_not_shortcuts() {
        let event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(map_key(event, InputMode::Help), Some(Action::TypeChar('n')));
    }

    #[test]
    fn search_edit_routes_digits_to_categories() {
        let event = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::SearchEdit),
            Some(Action::SwitchSearchCategory(SearchCategory::Artists))
        );
    }

    #[test]
    fn slash_toggles_focus_in_search() {
        let event = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::SearchEdit),
            Some(Action::ToggleSearchFocus)
        );
        assert_eq!(
            map_key(event, InputMode::SearchResults),
            Some(Action::ToggleSearchFocus)
        );
    }

    #[test]
    fn search_results_allow_actions() {
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::SearchResults),
            Some(Action::AddToQueue)
        );
        let event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);
        assert_eq!(
            map_key(event, InputMode::SearchResults),
            Some(Action::AddAllToQueue)
        );
    }
}
