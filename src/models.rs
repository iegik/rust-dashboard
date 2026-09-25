use csv::ReaderBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weather {
    pub city: String,
    pub temp_c: String,
    pub condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heartbeat {
    pub status: String,
    pub host: String,
    pub uptime_or_downtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsItem {
    pub source: String,
    pub title: String,
    pub url: String,
    pub datetime: String
}

fn rows(csv: &str) -> Result<Vec<Vec<String>>, String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv.as_bytes());
    let mut out = Vec::new();
    for rec in reader.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        let cols: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        if cols.iter().all(|c| c.is_empty()) {
            continue;
        }
        out.push(cols);
    }
    Ok(out)
}

fn col(row: &[String], idx: usize) -> String {
    row.get(idx).cloned().unwrap_or_default()
}

pub fn parse_weather(csv: &str) -> Result<Option<Weather>, String> {
    let row = rows(csv)?.into_iter().next();
    Ok(row.map(|r| Weather {
        city: col(&r, 0),
        temp_c: col(&r, 1),
        condition: col(&r, 2),
    }))
}

pub fn parse_heartbeat(csv: &str) -> Result<Vec<Heartbeat>, String> {
    rows(csv).map(|rows| {
        rows.into_iter()
            .map(|r| Heartbeat {
                status: col(&r, 0),
                host: col(&r, 1),
                uptime_or_downtime: col(&r, 2),
            })
            .collect()
    })
}

pub fn parse_news(csv: &str) -> Result<Vec<NewsItem>, String> {
    rows(csv).map(|rows| {
        rows.into_iter()
            .map(|r| NewsItem {
                source: col(&r, 0),
                title: col(&r, 1),
                url: col(&r, 2),
                datetime: col(&r, 3),
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_news_title() {
        let csv = "Hackernews,\"Windows Update, again\"\n";
        let items = parse_news(csv).unwrap();
        assert_eq!(items[0].title, "Windows Update, again");
    }
}
