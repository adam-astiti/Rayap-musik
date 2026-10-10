//! Renders the application state. The UI only reads state; it never controls audio.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::models::Focus;

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
            Constraint::Length(3),
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
    let track = match &app.root {
        Some(_) => "No track loaded",
        None => "No root directory provided",
    };
    let text = vec![
        Line::from(Span::styled(
            track.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Status: {}", app.playback.label())),
    ];
    frame.render_widget(
        Paragraph::new(text).block(panel_block("Now Playing", false)),
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
    frame.render_widget(
        Paragraph::new("(empty)")
            .style(Style::default().fg(Color::DarkGray))
            .block(panel_block(title, focused)),
        area,
    );
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
    let help = "Tab: focus  Space: play/pause  n/p: track  +/-: volume  q: quit";
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
