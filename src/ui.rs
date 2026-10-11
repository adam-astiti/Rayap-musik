//! Renders the application state. The UI only reads state; it never controls audio.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::catalog::SearchEntry;
use crate::models::{Focus, LibraryView};
use crate::theme::Theme;

/// Minimum usable terminal size before we show a warning instead of the UI.
const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 20;

/// ASCII termite mascot. Kept compact so it fits in most terminals.
const MASCOT: &[&str] = &[
    "   \\   /   ",
    "    \\ /    ",
    "    / \\    ",
    "   / O O \\ ",
    "   |  _  | ",
    "   \\_/ \\_/ ",
    "    _|||_   ",
];

const HELP_LINES: &[(&str, &str)] = &[
    ("Up / Down", "Move selection"),
    ("Tab", "Switch focus (Library / Queue)"),
    ("1 / 2 / 3 / 4", "Songs / Albums / Artists / Playlists"),
    ("/", "Search (Esc to leave)"),
    ("Enter", "Play Now / open playlist"),
    ("a", "Add selection to queue"),
    ("N", "Play Next (insert after active)"),
    ("d", "Remove queue entry (Queue view)"),
    ("s", "Toggle shuffle"),
    ("R", "Cycle repeat (Off/All/One)"),
    ("m", "Toggle Play Mode"),
    ("T", "Cycle theme"),
    ("?", "Toggle this help"),
    (
        "c / e / D / x",
        "Create / rename / duplicate / delete playlist",
    ),
    ("P", "Add selection to a playlist"),
    ("I / E", "Import / export M3U"),
    ("J / K / S", "Move track down / up / sort (in playlist)"),
    ("Esc / Backspace", "Back (leave playlist, cancel picker)"),
    ("Space", "Play / Pause"),
    ("n / p", "Next / previous track"),
    ("+ / -", "Volume up / down"),
    ("r", "Rescan library"),
    ("q", "Quit"),
];

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.theme;

    frame.render_widget(
        Block::default().style(Style::default().bg(theme.background)),
        area,
    );

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area, theme);
        return;
    }

    if app.show_help {
        draw_help(frame, area, theme);
        return;
    }

    if app.play_mode {
        draw_play_mode(frame, area, app);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);

    draw_now_playing(frame, rows[0], app);
    draw_progress(frame, rows[1], app);
    draw_library(frame, middle[0], app);
    draw_queue(frame, middle[1], app);
    draw_status(frame, rows[3], app);
}

fn draw_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let message = format!(
        "Terminal too small\nNeed at least {MIN_WIDTH}x{MIN_HEIGHT}\nCurrent: {}x{}",
        area.width, area.height
    );
    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Rayap Musik")
                .border_style(Style::default().fg(theme.warn)),
        )
        .style(Style::default().fg(theme.warn))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn panel_block<'a>(title: &'a str, focused: bool, theme: &Theme) -> Block<'a> {
    let style = if focused {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(Span::styled(title.to_string(), style))
}

fn draw_now_playing(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lines = match app.current_track() {
        Some(track) => vec![
            Line::from(Span::styled(
                track.title.clone(),
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{}  |  {}", track.artist_label(), track.album_label()),
                Style::default().fg(theme.accent_alt),
            )),
            Line::from(Span::styled(
                format!(
                    "Duration: {}   ({})",
                    format_duration(track.duration),
                    app.playback.label()
                ),
                Style::default().fg(theme.muted),
            )),
        ],
        None => vec![
            Line::from(Span::styled(
                "No track playing",
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Status: {}", app.playback.label()),
                Style::default().fg(theme.muted),
            )),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines).block(panel_block("Now Playing", false, theme)),
        area,
    );
}

fn draw_progress(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let position = app.position();
    let duration = app.duration();
    let ratio = match (position, duration) {
        (Some(position), Some(duration)) if !duration.is_zero() => {
            position.as_secs_f64() / duration.as_secs_f64()
        }
        _ => 0.0,
    };
    let text = Line::from(vec![
        Span::styled(
            format!(
                "{} / {}",
                format_duration(position),
                format_duration(duration)
            ),
            Style::default().fg(theme.foreground),
        ),
        Span::styled(
            format!("    Volume: {}%    [", app.volume),
            Style::default().fg(theme.muted),
        ),
        Span::styled(
            progress_bar(ratio, 20),
            Style::default().fg(theme.accent_alt),
        ),
        Span::styled(
            format!(
                "]    Shuffle:{}  Repeat:{}",
                if app.queue.is_shuffled() { "on" } else { "off" },
                app.repeat.label()
            ),
            Style::default().fg(theme.muted),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Progress", false, theme)),
        area,
    );
}

fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "#".repeat(filled), "-".repeat(empty))
}

fn draw_library(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Library;
    let title = library_title(app, focused);
    let block = panel_block(&title, focused, theme);

    let rows = library_rows(app);
    if rows.is_empty() {
        let message = if app.view == LibraryView::Search && app.search_query.is_empty() {
            "Type to search".to_string()
        } else {
            "(empty)".to_string()
        };
        frame.render_widget(
            Paragraph::new(message)
                .style(Style::default().fg(theme.muted))
                .block(block),
            area,
        );
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let offset = if focused {
        scroll_offset(app.selected, inner_height, rows.len())
    } else {
        0
    };

    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let selected = focused && index == app.selected;
            let style = if selected {
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.highlight_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            Line::from(label.clone()).style(style)
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((offset as u16, 0)),
        area,
    );
}

fn library_title(app: &App, focused: bool) -> String {
    let marker = if focused { ">" } else { " " };
    if app.picker_active() {
        return format!("{marker} Add to playlist");
    }
    if let Some(playlist) = &app.open_playlist {
        return format!("{marker} Playlist: {}", playlist.name);
    }
    match app.view {
        LibraryView::Search => format!("{marker} Search: \"{}\"", app.search_query),
        view => format!("{marker} {}", view.label()),
    }
}

fn library_rows(app: &App) -> Vec<String> {
    if app.picker_active() {
        return app
            .playlists
            .iter()
            .map(|playlist| playlist.name.clone())
            .collect();
    }
    if app.open_playlist.is_some() {
        return app
            .open_tracks
            .iter()
            .map(|track| format!("{} — {}", track.artist_label(), track.title))
            .collect();
    }
    match app.view {
        LibraryView::Songs => app
            .catalog
            .songs
            .iter()
            .map(|track| format!("{} — {}", track.artist_label(), track.title))
            .collect(),
        LibraryView::Albums => app
            .catalog
            .albums
            .iter()
            .map(|album| format!("{} — {}", album.title, album.artist_label))
            .collect(),
        LibraryView::Artists => app
            .catalog
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect(),
        LibraryView::Playlists => app
            .playlists
            .iter()
            .map(|playlist| playlist.name.clone())
            .collect(),
        LibraryView::Search => app.search_results.iter().map(search_label).collect(),
    }
}

fn search_label(entry: &SearchEntry) -> String {
    match entry {
        SearchEntry::Song(track) => format!("[Song] {} — {}", track.artist_label(), track.title),
        SearchEntry::Album(album) => format!("[Album] {} — {}", album.title, album.artist_label),
        SearchEntry::Artist(artist) => format!("[Artist] {}", artist.name),
        SearchEntry::Playlist(playlist) => format!("[Playlist] {}", playlist.name),
    }
}

fn scroll_offset(selected: usize, height: usize, len: usize) -> usize {
    if height == 0 || len <= height {
        return 0;
    }
    if selected < height {
        0
    } else {
        selected + 1 - height
    }
}

fn draw_queue(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Queue;
    let title = if focused { "> Queue" } else { "  Queue" };
    let block = panel_block(title, focused, theme);

    if app.queue.is_empty() {
        frame.render_widget(
            Paragraph::new("(empty)")
                .style(Style::default().fg(theme.muted))
                .block(block),
            area,
        );
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let offset = if focused {
        scroll_offset(app.queue_selected, inner_height, app.queue.len())
    } else {
        0
    };

    let current = app.queue.current_index();
    let lines: Vec<Line> = app
        .queue
        .tracks()
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let marker = if index == current { ">" } else { " " };
            let style = if focused && index == app.queue_selected {
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.highlight_fg)
                    .add_modifier(Modifier::BOLD)
            } else if index == current {
                Style::default().fg(theme.playing)
            } else {
                Style::default().fg(theme.foreground)
            };
            Line::from(format!("{marker} {:>2}. {}", index + 1, track.title)).style(style)
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((offset as u16, 0)),
        area,
    );
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let help = "1-4 view  / search  Enter play  a add  N next  s shuffle  R repeat  c new  e rename  D dup  x del  P add-to  I/E m3u  ? help";
    let first_line = app.prompt_display().unwrap_or_else(|| app.status.clone());
    let text = vec![
        Line::from(Span::styled(
            first_line,
            Style::default().fg(theme.accent_alt),
        )),
        Line::from(Span::styled(help, Style::default().fg(theme.muted))),
    ];
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Status / Help", false, theme)),
        area,
    );
}

fn draw_play_mode(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title("Now Playing  (m: back)");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(MASCOT.len() as u16 + 1),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(1),
        ])
        .split(inner);

    let show_mascot = inner.width >= 24;

    let mascot_lines: Vec<Line> = if show_mascot {
        MASCOT
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.accent_alt),
                ))
            })
            .collect()
    } else {
        vec![Line::from(Span::styled(
            "(rayap musik)",
            Style::default().fg(theme.muted),
        ))]
    };
    frame.render_widget(
        Paragraph::new(mascot_lines).alignment(Alignment::Center),
        chunks[0],
    );

    let info_lines: Vec<Line> = match app.current_track() {
        Some(track) => vec![
            Line::from(Span::styled(
                track.title.clone(),
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{}  |  {}", track.artist_label(), track.album_label()),
                Style::default().fg(theme.accent_alt),
            )),
        ],
        None => vec![Line::from(Span::styled(
            "No track playing",
            Style::default().fg(theme.foreground),
        ))],
    };
    frame.render_widget(
        Paragraph::new(info_lines).alignment(Alignment::Center),
        chunks[1],
    );

    draw_artwork_box(frame, chunks[2], app);

    let position = app.position();
    let duration = app.duration();
    let ratio = match (position, duration) {
        (Some(position), Some(duration)) if !duration.is_zero() => {
            position.as_secs_f64() / duration.as_secs_f64()
        }
        _ => 0.0,
    };
    let status_lines = vec![
        Line::from(Span::styled(
            format!(
                "{} / {}    Volume: {}%    [{}]",
                format_duration(position),
                format_duration(duration),
                app.volume,
                progress_bar(ratio, 24)
            ),
            Style::default().fg(theme.foreground),
        )),
        Line::from(Span::styled(
            format!(
                "{}    Shuffle:{}    Repeat:{}",
                app.playback.label(),
                if app.queue.is_shuffled() { "on" } else { "off" },
                app.repeat.label()
            ),
            Style::default().fg(theme.muted),
        )),
        Line::from(Span::styled(
            "Space: play/pause  n/p: track  +/-: volume  m: back  q: quit",
            Style::default().fg(theme.muted),
        )),
    ];
    frame.render_widget(
        Paragraph::new(status_lines).alignment(Alignment::Center),
        chunks[3],
    );
}

fn draw_artwork_box(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let centered = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area)[1];

    let initial = app
        .current_track()
        .map(|track| {
            track
                .album_label()
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_else(|| "?".to_string())
        })
        .unwrap_or_else(|| "?".to_string());

    let label = app
        .artwork
        .as_ref()
        .map(artwork_label)
        .unwrap_or_else(|| "Rayap Musik".to_string());

    let lines = vec![
        Line::from(Span::styled(
            initial.clone(),
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(label, Style::default().fg(theme.highlight_fg))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.highlight_bg));

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block),
        centered,
    );
}

fn artwork_label(artwork: &crate::artwork::Artwork) -> String {
    artwork.label()
}

fn draw_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        "Rayap Musik — keys",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(""));

    for (key, desc) in HELP_LINES {
        lines.push(Line::from(Span::styled(
            format!("{:<20} {desc}", key),
            Style::default().fg(theme.foreground),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press ? to close",
        Style::default().fg(theme.muted),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title("Help");

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(theme.foreground)),
        area,
    );
}

fn format_duration(duration: Option<std::time::Duration>) -> String {
    match duration {
        Some(duration) => {
            let total = duration.as_secs();
            format!("{:02}:{:02}", total / 60, total % 60)
        }
        None => "--:--".to_string(),
    }
}
