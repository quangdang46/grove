# Built with Grove: session_activity_watcher

- Repository: [quangdang46/session_activity_watcher](https://github.com/quangdang46/session_activity_watcher/tree/built-with-grove)

## Overview

`session_activity_watcher` is a Grove-built tool focused on session analytics.

The point is not the internal CLI shape. The point is that Grove leaves structured local artifacts in `.grove/`, and this project turns those artifacts into something useful: session counts, live status, and early signals that a run is unhealthy.

## What Grove data looks like

From the current `.grove/` snapshot in this project:

- `grove.db` stores local Grove run state
- `transcripts/` currently contains **50** agent transcript directories
- `transcripts/*/*.jsonl` currently contains **68** session transcript files
- `prompts/*.json` currently contains **68** saved prompt snapshots

That is the real Grove story here: local session data becomes measurable and analyzable instead of opaque.

## One-command workflow

```bash
cargo run -- watch --dir "$PWD"
```

With one command, the tool watches the current repo and analyzes session activity in real time.

It helps answer practical questions such as:

- how many sessions exist for the repo
- which sessions are most recent
- whether a session is healthy, stuck, looping, or dead
- when a run is drifting out of scope or repeatedly failing

## Why it fits here

This belongs in `built-with-grove` because it is clearly Grove-centered:

- it works from Grove-generated local session artifacts
- it gives session-level visibility instead of raw logs only
- it makes Claude Code activity easier to inspect, count, and reason about
- it shortens the feedback loop when working with agents
