//! Library catalog: grouping and sorting of indexed tracks, plus global search.
//!
//! Pure domain logic with no UI or database dependencies. It is built from the
//! indexed tracks and re-generated whenever the index changes.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::library::compare_tracks;
use crate::models::{Playlist, SearchCategory, Track};

/// A collection of tracks grouped by album name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub title: String,
    /// Artist of the first track in deterministic order (never "Various Artists").
    pub artist_label: String,
    pub tracks: Vec<Track>,
}

/// A collection of tracks grouped by artist name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub name: String,
    pub tracks: Vec<Track>,
}

/// The in-memory view of the library.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    pub songs: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

/// Results of a global search, split into the four result categories.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchResults {
    pub songs: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

impl SearchResults {
    /// Number of results in a given category.
    pub fn count(&self, category: SearchCategory) -> usize {
        match category {
            SearchCategory::Songs => self.songs.len(),
            SearchCategory::Albums => self.albums.len(),
            SearchCategory::Artists => self.artists.len(),
            SearchCategory::Playlists => self.playlists.len(),
        }
    }
}

/// Build a deterministically sorted and grouped catalog from indexed tracks.
pub fn build_catalog(tracks: Vec<Track>) -> Catalog {
    let songs = sort_songs(tracks);
    let albums = group_albums(&songs);
    let artists = group_artists(&songs);
    Catalog {
        songs,
        albums,
        artists,
    }
}

/// Global, case-insensitive search across songs, albums, artists, and
/// playlists, returning results grouped by category.
///
/// Matching is substring-based and multi-word: the query is split on
/// whitespace and every term must match at least one field of an entity
/// (e.g. `radiohead kid a` matches the album "Kid A" by Radiohead).
pub fn search(catalog: &Catalog, playlists: &[Playlist], query: &str) -> SearchResults {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|term| term.to_lowercase())
        .collect();
    if terms.is_empty() {
        return SearchResults::default();
    }
    let terms: Vec<&str> = terms.iter().map(String::as_str).collect();

    let songs = catalog
        .songs
        .iter()
        .filter(|song| {
            let fields = [
                song.title.as_str(),
                song.artist.as_deref().unwrap_or(""),
                song.album.as_deref().unwrap_or(""),
            ];
            matches_all_terms(&terms, &fields)
        })
        .cloned()
        .collect();

    let albums = catalog
        .albums
        .iter()
        .filter(|album| {
            let fields = [album.title.as_str(), album.artist_label.as_str()];
            matches_all_terms(&terms, &fields)
        })
        .cloned()
        .collect();

    let artists = catalog
        .artists
        .iter()
        .filter(|artist| matches_all_terms(&terms, &[artist.name.as_str()]))
        .cloned()
        .collect();

    let playlists = playlists
        .iter()
        .filter(|playlist| matches_all_terms(&terms, &[playlist.name.as_str()]))
        .cloned()
        .collect();

    SearchResults {
        songs,
        albums,
        artists,
        playlists,
    }
}

/// Every term must match at least one field.
fn matches_all_terms(terms: &[&str], fields: &[&str]) -> bool {
    terms
        .iter()
        .all(|term| fields.iter().any(|field| contains_ci(field, term)))
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

fn ci(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

/// Songs ordered by artist, then album, then track number, then file name.
fn sort_songs(mut songs: Vec<Track>) -> Vec<Track> {
    songs.sort_by(|a, b| {
        ci(a.artist_label(), b.artist_label())
            .then_with(|| ci(a.album_label(), b.album_label()))
            .then_with(|| compare_tracks(a, b))
    });
    songs
}

fn group_albums(songs: &[Track]) -> Vec<Album> {
    let mut albums: Vec<Album> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for track in songs {
        let title = track.album_label().to_string();
        let key = title.to_lowercase();
        match index.get(&key) {
            Some(&i) => albums[i].tracks.push(track.clone()),
            None => {
                index.insert(key, albums.len());
                albums.push(Album {
                    title,
                    artist_label: track.artist_label().to_string(),
                    tracks: vec![track.clone()],
                });
            }
        }
    }

    for album in &mut albums {
        album.tracks.sort_by(compare_tracks);
        album.artist_label = album.tracks[0].artist_label().to_string();
    }

    albums
        .sort_by(|a, b| ci(&a.title, &b.title).then_with(|| ci(&a.artist_label, &b.artist_label)));
    albums
}

fn group_artists(songs: &[Track]) -> Vec<Artist> {
    let mut artists: Vec<Artist> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for track in songs {
        let name = track.artist_label().to_string();
        let key = name.to_lowercase();
        match index.get(&key) {
            Some(&i) => artists[i].tracks.push(track.clone()),
            None => {
                index.insert(key, artists.len());
                artists.push(Artist {
                    name,
                    tracks: vec![track.clone()],
                });
            }
        }
    }

    artists.sort_by(|a, b| ci(&a.name, &b.name));
    artists
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{build_catalog, search};
    use crate::models::{Playlist, Track};

    fn track(title: &str, artist: &str, album: &str, number: Option<u32>) -> Track {
        Track {
            path: PathBuf::from(format!("{artist}/{album}/{title}.mp3")),
            title: title.to_string(),
            artist: Some(artist.to_string()),
            album: Some(album.to_string()),
            track_number: number,
            duration: None,
        }
    }

    fn song_titles(results: &super::SearchResults) -> Vec<String> {
        results.songs.iter().map(|t| t.title.clone()).collect()
    }

    #[test]
    fn songs_sorted_by_artist_album_track() {
        let catalog = build_catalog(vec![
            track("b", "Zed", "One", Some(1)),
            track("a", "Zed", "One", Some(2)),
            track("c", "Ally", "Two", Some(1)),
        ]);

        let titles: Vec<String> = catalog.songs.iter().map(|t| t.title.clone()).collect();
        assert_eq!(titles, vec!["c", "b", "a"]);
    }

    #[test]
    fn albums_merge_same_name_across_artists() {
        let catalog = build_catalog(vec![
            track("01", "Artist A", "Greatest Hits", Some(1)),
            track("01", "Artist B", "Greatest Hits", Some(1)),
            track("02", "Artist B", "Greatest Hits", Some(2)),
        ]);

        assert_eq!(catalog.albums.len(), 1, "same-name albums merge");
        let album = &catalog.albums[0];
        assert_eq!(album.title, "Greatest Hits");
        assert_eq!(album.tracks.len(), 3);
        // Track #1 from "Artist A" sorts first, so it supplies the label.
        assert_eq!(album.artist_label, "Artist A");
        assert!(
            !album.artist_label.contains("Various"),
            "no auto Various Artists"
        );
    }

    #[test]
    fn album_members_sorted_by_track_number() {
        let catalog = build_catalog(vec![
            track("03", "Artist", "Album", Some(3)),
            track("01", "Artist", "Album", Some(1)),
            track("02", "Artist", "Album", Some(2)),
        ]);

        let numbers: Vec<Option<u32>> = catalog.albums[0]
            .tracks
            .iter()
            .map(|t| t.track_number)
            .collect();
        assert_eq!(numbers, vec![Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn search_is_case_insensitive_and_categorized() {
        let catalog = build_catalog(vec![
            track("Hello", "World", "Greetings", Some(1)),
            track("Other", "Worldly", "Stuff", Some(1)),
        ]);
        let playlists = vec![Playlist {
            id: 1,
            name: "World Tour".to_string(),
        }];

        let results = search(&catalog, &playlists, "WORLD");

        assert_eq!(song_titles(&results), vec!["Hello", "Other"]);
        assert_eq!(results.artists.len(), 2, "both World and Worldly match");
        assert_eq!(results.playlists.len(), 1);
        assert_eq!(results.playlists[0].name, "World Tour");
    }

    #[test]
    fn search_matches_multi_word_across_fields() {
        let catalog = build_catalog(vec![track("01", "Radiohead", "Kid A", Some(1))]);
        let results = search(&catalog, &[], "radiohead kid a");
        assert_eq!(results.albums.len(), 1);
        assert_eq!(results.albums[0].title, "Kid A");
    }

    #[test]
    fn search_requires_all_terms() {
        let catalog = build_catalog(vec![track("01", "Radiohead", "Kid A", Some(1))]);
        assert!(search(&catalog, &[], "radiohead moon").albums.is_empty());
    }

    #[test]
    fn empty_search_returns_nothing() {
        let catalog = build_catalog(vec![track("a", "b", "c", Some(1))]);
        let results = search(&catalog, &[], "   ");
        assert!(results.songs.is_empty());
        assert!(results.albums.is_empty());
        assert!(results.artists.is_empty());
        assert!(results.playlists.is_empty());
    }
}
