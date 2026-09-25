use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
// use ratatui::prelude::Rect;
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::weather::{art_lines, format_temp};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Min(5),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_weather(frame, chunks[1], app);
    draw_heartbeat(frame, chunks[2], app);
    draw_news(frame, chunks[3], app);
    draw_todo(frame, chunks[4], app);
    draw_footer(frame, chunks[5], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let now = Local::now().format("%H:%M:%S");
    let line = Line::from(vec![
        Span::styled(
            format!("[ {} v{VERSION} ]", app.config.app.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" --- "),
        Span::styled(format!("[ {now} ]"), Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_weather(frame: &mut Frame, area: Rect, app: &App) {
    let now = Local::now();
    let date = now.format("%a %d %b %H:%M").to_string();

    if let Some(err) = app.weather.error.as_ref() {
        if app.weather.value.is_none() {
            frame.render_widget(Paragraph::new(error_line(err)), area);
            return;
        }
    }
    let Some(w) = app.weather.value.as_ref() else {
        frame.render_widget(Paragraph::new("weather: waiting…"), area);
        return;
    };
    let art = art_lines(&w.condition);
    let left = [
        date,
        w.city.clone(),
        format!("{}, {}", format_temp(&w.temp_c), w.condition),
    ];
    let lines: Vec<Line> = left
        .iter()
        .zip(art.iter())
        .map(|(l, r)| {
            let width = app.weather.width - r.len();
            Line::from(vec![
                Span::raw(format!("{l:<width$}")),
                Span::styled(*r, Style::default().fg(Color::Blue)),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_heartbeat(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![section_title("HEARTBEAT")];
    if let Some(err) = app.heartbeat.error.as_ref() {
        lines.push(error_line(err));
    }
    match app.heartbeat.value.as_ref() {
        None if app.heartbeat.error.is_none() => lines.push(Line::from("waiting…")),
        Some(rows) if rows.is_empty() => lines.push(Line::from("(none)")),
        Some(rows) => {
            for row in rows {
                let color = status_color(&row.status);
                let width = app.heartbeat.width - row.uptime_or_downtime.len() - 5;
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("[ {:>3} ]", row.status),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" {:<width$} ", truncate(&row.host, width))),
                    Span::styled(
                        format!("[ {:>8} ]", truncate(&row.uptime_or_downtime, 8)),
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
        }
        None => {}
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn hyperlink(url: &str, text: &str) -> String {
    format!(
        "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        url, text
    )
}

fn draw_news(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![section_title("NEWS")];
    if let Some(err) = app.news.error.as_ref() {
        lines.push(error_line(err));
    }
    match app.news.value.as_ref() {
        None if app.news.error.is_none() => lines.push(Line::from("waiting…")),
        Some(rows) if rows.is_empty() => lines.push(Line::from("(none)")),
        Some(rows) => {
            for row in rows {
                let width = area.width.saturating_sub(app.news.width.try_into().unwrap_or(20) - 8).max(80 - 8) as usize;
                let title = truncate(&row.title, width);
                let linked_title = hyperlink(&row.url, &title);
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("#{:<8}", truncate(&row.source, 8)),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!(" {}", linked_title),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::UNDERLINED),
                    )
                ]));
            }
        }
        None => {}
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_todo(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let mut title = vec![section_title("TODO")];
    if let Some(err) = app.todo_error.as_ref() {
        title.push(error_line(err));
    }
    frame.render_widget(Paragraph::new(title), chunks[0]);

    let items: Vec<ListItem> = if app.todos.is_empty() {
        vec![ListItem::new("(empty)")]
    } else {
        app.todos
            .iter()
            .map(|t| {
                let mark = if t.checked {
                    "x"
                } else {
                    &t.complexity.to_string()
                };
                let style = if t.checked {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };
                ListItem::new(format!("[{mark}] {}", t.title)).style(style)
            })
            .collect()
    };
    let mut state = ListState::default();
    if !app.todos.is_empty() {
        state.select(Some(app.selected));
    }
    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, chunks[1], &mut state);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    frame.render_widget(Paragraph::new("-".repeat(area.width as usize)), chunks[0]);
    let help = match &app.mode {
        Mode::Editing { adding, buffer } => {
            let label = if *adding { "New" } else { "Edit" };
            format!("{label} (n title): {buffer}_    Esc cancel")
        }
        Mode::Normal => "[a] - New; [i] - Edit; [d] - Delete; [j,k] - Up/Down; [q] - Quit".into(),
    };
    frame.render_widget(
        Paragraph::new(help).style(Style::default().fg(Color::Gray)),
        chunks[1],
    );
}

fn section_title(name: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("- [ {name} ] -----"),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ))
}

fn error_line(err: &str) -> Line<'static> {
    Line::from(Span::styled(
        truncate(err, 120),
        Style::default().fg(Color::Red),
    ))
}

fn status_color(status: &str) -> Color {
    let n: u16 = status.trim().parse().unwrap_or(0);
    if (200..300).contains(&n) {
        Color::Green
    } else if (300..400).contains(&n) {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else if max <= 12 {
        chars.into_iter().take(max).collect()
    } else {
        let mut out: String = chars.into_iter().take(max - 12).collect();
        out.push_str("...");
        out
    }
}
