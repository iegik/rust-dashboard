use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::cache;
use crate::config::Config;
use crate::fetch;
use crate::models::{parse_heartbeat, parse_news, parse_weather, Heartbeat, NewsItem, Weather};
use crate::news;
use crate::todo::{self, TodoItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceId {
    Weather,
    Heartbeat,
    News,
}

impl SourceId {
    fn cache_name(self) -> &'static str {
        match self {
            Self::Weather => "weather",
            Self::Heartbeat => "heartbeat",
            Self::News => "news",
        }
    }
}

struct FetchJob {
    id: SourceId,
    cmd: String,
    timeout: Duration,
}

struct FetchResult {
    id: SourceId,
    outcome: Result<String, String>,
}

pub struct SourceView<T> {
    pub value: Option<T>,
    pub error: Option<String>,
    pub fetched_at: Option<SystemTime>,
    last_attempt: Option<SystemTime>,
    in_flight: bool,
    pub width: usize,
}

impl<T> Default for SourceView<T> {
    fn default() -> Self {
        Self {
            value: None,
            error: None,
            fetched_at: None,
            last_attempt: None,
            in_flight: false,
            width: 80,
        }
    }
}

pub enum Mode {
    Normal,
    Editing { adding: bool, buffer: String },
}

pub struct App {
    pub config: Config,
    pub weather: SourceView<Weather>,
    pub heartbeat: SourceView<Vec<Heartbeat>>,
    pub news: SourceView<Vec<NewsItem>>,
    pub todos: Vec<TodoItem>,
    pub todo_error: Option<String>,
    pub selected: usize,
    pub mode: Mode,
    pub should_quit: bool,
    job_tx: Sender<FetchJob>,
    result_rx: Receiver<FetchResult>,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let (job_tx, job_rx) = mpsc::channel::<FetchJob>();
        let (result_tx, result_rx) = mpsc::channel::<FetchResult>();
        thread::Builder::new()
            .name("fetch".into())
            .spawn(move || fetch_worker(job_rx, result_tx))?;

        let mut app = Self {
            weather: SourceView::default(),
            heartbeat: SourceView::default(),
            news: SourceView::default(),
            todos: Vec::new(),
            todo_error: None,
            selected: 0,
            mode: Mode::Normal,
            should_quit: false,
            job_tx,
            result_rx,
            config,
        };
        app.hydrate_cache(SourceId::Weather);
        app.hydrate_cache(SourceId::Heartbeat);
        app.hydrate_cache(SourceId::News);
        app.reload_todos();
        app.kick_stale();
        Ok(app)
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.result_rx.try_recv() {
            self.apply_result(result);
        }
        self.kick_stale();
    }

    pub fn handle_event(&mut self, event: Event) {
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match &mut self.mode {
            Mode::Editing { adding, buffer } => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    let adding = *adding;
                    let buffer = buffer.clone();
                    self.commit_edit(adding, &buffer);
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Char(c) => buffer.push(c),
                _ => {}
            },
            Mode::Normal => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('a') => {
                    self.mode = Mode::Editing {
                        adding: true,
                        buffer: String::new(),
                    };
                }
                KeyCode::Char('i') => {
                    if let Some(item) = self.todos.get(self.selected) {
                        self.mode = Mode::Editing {
                            adding: false,
                            buffer: format!("{} {}", item.complexity, item.title),
                        };
                    }
                }
                KeyCode::Char('d') => self.delete_selected(),
                KeyCode::Char('j') | KeyCode::Down => self.move_sel(1),
                KeyCode::Char('k') | KeyCode::Up => self.move_sel(-1),
                _ => {}
            },
        }
    }

    pub fn poll_timeout(&self) -> Duration {
        Duration::from_millis(self.config.app.ui_tick_ms.max(16))
    }

    fn move_sel(&mut self, delta: i32) {
        if self.todos.is_empty() {
            self.selected = 0;
            return;
        }
        let len = self.todos.len() as i32;
        let next = (self.selected as i32 + delta).clamp(0, len - 1);
        self.selected = next as usize;
    }

    fn delete_selected(&mut self) {
        if self.selected < self.todos.len() {
            self.todos.remove(self.selected);
            if self.selected >= self.todos.len() {
                self.selected = self.todos.len().saturating_sub(1);
            }
            self.persist_todos();
        }
    }

    fn commit_edit(&mut self, adding: bool, buffer: &str) {
        let fallback = if adding {
            1
        } else {
            self.todos
                .get(self.selected)
                .map(|t| t.complexity)
                .unwrap_or(1)
        };
        let Some(mut item) = todo::parse_input(buffer, fallback) else {
            return;
        };
        if adding {
            self.todos.push(item);
        } else if let Some(existing) = self.todos.get_mut(self.selected) {
            item.checked = existing.checked;
            *existing = item;
        }
        self.todos.sort_by_key(|t| t.complexity);
        self.persist_todos();
    }

    fn persist_todos(&mut self) {
        match todo::save(&self.config.todo.path, &self.todos) {
            Ok(()) => {
                self.todo_error = None;
                self.reload_todos();
            }
            Err(e) => self.todo_error = Some(e.to_string()),
        }
    }

    fn reload_todos(&mut self) {
        match todo::load(&self.config.todo.path) {
            Ok(items) => {
                self.todos = items;
                self.todo_error = None;
                if self.selected >= self.todos.len() {
                    self.selected = self.todos.len().saturating_sub(1);
                }
            }
            Err(e) => self.todo_error = Some(e.to_string()),
        }
    }

    fn hydrate_cache(&mut self, id: SourceId) {
        let Some((fetched_at, csv)) = cache::load(id.cache_name()) else {
            return;
        };
        self.apply_csv(id, csv, fetched_at, None);
    }

    fn kick_stale(&mut self) {
        let timeout = Duration::from_millis(self.config.app.command_timeout_ms);
        let weather = (
            self.config.weather.cmd.clone(),
            Duration::from_millis(self.config.weather.ttl_ms),
        );
        let heartbeat = (
            self.config.heartbeat.cmd.clone(),
            Duration::from_millis(self.config.heartbeat.ttl_ms),
        );
        let news = (
            self.config.news.cmd.clone(),
            Duration::from_millis(self.config.news.ttl_ms),
        );
        self.maybe_fetch(SourceId::Weather, weather.0, weather.1, timeout);
        self.maybe_fetch(SourceId::Heartbeat, heartbeat.0, heartbeat.1, timeout);
        self.maybe_fetch(SourceId::News, news.0, news.1, timeout);
    }

    fn maybe_fetch(&mut self, id: SourceId, cmd: String, ttl: Duration, timeout: Duration) {
        let inflight = match id {
            SourceId::Weather => self.weather.in_flight,
            SourceId::Heartbeat => self.heartbeat.in_flight,
            SourceId::News => self.news.in_flight,
        };
        if inflight {
            return;
        }
        let (fetched_at, last_attempt) = match id {
            SourceId::Weather => (self.weather.fetched_at, self.weather.last_attempt),
            SourceId::Heartbeat => (self.heartbeat.fetched_at, self.heartbeat.last_attempt),
            SourceId::News => (self.news.fetched_at, self.news.last_attempt),
        };
        let now = SystemTime::now();
        if let Some(at) = fetched_at {
            if now.duration_since(at).map(|d| d < ttl).unwrap_or(false) {
                return;
            }
        }
        if let Some(at) = last_attempt {
            let backoff = ttl.min(Duration::from_secs(30));
            if now.duration_since(at).map(|d| d < backoff).unwrap_or(false) {
                return;
            }
        }
        match id {
            SourceId::Weather => {
                self.weather.in_flight = true;
                self.weather.last_attempt = Some(now);
            }
            SourceId::Heartbeat => {
                self.heartbeat.in_flight = true;
                self.heartbeat.last_attempt = Some(now);
            }
            SourceId::News => {
                self.news.in_flight = true;
                self.news.last_attempt = Some(now);
            }
        }
        let _ = self.job_tx.send(FetchJob { id, cmd, timeout });
    }

    fn apply_result(&mut self, result: FetchResult) {
        match result.id {
            SourceId::Weather => self.weather.in_flight = false,
            SourceId::Heartbeat => self.heartbeat.in_flight = false,
            SourceId::News => self.news.in_flight = false,
        }
        match result.outcome {
            Ok(csv) => {
                let now = SystemTime::now();
                if let Err(e) = cache::save(result.id.cache_name(), &csv, now) {
                    self.apply_csv(result.id, csv, now, Some(format!("cache: {e}")));
                } else {
                    self.apply_csv(result.id, csv, now, None);
                }
            }
            Err(e) => self.set_error(result.id, e),
        }
    }

    fn set_error(&mut self, id: SourceId, error: String) {
        match id {
            SourceId::Weather => self.weather.error = Some(error),
            SourceId::Heartbeat => self.heartbeat.error = Some(error),
            SourceId::News => self.news.error = Some(error),
        }
    }

    fn apply_csv(
        &mut self,
        id: SourceId,
        csv: String,
        fetched_at: SystemTime,
        cache_warn: Option<String>,
    ) {
        match id {
            SourceId::Weather => match parse_weather(&csv) {
                Ok(v) => {
                    self.weather.value = v;
                    self.weather.fetched_at = Some(fetched_at);
                    self.weather.error = cache_warn;
                }
                Err(e) => self.weather.error = Some(e),
            },
            SourceId::Heartbeat => match parse_heartbeat(&csv) {
                Ok(v) => {
                    self.heartbeat.value = Some(v);
                    self.heartbeat.fetched_at = Some(fetched_at);
                    self.heartbeat.error = cache_warn;
                }
                Err(e) => self.heartbeat.error = Some(e),
            },
            SourceId::News => match parse_news(&csv) {
                Ok(v) => {
                    let v = news::pick_distinct(
                        v,
                        self.config.news.max_articles,
                        self.config.news.levenshtein_min_distance,
                    );
                    self.news.value = Some(v);
                    self.news.fetched_at = Some(fetched_at);
                    self.news.error = cache_warn;
                }
                Err(e) => self.news.error = Some(e),
            },
        }
    }
}

fn fetch_worker(jobs: Receiver<FetchJob>, results: Sender<FetchResult>) {
    while let Ok(job) = jobs.recv() {
        let outcome = fetch::run_command(&job.cmd, job.timeout);
        if results
            .send(FetchResult {
                id: job.id,
                outcome,
            })
            .is_err()
        {
            break;
        }
    }
}

pub fn next_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
