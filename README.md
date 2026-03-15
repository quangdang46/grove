# grove

**Write code while you sleep. Complete all your beads tasks with one command.**

---

## The Pain That Built This

You know the workflow.

Open terminal. Run `claude`. Paste the init prompt — the one that loads context, explains the project, tells the agent what it's working on. Wait for it to understand. Finally, the agent starts working.

Then the context limit hits.

You exit. You open a new session. You paste the init prompt again. Load context again. Wait again. Resume where it left off — manually, because nothing remembers.

You can't leave. You can't do other work. You sit there, watching, waiting for the next context limit so you can loop it again. The agent does the work. But you're the one who can't walk away.

This is the loop:

```
open claude
paste init prompt
load context
<agent works>
context limit hit
exit
repeat
...until all beads done
...or until you give up for the night
```

You become the orchestrator. A human one. Manually chaining sessions, one at a time, unable to stop because the moment you step away the work stops too.

**Grove closes this loop.**

Assign your beads to the right agents. Type `grove run`. Walk away. Come back when it's done — all tasks completed, all sessions handled, context rotations managed, memory passed between nodes automatically.

Your project is ready for the next plan. Or you fixed the bugs while you slept.

---

## How It Works

```
br ready --json → [node_A, node_B]  (parallel, no deps)
                        │
         ┌──────────────┴──────────────┐
         ▼                             ▼
  claude session A               claude session B
  + parent handoffs              + parent handoffs
  + cass memory                  + cass memory
  + cm rules                     + cm rules
         │                             │
  GROVE_RESULT: done             context > 80%?
  write handoff_A.json           → GROVE_CHECKPOINT: {...}
  br close node_A                → new session spawns
  → node_C becomes ready         → resumes from checkpoint
```

### Context Exhaustion

```
Session running...
  estimated tokens > 80%?
    → grove warns node: "context filling up"
    → node writes GROVE_CHECKPOINT: {...}
    → grove spawns new session with checkpoint injected
    → node resumes mid-task

stop_reason = max_tokens?
    → grove detects from exit
    → emergency new session with last checkpoint
```

### Memory (cass + cm)

```
Node A done
  → cass indexes the session (incremental)
  → grove writes handoff_A.json
  → cm stores lessons from the session

Node B (child of A) starts
  → cass search for relevant past sessions
  → cm recall for rules/lessons
  → parent handoff injected into prompt
  → Node B starts with full context
```

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/quangdang46/grove/main/install.sh | bash
```

Installs grove + cass + cm + bv automatically.

**You need to install first:**
- `claude` CLI (Claude Code): https://claude.ai/code
- `br` (beads_rust): `cargo install --git https://github.com/Dicklesworthstone/beads_rust`

---

## Quick Start

```bash
# Init beads in your project
cd my-project
br init

# Create tasks with dependencies
br create "Set up database schema" --type task --priority 1
# → bd-e9b1d4

br create "Implement auth middleware" --type task --priority 1
# → bd-7f3a2c

br dep add bd-7f3a2c bd-e9b1d4  # auth depends on schema

# Run grove
grove run
```

Grove polls `br ready`, spawns sessions, handles context rotation, and triggers child nodes automatically.

---

## Usage

```bash
# Start orchestrator
grove run

# Limit parallel sessions (default: 5)
grove run --max-parallel 3

# Start with web UI
grove run --web

# Specific model
grove run --model opus

# Status overview
grove status

# Live TUI dashboard
grove tui

# Stream logs for a node
grove log bd-abc123

# Visualize DAG
grove tree

# Retry failed node
grove retry bd-abc123

# Start web UI only (orchestrator already running)
grove web --port 3030
```

---

## Node Protocol

Claude sessions communicate with grove via stdout markers:

```
# Task complete
GROVE_RESULT: Implemented JWT auth middleware with refresh token support
GROVE_ARTIFACTS: src/middleware/auth.rs, src/routes/auth.rs, tests/auth_test.rs
GROVE_LESSONS: Always validate token expiry before checking signature

# Checkpoint (context filling up)
GROVE_CHECKPOINT: {"progress": "routes done, middleware 60% complete", "next_step": "finish token refresh logic", "context": {}}
```

Grove injects context into each session's system prompt:

```
[GROVE NODE]
ID: bd-7f3a2c
Task: Implement auth middleware
Priority: P1
Parents done: bd-e9b1d4

[PARENT OUTPUTS]
bd-e9b1d4 — Set up database schema:
  Result: Schema implemented with users, sessions tables
  Artifacts: migrations/001_init.sql, src/db/schema.rs

[RELEVANT PAST SESSIONS]
Score 0.92: Previously solved JWT refresh token expiry edge case...

[AGENT MEMORY]
Rule: Always check token expiry before auth debugging
Rule: Use connection pooling for DB-heavy middleware

[TASK]
Implement auth middleware that validates JWT tokens...
```

---

## Config

```toml
# grove.toml

[orchestrator]
max_parallel = 5            # concurrent Claude sessions
context_threshold = 80      # % tokens → checkpoint warning
poll_interval_secs = 5      # br ready poll interval
retry_max = 3               # retries per node

[session]
claude_bin = "claude"
default_model = "sonnet"    # sonnet | opus | haiku
timeout_secs = 3600         # 1h max per node

[memory]
handoff_dir = ".grove/handoffs"
lock_dir = ".grove/locks"
cass_search_limit = 5
cass_days = 30

[beads]
br_bin = "br"
bv_bin = "bv"

[web]
port = 3030
host = "127.0.0.1"
```

---

## Web UI (Phase 3)

```bash
grove run --web
# → http://127.0.0.1:3030
```

- **DAG visualization** — nodes colored by state, click for details
- **Live log stream** — real-time output per node
- **Node detail** — task description, handoff data, config
- **Config editor** — edit grove.toml without restarting
- **SSE live updates** — no refresh needed

---

## Workspace Structure

```
my-project/
├── .beads/
│   ├── beads.jsonl         # br task data
│   └── .br_history/        # br backups
├── .grove/
│   ├── handoffs/           # handoff_<node_id>.json per completed node
│   ├── locks/              # advisory locks for parallel file writes
│   ├── events.jsonl        # audit log
│   └── state.json          # orchestrator checkpoint
└── grove.toml
```

---

## Comparison

| Feature | grove | ntm | ccswarm | agent_farm |
|---------|-------|-----|---------|------------|
| Beads task graph | ✅ | ❌ | ❌ | ❌ |
| Auto context rotate | ✅ | ✅ | partial | ✅ |
| Checkpoint/resume | ✅ | ❌ | partial | ❌ |
| cass/cm memory | ✅ | read-only | ❌ | ❌ |
| Parallel nodes | ✅ | ✅ | ✅ | ✅ |
| File lock coordination | ✅ | ❌ | ❌ | ✅ |
| Web UI | ✅ (Phase 3) | ❌ | ❌ | ❌ |
| Rust native | ✅ | ✅ (Go) | ✅ | ❌ (Python) |
| No tmux required | ✅ | ❌ | ✅ | ❌ |

---

## Dependencies

All required. Grove exits with clear install instructions if any are missing.

| Tool | Purpose | Install |
|------|---------|---------|
| `claude` CLI | Run Claude sessions | https://claude.ai/code |
| `br` (beads_rust) | Task graph | `cargo install --git github.com/Dicklesworthstone/beads_rust` |
| `bv` (beads_viewer) | DAG analytics | installed by grove installer |
| `cass` | Cross-session memory search | installed by grove installer |
| `cm` | Agent memory rules | installed by grove installer |

---

## Roadmap

- [ ] Phase 1: Sequential DAG, checkpoint/resume, cass/cm memory
- [ ] Phase 2: Parallel nodes, lock coordination, ratatui TUI
- [ ] Phase 3: Web UI (DAG viz, live logs, config editor)

---

## Related

- [beads_rust](https://github.com/Dicklesworthstone/beads_rust) — task graph CLI
- [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) — DAG analytics
- [coding_agent_session_search](https://github.com/Dicklesworthstone/coding_agent_session_search) — cass
- [cass_memory_system](https://github.com/Dicklesworthstone/cass_memory_system) — cm
- [ntm](https://github.com/Dicklesworthstone/ntm) — tmux-based agent manager (inspiration)
- [ccswarm](https://github.com/nwiizo/ccswarm) — Rust multi-agent (inspiration)

---

## License

MIT
