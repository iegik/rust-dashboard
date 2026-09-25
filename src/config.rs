use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub weather: SourceConfig,
    pub heartbeat: SourceConfig,
    pub news: NewsConfig,
    pub todo: TodoConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub ui_tick_ms: u64,
    pub command_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceConfig {
    pub schema: String,
    pub cmd: String,
    pub ttl_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsConfig {
    pub schema: String,
    pub cmd: String,
    pub ttl_ms: u64,
    pub max_articles: usize,
    pub levenshtein_min_distance: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TodoConfig {
    pub path: PathBuf,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: Self = toml::from_str(&raw).context("invalid config.toml")?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        check_schema(
            "weather",
            &self.weather.schema,
            &["city", "temp_c", "condition"],
        )?;
        check_schema(
            "heartbeat",
            &self.heartbeat.schema,
            &["status", "host", "uptime_or_downtime"],
        )?;
        check_schema("news", &self.news.schema, &["source", "title", "url", "datetime"])?;
        if self.app.ui_tick_ms == 0 {
            anyhow::bail!("app.ui_tick_ms must be > 0");
        }
        if self.news.max_articles == 0 {
            anyhow::bail!("news.max_articles must be > 0");
        }
        Ok(())
    }
}

fn check_schema(section: &str, actual: &str, expected: &[&str]) -> Result<()> {
    let cols: Vec<&str> = actual
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if cols != expected {
        anyhow::bail!("{section}.schema must be {}", expected.join(","));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_schema;

    #[test]
    fn rejects_wrong_schema() {
        assert!(check_schema("news", "title,source,datetime,url", &["source", "title", "url", "datetime"]).is_err());
    }
}
