//! Renders the application state. The UI only reads state; it never controls audio.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::keys;
use crate::models::{Focus, LibraryView, PlaybackState, SearchCategory};
use crate::theme::Theme;

/// Minimum usable terminal size before we show a warning instead of the UI.
const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 20;

/// Number of bars in the equalizer.
const EQ_BARS: usize = 7;

/// A single-line "listening to music" face. The eyes blink with `frame`.
fn mascot(frame: usize) -> String {
    let eyes = if frame % 4 == 3 { "-_-" } else { "◕_◕" };
    format!("--[ d({eyes})b ]--")
}

/// Per-bar equalizer levels, animated from the playback position.
fn eq_levels(app: &App, max: usize) -> Vec<usize> {
    let t = app.position().map(|d| d.as_secs_f64()).unwrap_or(0.0);
    (0..EQ_BARS)
        .map(|i| match app.playback {
            PlaybackState::Playing => {
                let v = (t * 2.2 + i as f64 * 0.85).sin() * 0.5 + 0.5;
                (v * max as f64).round() as usize
            }
            PlaybackState::Paused => 1,
            PlaybackState::Stopped => 0,
        })
        .collect()
}

/// A single-line equalizer using block characters.
fn equalizer_line(app: &App) -> String {
    const BLOCKS: [char; 8] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '█'];
    eq_levels(app, 7)
        .iter()
        .map(|&level| BLOCKS[level.min(7)])
        .collect()
}

/// A taller equalizer rendered as vertical bars, one line per row.
fn equalizer_rows(app: &App, height: usize) -> Vec<String> {
    let levels = eq_levels(app, height);
    (1..=height)
        .rev()
        .map(|row| {
            let mut line = String::new();
            for &level in &levels {
                line.push(if level >= row { '█' } else { ' ' });
                line.push(' ');
            }
            line
        })
        .collect()
}

/// Animation frame derived from playback position (frozen while paused).
fn anim_frame(app: &App) -> usize {
    app.position().map(|d| d.as_millis() / 300).unwrap_or(0) as usize
}

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
        draw_help(frame, area, app);
        return;
    }

    if app.play_mode {
        draw_play_mode(frame, area, app);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
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
    draw_keys(frame, rows[4], app);
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
            Line::from(vec![
                Span::styled("♪ ", Style::default().fg(theme.accent_alt)),
                Span::styled(equalizer_line(app), Style::default().fg(theme.playing)),
                Span::styled(
                    format!("   ({})", app.playback.label()),
                    Style::default().fg(theme.muted),
                ),
            ]),
        ],
        None => vec![
            Line::from(Span::styled(
                "No track playing  (._.)",
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
        Span::styled("   ", Style::default()),
        Span::styled(progress_bar(ratio, 24), Style::default().fg(theme.accent)),
        Span::styled(
            format!("  {}% {}", app.volume, volume_bar(app.volume)),
            Style::default().fg(theme.muted),
        ),
        Span::styled(
            format!(
                "  Shuffle:{} ⇄   Repeat:{} ↻",
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

/// A smooth progress bar using block characters with a fractional tip.
fn progress_bar(ratio: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let scaled = ratio.clamp(0.0, 1.0) * width as f64;
    let full = scaled.floor() as usize;
    let frac = scaled - full as f64;
    const EIGHTHS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];

    let mut out = String::with_capacity(width);
    for index in 0..width {
        match index.cmp(&full) {
            std::cmp::Ordering::Less => out.push('█'),
            std::cmp::Ordering::Equal => {
                let tip = ((frac * 8.0).round() as usize).min(7);
                out.push(if tip == 0 { '░' } else { EIGHTHS[tip] });
            }
            std::cmp::Ordering::Greater => out.push('░'),
        }
    }
    out
}

/// A small ten-segment volume meter.
fn volume_bar(percent: u8) -> String {
    let segments = (percent as usize * 10).div_ceil(100);
    format!("{}{}", "▮".repeat(segments), "▯".repeat(10 - segments))
}

fn draw_library(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Library;
    let title = library_title(app, focused);
    let block = panel_block(&title, focused, theme);

    let rows = library_rows(app);
    if rows.is_empty() {
        let message = if app.view == LibraryView::Search {
            if app.search_query.is_empty() {
                "Search your library…  (^_^)".to_string()
            } else {
                "No results found.  (._.)".to_string()
            }
        } else {
            r"(empty) ¯\_(ツ)_/¯".to_string()
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
    if let Some(label) = app.detail_label() {
        return format!("{marker} {label}");
    }
    match app.view {
        LibraryView::Search => format!(
            "{marker} Search [{}]: \"{}\"",
            app.search_category.label(),
            app.search_query
        ),
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
    if app.detail_open() {
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
        LibraryView::Search => search_rows(app),
    }
}

fn search_rows(app: &App) -> Vec<String> {
    match app.search_category {
        SearchCategory::Songs => app
            .search_results
            .songs
            .iter()
            .map(|track| format!("{} — {}", track.artist_label(), track.title))
            .collect(),
        SearchCategory::Albums => app
            .search_results
            .albums
            .iter()
            .map(|album| format!("{} — {}", album.title, album.artist_label))
            .collect(),
        SearchCategory::Artists => app
            .search_results
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect(),
        SearchCategory::Playlists => app
            .search_results
            .playlists
            .iter()
            .map(|playlist| playlist.name.clone())
            .collect(),
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
    let first_line = app.prompt_display().unwrap_or_else(|| app.status.clone());
    let face = if app.playback == PlaybackState::Playing {
        "(^_^)"
    } else if app.playback == PlaybackState::Paused {
        "(-_-)"
    } else {
        "(._.)"
    };
    let text = vec![Line::from(vec![
        Span::styled(first_line, Style::default().fg(theme.accent_alt)),
        Span::styled(format!("   {face}"), Style::default().fg(theme.muted)),
    ])];
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Status", false, theme)),
        area,
    );
}

/// A compact, context-aware keybinding helper panel from the shared table.
fn draw_keys(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let mut spans: Vec<Span> = Vec::new();
    let bindings = keys::for_context(app.context());
    for (index, binding) in bindings.into_iter().take(5).enumerate() {
        if index > 0 {
            spans.push(Span::styled("   ", Style::default()));
        }
        spans.push(Span::styled(
            format!("{} ", binding.label),
            Style::default()
                .fg(theme.accent_alt)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            binding.desc.to_string(),
            Style::default().fg(theme.muted),
        ));
    }
    spans.push(Span::styled(
        "   ? help",
        Style::default().fg(theme.accent_alt),
    ));

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(panel_block("Keys", false, theme))
            .wrap(Wrap { trim: true }),
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
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(inner);

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
        chunks[0],
    );

    let show_mascot = inner.width >= 24;
    let mut art_lines: Vec<Line> = if show_mascot {
        vec![Line::from(Span::styled(
            mascot(anim_frame(app)),
            Style::default()
                .fg(theme.accent_alt)
                .add_modifier(Modifier::BOLD),
        ))]
    } else {
        vec![Line::from(Span::styled(
            "d(o_o)b",
            Style::default().fg(theme.muted),
        ))]
    };
    art_lines.push(Line::from(Span::styled(
        "♪ ♫ ♪",
        Style::default().fg(theme.accent),
    )));
    for row in equalizer_rows(app, 5) {
        art_lines.push(Line::from(Span::styled(
            row,
            Style::default().fg(theme.playing),
        )));
    }
    frame.render_widget(
        Paragraph::new(art_lines).alignment(Alignment::Center),
        chunks[1],
    );

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
                "{} / {}   {}   {}% {}",
                format_duration(position),
                format_duration(duration),
                progress_bar(ratio, 24),
                app.volume,
                volume_bar(app.volume)
            ),
            Style::default().fg(theme.foreground),
        )),
        Line::from(Span::styled(
            format!(
                "{}    Shuffle:{} ⇄    Repeat:{} ↻",
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
        chunks[2],
    );
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title("Help  (^_^)");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    let search_text = if app.help_query.is_empty() {
        "Type to search keys…".to_string()
    } else {
        format!("Search: {}\u{2588}", app.help_query)
    };
    frame.render_widget(
        Paragraph::new(search_text)
            .style(Style::default().fg(theme.accent_alt))
            .block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    let entries = app.help_entries();
    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No matching keys.").style(Style::default().fg(theme.muted)),
            chunks[1],
        );
    } else {
        let inner_height = chunks[1].height.saturating_sub(2) as usize;
        let offset = scroll_offset(app.help_selected, inner_height, entries.len());
        let lines: Vec<Line> = entries
            .iter()
            .enumerate()
            .map(|(index, binding)| {
                let selected = index == app.help_selected;
                let style = if selected {
                    Style::default()
                        .bg(theme.highlight_bg)
                        .fg(theme.highlight_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground)
                };
                Line::from(format!("{:<20} {}", binding.label, binding.desc)).style(style)
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), chunks[1]);
    }

    frame.render_widget(
        Paragraph::new(Span::styled(
            "Type to search   Esc: clear/close   ?: close",
            Style::default().fg(theme.muted),
        )),
        chunks[2],
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
