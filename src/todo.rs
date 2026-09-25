use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub checked: bool,
    pub complexity: u32,
    pub title: String,
}

pub fn load(path: impl AsRef<Path>) -> Result<Vec<TodoItem>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut items = parse(&raw);
    items.sort_by_key(|t| t.complexity);
    Ok(items)
}

pub fn save(path: impl AsRef<Path>, items: &[TodoItem]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|t| t.complexity);
    let mut buf = String::new();
    for item in &sorted {
        let mark = if item.checked { "x" } else { " " };
        buf.push_str(&format!(
            "- [{mark}] ({}) {}\n",
            item.complexity, item.title
        ));
    }
    let tmp = path.with_extension("md.tmp");
    {
        let mut f = fs::File::create(&tmp).with_context(|| format!("write {}", tmp.display()))?;
        f.write_all(buf.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn parse(raw: &str) -> Vec<TodoItem> {
    raw.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<TodoItem> {
    let line = line.trim();
    let rest = line.strip_prefix("- [")?;
    let (mark, rest) = rest.split_once(']')?;
    let checked = match mark.trim() {
        "" => false,
        "x" | "X" => true,
        _ => return None,
    };
    let rest = rest.trim();
    let rest = rest.strip_prefix('(')?;
    let (n, rest) = rest.split_once(')')?;
    let complexity = n.trim().parse().ok()?;
    let title = rest.trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some(TodoItem {
        checked,
        complexity,
        title,
    })
}

pub fn parse_input(input: &str, fallback_complexity: u32) -> Option<TodoItem> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let (complexity, title) = match input.split_once(char::is_whitespace) {
        Some((n, title)) if n.chars().all(|c| c.is_ascii_digit()) => {
            (n.parse().ok()?, title.trim().to_string())
        }
        _ => (fallback_complexity, input.to_string()),
    };
    if title.is_empty() {
        return None;
    }
    Some(TodoItem {
        checked: false,
        complexity,
        title,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_tasks() {
        let items = parse("- [ ] (1) Fix button color\n- [x] (2) Fix critical bug\n");
        assert_eq!(items.len(), 2);
        assert!(!items[0].checked);
        assert!(items[1].checked);
        assert_eq!(items[1].complexity, 2);
    }

    #[test]
    fn input_defaults_complexity() {
        let item = parse_input("just a title", 1).unwrap();
        assert_eq!(item.complexity, 1);
        let item = parse_input("3 harder task", 1).unwrap();
        assert_eq!(item.complexity, 3);
        assert_eq!(item.title, "harder task");
    }
}
