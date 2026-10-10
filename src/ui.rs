//! Renders the application state. The UI only reads state; it never controls audio.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::models::{Focus, LibraryItem};

/// Minimum usable terminal size before we show a warning instead of the UI.
const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 20;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area);
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

fn draw_too_small(frame: &mut Frame, area: Rect) {
    let message = format!(
        "Terminal too small\nNeed at least {MIN_WIDTH}x{MIN_HEIGHT}\nCurrent: {}x{}",
        area.width, area.height
    );
    let paragraph = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title("Rayap Musik"))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn panel_block(title: &str, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(Span::styled(title.to_string(), style))
}

fn draw_now_playing(frame: &mut Frame, area: Rect, app: &App) {
    let lines = match app.selected_item().and_then(LibraryItem::as_track) {
        Some(track) => vec![
            Line::from(Span::styled(
                track.title.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "{}  |  {}",
                track.artist_label(),
                track.album_label()
            )),
            Line::from(format!(
                "Duration: {}   ({})",
                format_duration(track.duration),
                app.playback.label()
            )),
        ],
        None => vec![
            Line::from(Span::styled(
                "No track selected",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Status: {}", app.playback.label())),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines).block(panel_block("Now Playing", false)),
        area,
    );
}

fn draw_progress(frame: &mut Frame, area: Rect, app: &App) {
    let text = Line::from(format!(
        "00:00 / --:--    Volume: {}%    [{}]",
        app.volume,
        progress_bar(0.0, 20)
    ));
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Progress", false)),
        area,
    );
}

fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "#".repeat(filled), "-".repeat(empty))
}

fn draw_library(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Library;
    let title = if focused { "> Library" } else { "  Library" };
    let block = panel_block(title, focused);

    if app.items.is_empty() {
        frame.render_widget(
            Paragraph::new("(empty)")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let offset = if focused {
        scroll_offset(app.selected, inner_height, app.items.len())
    } else {
        0
    };

    let lines: Vec<Line> = app
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let selected = focused && index == app.selected;
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(item_label(item)).style(style)
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((offset as u16, 0)),
        area,
    );
}

fn item_label(item: &LibraryItem) -> String {
    match item {
        LibraryItem::Directory { name, .. } => format!("[dir] {name}/"),
        LibraryItem::Track(track) => match track.track_number {
            Some(number) => format!("{number:>3}. {}", track.title),
            None => format!("     {}", track.title),
        },
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
    let focused = app.focus == Focus::Queue;
    let title = if focused { "> Queue" } else { "  Queue" };
    frame.render_widget(
        Paragraph::new("(empty)")
            .style(Style::default().fg(Color::DarkGray))
            .block(panel_block(title, focused)),
        area,
    );
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let help = "Tab: focus  Enter: open/play  Backspace: parent  Space: play  n/p: track  +/-: vol  q: quit";
    let text = vec![
        Line::from(Span::styled(
            app.status.clone(),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(help, Style::default().fg(Color::DarkGray))),
    ];
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Status / Help", false)),
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
