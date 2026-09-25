use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

const CACHE_DIR: &str = ".cache";

pub fn dir() -> PathBuf {
    PathBuf::from(CACHE_DIR)
}

pub fn load(name: &str) -> Option<(SystemTime, String)> {
    let csv = fs::read_to_string(csv_path(name)).ok()?;
    let stamp = fs::read_to_string(meta_path(name)).ok()?;
    let millis: u64 = stamp.trim().parse().ok()?;
    let fetched_at = UNIX_EPOCH + std::time::Duration::from_millis(millis);
    Some((fetched_at, csv))
}

pub fn save(name: &str, csv: &str, fetched_at: SystemTime) -> Result<()> {
    fs::create_dir_all(dir()).context("create .cache")?;
    let millis = fetched_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    atomic_write(&csv_path(name), csv.as_bytes())?;
    atomic_write(&meta_path(name), format!("{millis}").as_bytes())?;
    Ok(())
}

fn csv_path(name: &str) -> PathBuf {
    dir().join(format!("{name}.csv"))
}

fn meta_path(name: &str) -> PathBuf {
    dir().join(format!("{name}.fetched_at"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp).with_context(|| format!("write {}", tmp.display()))?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path).with_context(|| format!("rename {}", path.display()))?;
    Ok(())
}
