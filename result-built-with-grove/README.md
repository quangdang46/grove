# Built with Grove: session_activity_watcher

- Repository: [quangdang46/session_activity_watcher](https://github.com/quangdang46/session_activity_watcher/tree/built-with-grove)
- Local source inspected: `~/projects/tools/session_activity_watcher`

## Overview

`session_activity_watcher` ships a Rust CLI named `saw` that watches Claude Code session activity and surfaces stuck states before they waste time.

It monitors Claude session JSONL logs, hook events, and process metrics to detect:

- API hangs
- tool loops
- repeated failing test loops
- scope leaks outside a guarded subtree
- context resets after compaction
- dead sessions

## What it includes

From the local project metadata, this is a Rust workspace centered on the `saw` binary with these crates:

- `saw` — the main CLI
- `saw-core` — shared domain types and serialization helpers
- `saw-daemon` — background monitoring/runtime pieces

## Main commands

The project README documents these primary commands:

- `saw watch` — continuously monitor the active Claude Code session
- `saw status` — print one status snapshot and exit
- `saw hook` — normalize Claude hook payloads into JSONL for live monitoring
- `saw tui` — open a terminal dashboard for live status and alerts

## Build and run

```bash
cargo build --release --bin saw
./target/release/saw --help
```

Quick example:

```bash
cargo run -- watch --dir "$PWD"
```

## Tech notes

The local workspace uses Rust 2021 and includes libraries such as `clap`, `ratatui`, `crossterm`, `tokio`, `rusqlite`, `notify`, `sysinfo`, `serde`, and `chrono`.

## Why this belongs here

This project is a concrete example of something built around the Grove/Claude Code workflow: monitoring Claude Code sessions, detecting failure modes early, and surfacing live operational state in both CLI and TUI forms.
