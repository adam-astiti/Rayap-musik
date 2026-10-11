//! Color themes.
//!
//! Themes are defined once through a palette struct so UI code never scatters
//! colour decisions across widgets.

use ratatui::style::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub muted: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub playing: Color,
    pub warn: Color,
}

/// Resolve a theme by name, falling back to the default on unknown input.
pub fn theme_from_name(name: &str) -> Theme {
    match name.to_ascii_lowercase().as_str() {
        "catppuccin" => catppuccin(),
        "dracula" => dracula(),
        "nord" => nord(),
        _ => rust_walnut(),
    }
}

/// The default theme: warm, organic "Rust & Walnut".
pub fn rust_walnut() -> Theme {
    Theme {
        name: "rust-walnut".to_string(),
        background: hex("242320"),
        foreground: hex("E6D8C4"),
        accent: hex("A64B32"),
        accent_alt: hex("D7B995"),
        muted: hex("8A8374"),
        highlight_bg: hex("A64B32"),
        highlight_fg: hex("FFF8EE"),
        playing: hex("7FA65A"),
        warn: hex("E3B341"),
    }
}

pub fn catppuccin() -> Theme {
    Theme {
        name: "catppuccin".to_string(),
        background: hex("1E1E2E"),
        foreground: hex("CDD6F4"),
        accent: hex("CBA6F7"),
        accent_alt: hex("89B4FA"),
        muted: hex("6C7086"),
        highlight_bg: hex("CBA6F7"),
        highlight_fg: hex("1E1E2E"),
        playing: hex("A6E3A1"),
        warn: hex("F9E2AF"),
    }
}

pub fn dracula() -> Theme {
    Theme {
        name: "dracula".to_string(),
        background: hex("282A36"),
        foreground: hex("F8F8F2"),
        accent: hex("BD93F9"),
        accent_alt: hex("50FA7B"),
        muted: hex("6272A4"),
        highlight_bg: hex("BD93F9"),
        highlight_fg: hex("282A36"),
        playing: hex("50FA7B"),
        warn: hex("F1FA8C"),
    }
}

pub fn nord() -> Theme {
    Theme {
        name: "nord".to_string(),
        background: hex("2E3440"),
        foreground: hex("ECEFF4"),
        accent: hex("88C0D0"),
        accent_alt: hex("5E81AC"),
        muted: hex("4C566A"),
        highlight_bg: hex("5E81AC"),
        highlight_fg: hex("ECEFF4"),
        playing: hex("A3BE8C"),
        warn: hex("EBCB8B"),
    }
}

fn hex(value: &str) -> Color {
    let value = value.trim_start_matches('#');
    let rgb = u32::from_str_radix(value, 16).unwrap_or(0x00FF_FFFF);
    Color::Rgb(
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::theme_from_name;

    #[test]
    fn unknown_name_falls_back_to_default() {
        assert_eq!(theme_from_name("bogus").name, "rust-walnut");
        assert_eq!(theme_from_name("").name, "rust-walnut");
    }

    #[test]
    fn known_themes_resolve() {
        assert_eq!(theme_from_name("catppuccin").name, "catppuccin");
        assert_eq!(theme_from_name("Dracula").name, "dracula");
        assert_eq!(theme_from_name("NORD").name, "nord");
        assert_eq!(theme_from_name("rust-walnut").name, "rust-walnut");
    }
}
