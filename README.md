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

See [config.example.toml](config.example.toml) for more information.

## Purpose of the project

The goal of this project is learn more about Rust for myself and show how TUI
can be transparent for integrating other apps with command lines
and use generic interface for input data.

Feel free to make changes, forks and even architecture changes.

Here is an OOP used, but it can be also FP or mixed.

[LICENSE](LICENSE)
