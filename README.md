# Dashboard TUI app

This application shows weather, news and tasks. Refreshes by interval.
Data read from CSV lines coming from command lines configured in `config.toml`
for each section. TODO'es formatted in GitHub Flavored Markdown (GFM), that
allows to use existing files from your projects.

Unlike other apps, `config.toml` contains command lines that app executes
to get CSV input. You can integrate API requests, Bash, PowerShell commands,
Telnet or anything that outputs csv.

## Usage

- copy `config.example.toml` to `config.toml`
- build with `make build`
- run `make run` or `./target/release/dashboard`, or copy it wherever you want.

## Configuration

Configuration file separated by sections for each part of application.

```
[weather]
schema = "city,temp_c,condition"
cmd = "printf '%s\\n' 'Riga,15,Rainy'"
ttl_ms = 10800000

[heartbeat]
schema = "status,host,uptime_or_downtime"
cmd = "printf '%s\\n' '200,example.com,0d up' '404,dummy.com,15m down'"
ttl_ms = 5000

[news]
schema = "source,title,url,datetime"
cmd = "printf '%s\\n' 'The Times,People against government cuts' 'Hackernews,Windows Update fails again' 'BBC,Markets rally after rate pause' 'The Times,People against government plans' 'Ars Technica,Rust 1.81 ships' 'Wired,Windows update fails in enterprises'"
ttl_ms = 14400000
max_articles = 5
# Skip a title if Levenshtein distance to an already kept title is below this.
levenshtein_min_distance = 12

[todo]
# path to Markdown formatted todo list `- [ ] (1|3|5|8|13) Task name`
path = "TODO.md"
```

See [config.example.toml](config.example.toml) for more information.
