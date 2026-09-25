use crate::models::NewsItem;

pub fn pick_distinct(items: Vec<NewsItem>, max: usize, min_distance: usize) -> Vec<NewsItem> {
    let mut kept: Vec<NewsItem> = Vec::new();
    for item in items {
        if item.title.trim().is_empty() {
            continue;
        }
        let title = normalize(&item.title);
        let similar = kept
            .iter()
            .any(|k| levenshtein(&title, &normalize(&k.title)) < min_distance);
        if similar {
            continue;
        }
        kept.push(item);
        if kept.len() >= max {
            break;
        }
    }
    kept
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_near_duplicate_titles() {
        let items = vec![
            NewsItem {
                source: "A".into(),
                title: "People against government cuts".into(),
            },
            NewsItem {
                source: "B".into(),
                title: "People against government plans".into(),
            },
            NewsItem {
                source: "C".into(),
                title: "Rust 1.81 ships".into(),
            },
        ];
        let kept = pick_distinct(items, 5, 12);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[1].source, "C");
    }

    #[test]
    fn identical_strings_are_zero() {
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }
}
