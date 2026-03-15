# grove

**Write code while you sleep. Complete all your beads tasks with one command.**

---

## The Pain That Built This

You know the workflow.

Open terminal. Run `claude`. Paste the init prompt — the one that loads context, explains the project, tells the agent what it's working on. Wait for it to understand. Finally, the agent starts working.

Then the context limit hits.

You exit. You open a new session. You paste the init prompt again. Load context again. Wait again. Resume where it left off — manually, because nothing remembers.

You can't leave. You can't do other work. You sit there, watching, waiting for the next context limit so you can loop it again. The agent does the work. But you're the one who can't walk away.

```
open claude
paste init prompt
load context
<agent works>
context limit hit
exit
open claude again
paste init prompt again
...
...repeat until all beads done
...or until you give up for the night
```

You become the orchestrator. A human one. Manually chaining sessions, one at a time, unable to stop because the moment you step away the work stops too.

**Grove closes this loop.**

Assign your beads to the right agents. Type `grove run`. Walk away. Come back when it's done — all tasks completed, all sessions handled, context rotations managed, memory passed between nodes automatically.

Your project is ready for the next plan. Or the bugs are fixed. And you were asleep.

---

## How It Works

Grove runs a continuous autonomous loop over your beads task graph. Each bead is a Claude session. When context exhausts, grove checkpoints and spawns a fresh session automatically. Child nodes inherit memory from parents. Parallel nodes run concurrently.

### The Loop

```
grove run
  │
  ├─ poll br ready --json
  │     → [node_A, node_B]  (no blockers, both ready)
  │
  ├─ spawn parallel sessions
  │     session A: claude -p "<task A + parent handoffs + cass memory + cm rules>"
  │     session B: claude -p "<task B + parent handoffs + cass memory + cm rules>"
  │
  ├─ session A outputs GROVE_RESULT: done
  │     → write handoff_A.json
  │     → cass index session A
  │     → cm store lesson
  │     → br close node_A
  │     → node_C (depends on A) becomes ready
  │     → grove spawns session C
  │
  ├─ session B hits context limit
  │     → GROVE_CHECKPOINT: {"progress": "60% done", "next": "finish auth"}
  │     → grove spawns new session B'
  │     → B' resumes from checkpoint with full memory injected
  │
  └─ loop until all beads closed
```

### Intelligent Exit Detection (from ralph)

Grove does not exit just because Claude says it's done. It uses a **dual-condition check**:

**Exit requires BOTH:**
1. `completion_indicators >= 2` — heuristic from natural language patterns in output
2. Claude's explicit `GROVE_EXIT: true` in the status block

```
Node loop 5: "Phase complete, moving to next feature"
  → completion_indicators: 3 (high from patterns)
  → GROVE_EXIT: false (Claude says more work needed)
  → Result: CONTINUE

Node loop 8: "All tasks complete"
  → completion_indicators: 4
  → GROVE_EXIT: true
  → Result: mark done, trigger children
```

This prevents premature exits during productive iterations — a real problem ralph solved and grove adopts directly.

### Circuit Breaker (from ralph)

Grove monitors each node session for stuck loops:

```
No file changes for 3 loops    → circuit OPEN, checkpoint, retry with new session
Same error repeated 5 loops    → circuit OPEN, fail node, notify user
Output declining 70%+          → circuit OPEN, investigate
```

Auto-recovery: OPEN → cooldown (30min) → HALF_OPEN → CLOSED.

### Context Exhaustion (grove's core addition)

Ralph loops within one session. Grove goes further — when context exhausts, grove spawns a **brand new session** with full memory reconstructed:

```
session running...
  estimated tokens > 80%?
    → node outputs GROVE_CHECKPOINT: {progress, next_step, context}
    → grove kills session gracefully
    → spawns new session with checkpoint + parent handoffs + cass search + cm rules
    → node resumes mid-task in fresh context window

stop_reason = max_tokens (exit code indicates context cut)?
    → emergency checkpoint from last known state
    → new session spawned immediately
```

### Memory (cass + cm)

```
Node A session ends
  → cass indexes the session (incremental, automatic)
  → grove writes handoff_A.json
  → cm stores key lessons from node A's work

Node B (child of A) spawns
  → cass search: "auth middleware" → returns relevant snippets from A's session
  → cm recall: returns rules learned in past sessions
  → handoff_A.json injected into prompt
  → Node B starts knowing exactly what A did
```

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/quangdang46/grove/main/install.sh | bash
```

Installs grove + cass + cm + bv automatically.

**Prerequisites (install manually first):**
- `claude` CLI — https://claude.ai/code
- `br` (beads_rust) — `cargo install --git https://github.com/Dicklesworthstone/beads_rust`

---

## Quick Start

```bash
# Init beads in your project
cd my-project
br init

# Create tasks
br create "Set up database schema" --type task
# → bd-e9b1d4

br create "Implement auth middleware" --type task
# → bd-7f3a2c

br dep add bd-7f3a2c bd-e9b1d4   # auth depends on schema

# Run grove — then go do something else
grove run
```

Grove handles everything from here. When it's done, all your beads are closed and the project is ready for the next plan.

---

## Usage

```bash
# Start orchestrator
grove run

# Limit parallel sessions (default: 5)
grove run --max-parallel 3

# Use specific model
grove run --model opus

# Start with web UI
grove run --web

# Check status
grove status

# Live TUI dashboard
grove tui

# Stream logs for a specific node
grove log bd-abc123

# Visualize the beads DAG
grove tree

# Retry a failed node
grove retry bd-abc123

# Web UI only (orchestrator already running)
grove web --port 3030
```

---

## Node Protocol

Grove communicates with Claude sessions through stdout markers. The session outputs structured signals, grove reads and acts on them.

**Task complete:**
```
GROVE_RESULT: Implemented JWT auth middleware with refresh token support
GROVE_ARTIFACTS: src/middleware/auth.rs, tests/auth_test.rs
GROVE_LESSONS: Always validate token expiry before checking signature
GROVE_EXIT: true
```

**Checkpoint (context filling up):**
```
GROVE_CHECKPOINT: {"progress": "routes done, middleware 60%", "next_step": "finish token refresh", "context": {}}
```

**Still working (prevent premature exit):**
```
GROVE_EXIT: false
```

Grove injects full context into each session's prompt:

```
[GROVE NODE]
ID: bd-7f3a2c
Task: Implement auth middleware
Priority: P1
Parents done: bd-e9b1d4

[PARENT OUTPUTS]
bd-e9b1d4 — Set up database schema
  Result: Schema done with users, sessions tables
  Artifacts: migrations/001_init.sql, src/db/schema.rs

[RELEVANT PAST SESSIONS]
Score 0.92: Previously solved JWT refresh token edge case...

[AGENT MEMORY]
Rule: Always check token expiry before debugging auth
Rule: Use connection pooling for DB-heavy middleware

[TASK]
Implement auth middleware that validates JWT tokens...

[GROVE PROTOCOL]
GROVE_RESULT: <summary>
GROVE_ARTIFACTS: <files>
GROVE_LESSONS: <lesson>
GROVE_EXIT: true | false

If context filling up:
GROVE_CHECKPOINT: {"progress": "...", "next_step": "...", "context": {}}
```

---

## Config

```toml
# grove.toml

[orchestrator]
max_parallel = 5            # concurrent Claude sessions
context_threshold = 80      # % estimated tokens → checkpoint warning
poll_interval_secs = 5      # br ready poll interval
retry_max = 3               # retries per node
retry_backoff_secs = 30

# Circuit breaker (from ralph pattern)
cb_no_progress_threshold = 3    # open circuit after N loops with no file changes
cb_same_error_threshold = 5     # open circuit after N loops with same error
cb_cooldown_minutes = 30        # auto-recovery cooldown

[session]
claude_bin = "claude"
default_model = "sonnet"        # sonnet | opus | haiku
timeout_minutes = 60            # max per session (not per node — node spans multiple sessions)

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

## Project Structure

```
my-project/
├── .beads/
│   ├── beads.jsonl
│   └── .br_history/
├── .grove/
│   ├── handoffs/           # handoff_<node_id>.json — output of each completed node
│   ├── locks/              # advisory locks for parallel file writes
│   ├── events.jsonl        # full audit log of all node events
│   └── state.json          # orchestrator checkpoint for crash recovery
└── grove.toml
```

---

## Web UI

```bash
grove run --web
# → http://127.0.0.1:3030
```

- **DAG visualization** — nodes colored by state: gray=pending, blue=running, green=done, red=failed
- **Node detail** — task description, live log stream, handoff output, config
- **Live updates** — SSE, no refresh needed
- **Config editor** — edit grove.toml fields without restarting

---

## Dependencies

All required. Grove exits with clear install instructions if any are missing.

| Tool | Purpose |
|------|---------|
| `claude` CLI | Run Claude sessions |
| `br` (beads_rust) | Task graph — frozen stable API |
| `bv` (beads_viewer) | DAG analytics, parallel track detection |
| `cass` | Cross-session memory search |
| `cm` | Agent memory rules and lessons |

---

## Roadmap

- [ ] Phase 1 — Sequential DAG, checkpoint/resume, cass/cm memory, circuit breaker
- [ ] Phase 2 — Parallel nodes, lock coordination, ratatui TUI
- [ ] Phase 3 — Web UI (DAG viz, live logs, config editor)

---

## Related

- [ralph-claude-code](https://github.com/frankbria/ralph-claude-code) — autonomous loop with intelligent exit detection (inspiration for grove's exit gate + circuit breaker)
- [beads_rust](https://github.com/Dicklesworthstone/beads_rust) — task graph CLI
- [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) — DAG analytics
- [coding_agent_session_search](https://github.com/Dicklesworthstone/coding_agent_session_search) — cass
- [cass_memory_system](https://github.com/Dicklesworthstone/cass_memory_system) — cm
- [ntm](https://github.com/Dicklesworthstone/ntm) — tmux agent manager
- [ccswarm](https://github.com/nwiizo/ccswarm) — Rust multi-agent

---

## License

MIT
