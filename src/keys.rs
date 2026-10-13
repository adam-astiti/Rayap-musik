//! Centralized keybinding definitions.
//!
//! This is the single source of truth for keybindings. It powers keyboard
//! dispatch, the contextual keybinding helper, the full Help page, and the
//! searchable Help results, so the four representations cannot drift apart.

use crossterm::event::{KeyCode, KeyEvent};

use crate::models::{Action, LibraryView};

/// The interface or interaction context a keybinding applies to.
///
/// Bindings with an empty `contexts` list are global and apply everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    Songs,
    Albums,
    Artists,
    Playlists,
    OpenPlaylist,
    Queue,
    Search,
    PlayMode,
    Help,
    TextInput,
}

/// A single keybinding: its human-readable label, description, the action it
/// triggers, the contexts where it is relevant, extra search terms, and the
/// key codes that produce it.
pub struct Binding {
    pub label: &'static str,
    pub desc: &'static str,
    pub action: Action,
    /// Empty means the binding is global (relevant in every context).
    pub contexts: &'static [Context],
    /// Extra search terms matched by the searchable Help page.
    pub terms: &'static [&'static str],
    pub keys: &'static [KeyCode],
}

const GLOBAL: &[Context] = &[];

use Context as C;

#[rustfmt::skip]
pub static BINDINGS: &[Binding] = &[
    Binding { label: "Up",     desc: "Move selection up",          action: Action::SelectUp,       contexts: GLOBAL, terms: &["navigate", "previous"], keys: &[KeyCode::Up] },
    Binding { label: "Down",   desc: "Move selection down",        action: Action::SelectDown,     contexts: GLOBAL, terms: &["navigate", "next"],     keys: &[KeyCode::Down] },
    Binding { label: "Tab",    desc: "Switch focus",               action: Action::NextFocus,      contexts: GLOBAL, terms: &["focus", "panel"],       keys: &[KeyCode::Tab] },
    Binding { label: "Enter",  desc: "Play Now / open",            action: Action::Enter,          contexts: GLOBAL, terms: &["play", "open", "select"], keys: &[KeyCode::Enter] },
    Binding { label: "Esc",    desc: "Back",                       action: Action::Back,           contexts: GLOBAL, terms: &["back", "cancel"],       keys: &[KeyCode::Esc, KeyCode::Backspace] },
    Binding { label: "Space",  desc: "Play / Pause",               action: Action::TogglePlay,     contexts: GLOBAL, terms: &["play", "pause"],         keys: &[KeyCode::Char(' ')] },
    Binding { label: "n",      desc: "Next track",                 action: Action::NextTrack,      contexts: GLOBAL, terms: &["next", "skip"],          keys: &[KeyCode::Char('n')] },
    Binding { label: "p",      desc: "Previous / restart",         action: Action::PreviousTrack,  contexts: GLOBAL, terms: &["previous", "restart"],   keys: &[KeyCode::Char('p')] },
    Binding { label: "+",      desc: "Volume up",                  action: Action::VolumeUp,       contexts: GLOBAL, terms: &["volume", "louder"],      keys: &[KeyCode::Char('+'), KeyCode::Char('=')] },
    Binding { label: "-",      desc: "Volume down",                action: Action::VolumeDown,     contexts: GLOBAL, terms: &["volume", "quieter"],     keys: &[KeyCode::Char('-')] },
    Binding { label: "q",      desc: "Quit",                       action: Action::Quit,           contexts: GLOBAL, terms: &["quit", "exit"],          keys: &[KeyCode::Char('q')] },
    Binding { label: "r",      desc: "Rescan library",             action: Action::Rescan,         contexts: GLOBAL, terms: &["rescan", "scan", "index"], keys: &[KeyCode::Char('r')] },
    Binding { label: "/",      desc: "Search",                     action: Action::StartSearch,    contexts: GLOBAL, terms: &["search", "filter"],      keys: &[KeyCode::Char('/')] },
    Binding { label: "m",      desc: "Play Mode",                  action: Action::TogglePlayMode, contexts: GLOBAL, terms: &["playmode", "now playing"], keys: &[KeyCode::Char('m')] },
    Binding { label: "?",      desc: "Help",                       action: Action::ToggleHelp,     contexts: GLOBAL, terms: &["help"],                  keys: &[KeyCode::Char('?')] },
    Binding { label: "T",      desc: "Cycle theme",                action: Action::CycleTheme,     contexts: GLOBAL, terms: &["theme", "color"],        keys: &[KeyCode::Char('T')] },
    Binding { label: "1",      desc: "Songs view",                 action: Action::SwitchView(LibraryView::Songs),     contexts: GLOBAL, terms: &["songs"],     keys: &[KeyCode::Char('1')] },
    Binding { label: "2",      desc: "Albums view",                action: Action::SwitchView(LibraryView::Albums),    contexts: GLOBAL, terms: &["albums"],    keys: &[KeyCode::Char('2')] },
    Binding { label: "3",      desc: "Artists view",               action: Action::SwitchView(LibraryView::Artists),   contexts: GLOBAL, terms: &["artists"],   keys: &[KeyCode::Char('3')] },
    Binding { label: "4",      desc: "Playlists view",             action: Action::SwitchView(LibraryView::Playlists), contexts: GLOBAL, terms: &["playlists"], keys: &[KeyCode::Char('4')] },
    Binding { label: "a",      desc: "Add to queue",               action: Action::AddToQueue,     contexts: &[C::Songs, C::Albums, C::Artists, C::Playlists, C::OpenPlaylist, C::Search], terms: &["queue", "add"],   keys: &[KeyCode::Char('a')] },
    Binding { label: "A",      desc: "Add all to queue",           action: Action::AddAllToQueue,  contexts: &[C::Songs, C::Albums, C::Artists, C::OpenPlaylist, C::Search], terms: &["queue", "add", "all", "bulk"], keys: &[KeyCode::Char('A')] },
    Binding { label: "N",      desc: "Play Next",                  action: Action::PlayNext,       contexts: &[C::Songs, C::Albums, C::Artists, C::Playlists, C::OpenPlaylist, C::Search], terms: &["next", "insert"], keys: &[KeyCode::Char('N')] },
    Binding { label: "d",      desc: "Remove queue entry",         action: Action::RemoveFromQueue, contexts: &[C::Queue],            terms: &["remove", "delete", "queue"], keys: &[KeyCode::Char('d')] },
    Binding { label: "s",      desc: "Toggle shuffle",             action: Action::ToggleShuffle,  contexts: &[C::Queue],            terms: &["shuffle"],                 keys: &[KeyCode::Char('s')] },
    Binding { label: "R",      desc: "Cycle repeat",               action: Action::CycleRepeat,    contexts: &[C::Queue],            terms: &["repeat"],                  keys: &[KeyCode::Char('R')] },
    Binding { label: "c",      desc: "Create playlist",            action: Action::NewPlaylist,    contexts: &[C::Playlists],        terms: &["create", "new"],           keys: &[KeyCode::Char('c')] },
    Binding { label: "e",      desc: "Rename playlist",            action: Action::Rename,         contexts: &[C::Playlists],        terms: &["rename"],                  keys: &[KeyCode::Char('e')] },
    Binding { label: "D",      desc: "Duplicate playlist",         action: Action::Duplicate,      contexts: &[C::Playlists],        terms: &["duplicate", "copy"],       keys: &[KeyCode::Char('D')] },
    Binding { label: "x",      desc: "Delete / remove track",      action: Action::Delete,         contexts: &[C::Playlists, C::OpenPlaylist], terms: &["delete", "remove"], keys: &[KeyCode::Char('x')] },
    Binding { label: "P",      desc: "Add to playlist",            action: Action::AddToPlaylist,  contexts: &[C::Songs, C::Albums, C::Artists, C::Search], terms: &["playlist", "add"], keys: &[KeyCode::Char('P')] },
    Binding { label: "I",      desc: "Import M3U",                 action: Action::Import,         contexts: &[C::Playlists],        terms: &["import", "m3u"],           keys: &[KeyCode::Char('I')] },
    Binding { label: "E",      desc: "Export M3U",                 action: Action::Export,         contexts: &[C::Playlists],        terms: &["export", "m3u"],           keys: &[KeyCode::Char('E')] },
    Binding { label: "J",      desc: "Move track down",            action: Action::MoveDown,       contexts: &[C::OpenPlaylist],     terms: &["move", "reorder"],         keys: &[KeyCode::Char('J')] },
    Binding { label: "K",      desc: "Move track up",              action: Action::MoveUp,         contexts: &[C::OpenPlaylist],     terms: &["move", "reorder"],         keys: &[KeyCode::Char('K')] },
    Binding { label: "S",      desc: "Sort playlist",              action: Action::Sort,           contexts: &[C::OpenPlaylist],     terms: &["sort"],                    keys: &[KeyCode::Char('S')] },
];

/// Resolve a key event to the action it triggers. Context is intentionally
/// ignored here: dispatch keeps historical global behaviour, while `contexts`
/// drives which bindings the contextual helper and Help page surface.
pub fn lookup(key: &KeyEvent) -> Option<Action> {
    lookup_code(&key.code)
}

/// Resolve a key code to the action it triggers.
pub fn lookup_code(code: &KeyCode) -> Option<Action> {
    BINDINGS
        .iter()
        .find(|binding| binding.keys.contains(code))
        .map(|binding| binding.action)
}

/// Bindings relevant to `ctx`, context-specific ones first, then global ones.
pub fn for_context(ctx: Context) -> Vec<&'static Binding> {
    let mut specific = Vec::new();
    let mut global = Vec::new();
    for binding in BINDINGS {
        if binding.contexts.is_empty() {
            global.push(binding);
        } else if binding.contexts.contains(&ctx) {
            specific.push(binding);
        }
    }
    specific.extend(global);
    specific
}

/// All bindings, filtered by a case-insensitive query over label, description,
/// and search terms. An empty query returns the full list.
pub fn search(query: &str) -> Vec<&'static Binding> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return BINDINGS.iter().collect();
    }
    BINDINGS
        .iter()
        .filter(|binding| matches(binding, &query))
        .collect()
}

fn matches(binding: &Binding, query: &str) -> bool {
    binding.label.to_lowercase().contains(query)
        || binding.desc.to_lowercase().contains(query)
        || binding
            .terms
            .iter()
            .any(|term| term.to_lowercase().contains(query))
}

#[cfg(test)]
mod tests {
    use super::{lookup, search, Context, BINDINGS};
    use crate::models::{Action, LibraryView};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn lookup_resolves_known_keys() {
        assert_eq!(lookup(&event(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(
            lookup(&event(KeyCode::Char('2'))),
            Some(Action::SwitchView(LibraryView::Albums))
        );
        assert_eq!(lookup(&event(KeyCode::Up)), Some(Action::SelectUp));
    }

    #[test]
    fn lookup_returns_none_for_unbound_keys() {
        assert_eq!(lookup(&event(KeyCode::Char('z'))), None);
        assert_eq!(lookup(&event(KeyCode::F(12))), None);
    }

    #[test]
    fn shared_key_codes_share_one_binding() {
        let volume = BINDINGS
            .iter()
            .find(|binding| binding.action == Action::VolumeUp)
            .unwrap();
        assert!(volume.keys.contains(&KeyCode::Char('+')));
        assert!(volume.keys.contains(&KeyCode::Char('=')));
    }

    #[test]
    fn for_context_surfaces_queue_actions_in_queue() {
        let actions: Vec<Action> = super::for_context(Context::Queue)
            .into_iter()
            .map(|binding| binding.action)
            .collect();
        assert!(actions.contains(&Action::RemoveFromQueue));
        assert!(actions.contains(&Action::ToggleShuffle));
        assert!(actions.contains(&Action::CycleRepeat));
    }

    #[test]
    fn search_matches_description_and_terms() {
        assert!(!search("volume").is_empty());
        assert!(!search("shuffle").is_empty());
        assert!(search("zzzznope").is_empty());
    }

    #[test]
    fn empty_search_returns_everything() {
        assert_eq!(search("").len(), BINDINGS.len());
    }

    #[test]
    fn no_duplicate_key_codes() {
        let mut seen = std::collections::HashSet::new();
        for binding in BINDINGS {
            for key in binding.keys {
                assert!(seen.insert(*key), "duplicate key binding: {key:?}");
            }
        }
    }
}
