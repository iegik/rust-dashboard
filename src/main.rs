mod app;
mod cache;
mod config;
mod fetch;
mod models;
mod news;
mod todo;
mod ui;
mod weather;

use std::io::{self, stdout};
use std::path::Path;

use anyhow::Context;
use app::App;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

fn main() -> anyhow::Result<()> {
    let config_path = config_path();
    let cfg = config::Config::load(config_path)
        .with_context(|| format!("load {}", config_path.display()))?;
    let mut app = App::new(cfg)?;

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run(&mut terminal, &mut app);
    restore_terminal()?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if let Some(event) = app::next_event(app.poll_timeout())? {
            app.handle_event(event);
        }
        app.tick();
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn config_path() -> &'static Path {
    Path::new("config.toml")
}
