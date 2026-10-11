//! Artwork resolution.
//!
//! Artwork is looked up in priority order: embedded picture, then a local
//! image in the album folder, then the Rayap Musik placeholder. Never fetched
//! from the network.

use std::path::PathBuf;

use lofty::file::TaggedFileExt;

use crate::models::Track;

/// Image file names looked up in the album folder, in priority order.
pub const FOLDER_IMAGES: &[&str] = &[
    "cover.jpg",
    "cover.png",
    "folder.jpg",
    "folder.png",
    "front.jpg",
    "front.png",
    "album.jpg",
    "album.png",
    "albumart.jpg",
    "albumart.png",
    "art.jpg",
    "art.png",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Artwork {
    Embedded { data: Vec<u8>, mime: Option<String> },
    FolderImage(PathBuf),
    Placeholder,
}

impl Artwork {
    /// Human-readable source label.
    pub fn label(&self) -> String {
        match self {
            Artwork::Embedded { mime, .. } => match mime {
                Some(mime) => format!("Embedded artwork ({mime})"),
                None => "Embedded artwork".to_string(),
            },
            Artwork::FolderImage(_) => "Folder image".to_string(),
            Artwork::Placeholder => "Rayap Musik".to_string(),
        }
    }
}

/// Resolve the artwork for a track, following the priority order.
pub fn resolve(track: &Track) -> Artwork {
    if let Ok(tagged) = lofty::read_from_path(&track.path) {
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if let Some(picture) = tag.pictures().first() {
                let data = picture.data().to_vec();
                if !data.is_empty() {
                    let mime = picture.mime_type().map(|mime| mime.as_str().to_string());
                    return Artwork::Embedded { data, mime };
                }
            }
        }
    }

    if let Some(parent) = track.path.parent() {
        for name in FOLDER_IMAGES {
            let path = parent.join(name);
            if path.is_file() {
                return Artwork::FolderImage(path);
            }
        }
    }

    Artwork::Placeholder
}
