# Rayap Musik 2.2 — Development PRD
## Milestone 1 & 2: Navigation Foundation and Global Search

**Status:** Ready for implementation planning  
**Product:** Rayap Musik  
**Platform:** Local-first Rust TUI music player

---

## 1. Product Principles

Development must follow these principles:

1. **Minimal changes:** Preserve existing navigation, keybindings, interfaces, and playback behavior wherever possible.
2. **Consistency:** The same interaction should behave consistently across equivalent interfaces.
3. **Single source of truth:** Keybinding handlers, contextual helper, and Help documentation must share one keybinding definition.
4. **Local-first:** Search and navigation operate on the local music library without requiring network access.
5. **Lightweight:** Avoid unnecessary dependencies, expensive background processing, and architectural complexity.
6. **Incremental delivery:** Each milestone must remain independently usable and testable.

## 2. Milestone Overview

| Milestone | Name | Main outcome |
|---|---|---|
| 1 | Navigation Foundation | Consistent navigation, contextual keybinding helper, searchable Help, wrap-around navigation |
| 2 | Global Search & UX Improvements | Dedicated Search Interface, live search, bulk queue action, playmode ASCII art improvement |

Custom Theme Editor is a deferred enhancement and must not block either milestone.

---

# Milestone 1 — Navigation Foundation

## 3. Objective

Improve navigation discoverability and consistency without redesigning the existing interfaces or changing established keybindings unnecessarily.

### 3.1 Contextual Keybinding Helper

**Requirements**

- Display a keybinding helper along the bottom of the interface.
- The helper must update according to the currently active interface or navigation context.
- Show approximately 3–5 of the most useful actions for the current context.
- Include relevant navigation shortcuts and context-specific actions.
- Include a discoverable shortcut to open the full Help page.
- Existing Help functionality must remain available.

Examples of contextual information:

| Context | Example actions |
|---|---|
| Songs | Navigate, play, add to queue |
| Albums | Navigate, open album |
| Artists | Navigate, open artist |
| Playlists | Navigate, open playlist |
| Queue | Navigate, remove, shuffle, repeat |
| Search | Edit query, switch category, perform action |
| Help | Navigate, search keybindings, clear query |

The examples above are illustrative. Actual displayed shortcuts must match the implemented handlers.

**Acceptance criteria**

- The helper updates when switching interfaces or entering a distinct interaction context.
- Displayed shortcuts correspond to working actions.
- The helper does not obscure content or interfere with the existing layout.
- The full Help page remains accessible from the helper.

### 3.2 Centralized Keybinding Definitions

**Requirements**

- Establish one authoritative definition for keybindings, action descriptions, and relevant context.
- Use this definition to power:
  1. Actual keybinding dispatch or handler registration.
  2. Contextual keybinding helper.
  3. Full Help page.
  4. Searchable Help results.
- Avoid separately maintained shortcut lists that can drift out of sync.

**Implementation constraint**

Inspect the current keybinding architecture first. Prefer the smallest refactor that creates a shared definition. Do not introduce a general-purpose configurable keybinding framework.

**Acceptance criteria**

- Adding or modifying a keybinding updates all relevant representations from the shared definition.
- Help does not display a shortcut that is unsupported in the corresponding context.
- Existing shortcuts retain their behavior unless a conflict must be resolved to support a new feature.

### 3.3 Searchable Help

The existing Help page must be enhanced rather than replaced.

**Requirements**

- Add a search input within the Help interface.
- Update results live as the query changes.
- Search by:
  - Keybinding.
  - Action name.
  - Action description.
  - Relevant search terms, where available.
- Provide a way to clear the query and restore the full Help list.
- Preserve existing Help navigation and keyboard usability.
- Reuse the same interaction conventions and keybinding metadata model as Global Search wherever appropriate.

**Acceptance criteria**

- Matching entries appear as the query changes.
- Clearing the query restores the complete Help list.
- An empty result set displays a useful message.
- The displayed keybinding and description match the authoritative definition.
- Search input does not accidentally trigger navigation or playback shortcuts.

### 3.4 Wrap-around Navigation

**Requirements**

Implement circular navigation for selectable lists throughout the application.

- Pressing `Up` on the first item selects the last item.
- Pressing `Down` on the last item selects the first item.
- The behavior must be consistent across Songs, Albums, Artists, Playlists, Queue, Search results, Help results, and other applicable navigation lists.
- Apply the behavior to menus and option lists where appropriate.
- Do not change arrow-key behavior inside text inputs, query editing, or other components with distinct interaction semantics.

**Edge cases**

- Empty list: navigation must not crash or produce an invalid selection.
- Single-item list: selection remains valid.
- Dynamic filtering: selection must remain valid when the number of results changes.
- If the selected item disappears after filtering, select a valid item or clear selection according to the existing selection model.

**Acceptance criteria**

- Every applicable list wraps consistently in both directions.
- Text editing remains unaffected.
- Empty and dynamically changing lists remain safe.

---

# Milestone 2 — Global Search & UX Improvements

## 4. Objective

Introduce a dedicated interface for searching the entire music library while preserving the distinct responsibilities of Songs, Albums, Artists, and Playlists interfaces.

## 4.1 Dedicated Search Interface

**Requirements**

- Add a new Search Interface.
- Do not implement Global Search as a mode that transforms an existing interface.
- The Search Interface must contain four result categories:
  - Songs
  - Albums
  - Artists
  - Playlists
- Reuse the existing tab-navigation mechanism to switch categories.
- Maintain one global query across all categories.
- Results update live while the user types.
- Switching tabs must not clear or rerun the query unnecessarily.
- The interface must have its own contextual keybinding helper.

**Empty-state copy**

Use:

`Search your library…`

Display this as a placeholder or empty-state prompt when no query has been entered.

When no results match a non-empty query, display an appropriate message such as:

`No results found.`

**Acceptance criteria**

- The Search Interface is distinct from all four existing library interfaces.
- All four categories can be accessed using the existing tab-navigation convention.
- Query text persists when switching categories.
- Empty query and no-match states are visually distinguishable.
- The interface remains usable with the keyboard alone.

## 4.2 Search Interaction and Matching

**Requirements**

- Use live search.
- Matching must be case-insensitive.
- Support substring matching.
- Support multi-word queries.
- Query terms may match relevant metadata across different fields of an entity.

Example: `radiohead kid a` should be capable of matching the album *Kid A* by Radiohead even when the query terms match different metadata fields.

- Do not implement typo-tolerant fuzzy search in this milestone.
- Search only local library data; no network request is required.
- Avoid repeated full-library scans on every keystroke if the current data model supports a more efficient approach.

**Acceptance criteria**

- Search results update as query text changes.
- Uppercase and lowercase variants produce equivalent matches.
- Multi-word queries can match relevant fields on the same entity.
- Clearing the query returns the interface to its empty state.
- Search does not interfere with playback or cause unnecessary UI blocking on ordinary library sizes.

**Technical note:** Inspect the existing library index and metadata model before deciding whether matching should be performed in memory or through SQLite. Choose the simplest approach consistent with existing architecture and acceptable responsiveness.

## 4.3 Search Result Actions

Enter must behave according to the selected entity type.

| Selected result | Enter behavior |
|---|---|
| Song | Play the selected song using existing playback semantics |
| Album | Open the corresponding Album Interface |
| Artist | Open the corresponding Artist Interface |
| Playlist | Open the corresponding Playlist Interface |

Additional actions should be exposed through the contextual keybinding helper when applicable:

- Add to Queue.
- Play Now / Replace Queue, if supported by the existing playback model.
- Open the selected entity.

Reuse existing action handlers where possible. Do not duplicate playback or queue logic specifically for Search.

### Navigation history and Back behavior

This is a critical requirement.

- If a user opens an album from Search, pressing Back must return to Search with the previous query, active category, and result selection preserved where possible.
- If a user opened Album Interface directly from elsewhere, Back must return to the actual previous interface.
- Back must not automatically navigate to Search simply because Search was previously used.
- Navigation history must reflect the actual navigation path.

Example:

`Songs → Search → Album → Back → Search`

But:

`Songs → Album → Back → Songs`

Search state should be retained when returning to an existing Search Interface instance. Avoid creating unnecessary duplicate Search Interface instances.

**Acceptance criteria**

- Back follows the actual navigation history.
- Returning to Search preserves query, active category, and selection where possible.
- Direct navigation to Album, Artist, or Playlist does not incorrectly route Back to Search.
- Existing navigation flows remain intact.

## 4.4 Search Entry and Keybindings

**Requirements**

- Add a shortcut to open Search Interface.
- Preserve existing keybindings.
- Inspect the current keybinding map before assigning the new shortcut.
- Prefer a familiar, conflict-free shortcut. `/` may be used if available and appropriate, but must not silently override an existing binding.
- Reuse the existing tab-switching mechanism for category navigation.
- Document all new shortcuts through the centralized keybinding definition.

**Acceptance criteria**

- The Search Interface can be opened from an appropriate existing navigation context.
- No existing shortcut is overwritten without an explicit decision.
- The contextual helper and Help page accurately document all new actions.

## 4.5 Bulk Add to Queue from the Current Songs Panel

**Problem**

Users can add individual songs to the queue, but adding many songs currently requires navigating into an album or playlist. This creates unnecessary steps when the user already has a list of matching songs.

**Requirements**

- Add an action to add all songs represented by the current Songs panel to the queue.
- In the regular Songs Interface, the action applies to the songs currently represented by that panel.
- In the Search Interface's Songs category, the action applies to the matching songs in the current search results.
- When no search query is active, the action applies to the complete set of songs displayed by the Songs panel.
- Preserve the existing per-song Add to Queue action.
- Reuse the existing queue insertion logic and ordering conventions.
- Do not require users to open an album or playlist first.
- The action must not unintentionally replace the current queue.
- Avoid duplicate additions caused by repeated event handling.

**Acceptance criteria**

- Users can add all songs in the current Songs panel with one action.
- Search-filtered results add only the matching songs, not the entire library.
- The ordinary Songs panel adds all songs it currently represents.
- The queue preserves the expected order of inserted songs.
- Existing playback and queue state remain valid.
- Empty result sets are handled safely.

**Implementation note:** Decide the exact shortcut after auditing existing bindings. Display the new action in the contextual helper and Help.

## 4.6 Playmode ASCII Art Improvement

**Problem**

The current ASCII rayap illustration in Play Mode is visually unsatisfactory.

**Requirements**

- Redesign or replace only the ASCII art used in Play Mode.
- Keep the rayap/ant concept.
- Prefer a simple, recognizable, visually balanced design.
- Keep the illustration compatible with the existing terminal layout and supported widths.
- Avoid complex animation or rendering logic.
- Do not redesign unrelated interfaces or the application's overall visual identity.

**Acceptance criteria**

- The new illustration is recognizable and fits the Play Mode layout.
- It does not cause clipping, unwanted line wrapping, or layout instability at supported terminal sizes.
- Playback behavior and resource usage remain unaffected.

---

# 5. Deferred Feature — Custom Theme Editor

This feature is recorded for future development but is **not required to complete Milestones 1 or 2**.

## 5.1 Intended Product Behavior

The user should eventually be able to create a custom theme through an in-app interface.

Expected capabilities:

- Create a theme and assign it a name.
- Configure colors by semantic category.
- Enter colors as HEX values.
- Save a theme.
- Select and apply a saved theme.
- Preserve existing built-in themes.

Potential semantic categories include:

- Background
- Primary Text
- Muted Text
- Accent
- Selection
- Border

The final categories should be derived from the actual UI color architecture rather than hardcoded prematurely.

## 5.2 Deferral Criteria

Before implementation, inspect the existing theme system and estimate the work required to expose semantic color tokens consistently.

Proceed only when the feature can be implemented without a disproportionately large refactor.

Do not block Global Search or Navigation Foundation on the theme editor.

---

# 6. Cross-Milestone Non-Goals

The following are explicitly out of scope:

- Replacing existing navigation conventions wholesale.
- Rewriting existing Songs, Albums, Artists, or Playlists interfaces.
- User-configurable keybindings.
- Fuzzy matching and typo correction.
- Network-based search.
- Audio similarity radio.
- Playback-history analytics and smart playlists.
- Lyrics retrieval.
- A complex mascot animation system.
- A full custom theme editor in Milestone 2.

These features may be considered in later milestones.

# 7. Implementation Workflow

For each milestone:

1. Inspect the current codebase and identify relevant interfaces, state models, navigation logic, and keybinding handlers.
2. Confirm existing behavior before changing it.
3. Write a concise implementation plan and identify files/modules likely to change.
4. Implement incrementally, reusing existing components and handlers.
5. Add or update tests for behavior and edge cases.
6. Run the existing test suite and relevant formatting/linting checks.
7. Verify terminal rendering, keyboard interactions, playback behavior, and regression cases.
8. Document any deviations from this PRD before proceeding.

Do not perform broad refactors unless a concrete requirement cannot be met with a smaller change.

# 8. Definition of Done

A milestone is complete when:

- All required acceptance criteria pass.
- Existing core playback and navigation behaviors remain functional.
- New keybindings are documented and consistent with actual handlers.
- Keyboard-only operation works throughout the affected interfaces.
- Empty states, boundary conditions, and Back navigation are tested.
- No unrelated feature or architecture changes have been introduced.
- The application remains lightweight and responsive.

**Final development priority:**

1. Complete Navigation Foundation.
2. Implement Global Search and its navigation-history behavior.
3. Add bulk queue insertion from the current Songs panel.
4. Improve Play Mode ASCII art.
5. Reassess the Custom Theme Editor separately; do not let it delay the release.
