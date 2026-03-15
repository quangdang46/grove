# PLAN.md — grove

> Rust CLI orchestrator for beads-driven multi-session Claude agent workflows.
> Each bead task = one Claude session. Context exhaustion triggers checkpoint + new session.
> Cross-session memory via cass + cm (Jeff Emanuel's ecosystem).

---

## 0. Research Findings từ Các Repo Thực Tế

### 0.1 beads_rust (`br`) — JSON format frozen, stable vĩnh viễn

Jeff tạo beads_rust để **freeze** "classic beads" architecture — Go version đang thay đổi sang GasTown, còn beads_rust được tạo ra để không thay đổi. JSON output format qua `--json`/`--robot` là stable API.

```
CRITICAL từ AGENTS.md của beads_rust:
  Always use --json or --robot flags when parsing br output programmatically.
  CORRECT: br ready --json | jq '.[0]'
  WRONG:   br list | head -1   ← output format thay đổi theo TTY state
```

**br commands grove dùng:**
```bash
br ready --json                          # nodes không bị block
br show <id> --json                      # task details
br list --json                           # tất cả tasks
br update <id> --status in_progress      # mark running
br close <id> --reason "<summary>"       # mark done
br dep add <child-id> <parent-id>        # add dependency
```

**br JSON schema (thực tế):**
```json
[
  {
    "id": "bd-abc123",
    "title": "Implement auth middleware",
    "description": "...",
    "priority": 1,
    "type": "task",
    "status": "open",
    "assignee": null,
    "labels": [],
    "blocked_by": [],
    "blocks": ["bd-def456"]
  }
]
```

### 0.2 beads_viewer (`bv`) — DAG analytics mà br không có

```bash
bv --robot-triage         # full DAG: PageRank, critical path, parallel tracks
bv --robot-plan           # topological sort + execution order
bv --robot-next           # single top-priority task
bv --robot-graph          # export graph JSON/DOT/Mermaid
```

Grove dùng `bv --robot-plan` để detect parallel tracks và optimize scheduling.

### 0.3 cass — Required (install cùng grove)

```bash
# Health check
cass health      # exit 0=ok, exit 1=cần index trước

# Grove dùng
cass search "<task keywords>" --robot --limit 5 --fields minimal --mode hybrid
cass search "<task>" --agent claude --days 30 --robot
cass index       # incremental index, gọi sau mỗi node done
```

**cass robot output:**
```json
{
  "results": [
    {
      "session_path": "/path/to/session.jsonl",
      "line_number": 42,
      "score": 0.95,
      "snippet": "...",
      "agent": "claude",
      "workspace": "myproject"
    }
  ],
  "total": 5,
  "mode": "hybrid"
}
```

**Decision: cass = required**, grove install script cài luôn.
Lý do: không có cass thì child nodes không có memory từ parent sessions — đây là core feature.

### 0.4 cm (cass_memory_system) — Required (install cùng grove)

```bash
cm onboard status     # health check
cm recall "<context>" # query relevant rules/lessons
cm store "<lesson>"   # store lesson sau khi node xong

# cm cũng có MCP HTTP server mode (Phase 3)
cm serve --port 8765
```

**Decision: cm = required**, install script cài cùng cass.

### 0.5 ntm — Học context rotation, không depend

ntm internal structure có `internal/context/` cho context monitoring và `internal/checkpoint/` cho checkpoint types. ntm estimate tokens từ **message count + output character count** (heuristic ~4 chars/token) vì Claude Code không expose token count trực tiếp. Grove dùng cùng approach.

ntm cũng có graceful degradation: optional tools thiếu thì warn, không crash. Grove học pattern này.

### 0.6 ccswarm — Học PTY + workspace structure

ccswarm dùng `portable-pty` crate để spawn AI sessions. Grove bắt đầu với `tokio::process::Command` (simpler), upgrade sang PTY nếu `claude -p` cần TTY.

---

## 1. Architecture

```
grove/
├── Cargo.toml
├── grove.toml
├── install.sh                    # cài grove + cass + cm tự động
├── PLAN.md
├── README.md
├── crates/
│   ├── grove-core/               # types, state machine, config
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── node.rs           # NodeId, NodeState, HandoffData, CheckpointData
│   │       ├── dag.rs            # DagView in-memory
│   │       └── config.rs         # GroveConfig từ grove.toml
│   │
│   ├── grove-beads/              # br + bv integration
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── br_client.rs      # wrap br CLI + JSON parsing
│   │       ├── bv_client.rs      # wrap bv --robot-* flags
│   │       └── schema.rs         # BrIssue, BvPlan types
│   │
│   ├── grove-session/            # claude session management
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── spawn.rs          # tokio::process spawn claude -p
│   │       ├── monitor.rs        # context threshold heuristic
│   │       ├── parser.rs         # parse GROVE_* markers từ stdout
│   │       └── prompt.rs         # build system prompt
│   │
│   ├── grove-memory/             # cass + cm + handoff
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── cass_client.rs    # cass search --robot
│   │       ├── cm_client.rs      # cm recall / cm store
│   │       ├── handoff_store.rs  # atomic read/write handoff JSON
│   │       └── context_builder.rs # assemble full prompt
│   │
│   ├── grove-lock/               # parallel file coordination
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lock.rs           # fs2 advisory lock
│   │       └── registry.rs       # track node → lock mapping
│   │
│   ├── grove-orchestrator/       # main event loop
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs   # poll beads → spawn nodes
│   │       ├── scheduler.rs      # dependency check, parallel tracks
│   │       ├── runner.rs         # tokio task pool + semaphore
│   │       ├── checkpoint.rs     # checkpoint/resume session loop
│   │       └── events.rs         # NodeEvent bus, events.jsonl
│   │
│   ├── grove-web/                # embedded web UI (Phase 3)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs         # axum HTTP server
│   │       ├── api.rs            # REST + SSE endpoints
│   │       └── assets/           # embedded index.html + D3.js
│   │
│   └── grove-cli/                # binary
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── run.rs        # grove run [--web]
│               ├── status.rs     # grove status
│               ├── tui.rs        # grove tui (ratatui)
│               ├── log.rs        # grove log <node-id>
│               ├── retry.rs      # grove retry <node-id>
│               ├── tree.rs       # grove tree
│               └── web.rs        # grove web [--port N]
│
└── .grove/                       # runtime state (gitignored)
    ├── handoffs/                  # handoff_<node_id>.json
    ├── locks/                     # advisory lock files
    ├── events.jsonl               # audit log
    └── state.json                 # orchestrator checkpoint
```

---

## 2. Core Types (grove-core)

```rust
// node.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);  // "bd-abc123"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeState {
    Pending,
    Ready,
    Running {
        session_id: String,
        started_at: DateTime<Utc>,
        attempt: u32,
    },
    Checkpointed {
        checkpoint: CheckpointData,
        attempt: u32,
    },
    Done {
        handoff: HandoffData,
        completed_at: DateTime<Utc>,
    },
    Failed {
        reason: String,
        attempts: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffData {
    pub node_id: NodeId,
    pub task_title: String,
    pub result_summary: String,
    pub artifacts: Vec<String>,      // files created/modified
    pub git_commits: Vec<String>,
    pub key_decisions: Vec<String>,
    pub warnings: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData {
    pub node_id: NodeId,
    pub progress: String,
    pub next_step: String,
    pub context: serde_json::Value,
    pub attempt: u32,
    pub saved_at: DateTime<Utc>,
}

// dag.rs
pub struct DagView {
    pub nodes: HashMap<NodeId, NodeMeta>,
    pub edges: Vec<(NodeId, NodeId)>,  // (blocked_by, blocks)
}

impl DagView {
    pub fn all_parents_done(&self, node_id: &NodeId, done_set: &HashSet<NodeId>) -> bool {
        self.edges.iter()
            .filter(|(_, child)| child == node_id)
            .all(|(parent, _)| done_set.contains(parent))
    }
}
```

---

## 3. br + bv Integration (grove-beads)

```rust
// br_client.rs
pub struct BrClient {
    bin: PathBuf,
    project_dir: PathBuf,
}

impl BrClient {
    async fn run_json<T: DeserializeOwned>(&self, args: &[&str]) -> Result<T> {
        let output = Command::new(&self.bin)
            .args(args)
            .arg("--json")
            .current_dir(&self.project_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("br {} failed: {}", args[0], stderr));
        }

        Ok(serde_json::from_slice(&output.stdout)?)
    }

    pub async fn ready(&self) -> Result<Vec<BrIssue>> {
        self.run_json(&["ready"]).await
    }

    pub async fn show(&self, id: &NodeId) -> Result<BrIssue> {
        self.run_json(&["show", &id.0]).await
    }

    pub async fn list_all(&self) -> Result<Vec<BrIssue>> {
        self.run_json(&["list"]).await
    }

    pub async fn mark_in_progress(&self, id: &NodeId) -> Result<()> {
        Command::new(&self.bin)
            .args(["update", &id.0, "--status", "in_progress"])
            .current_dir(&self.project_dir)
            .output()
            .await?;
        Ok(())
    }

    pub async fn close(&self, id: &NodeId, summary: &str) -> Result<()> {
        Command::new(&self.bin)
            .args(["close", &id.0, "--reason", summary])
            .current_dir(&self.project_dir)
            .output()
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrIssue {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: u8,
    pub r#type: String,
    pub status: String,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
}

// bv_client.rs
pub struct BvClient {
    bin: PathBuf,
    project_dir: PathBuf,
}

impl BvClient {
    pub async fn parallel_tracks(&self) -> Result<BvPlan>
    // bv --robot-plan --json
    // Parse parallel execution tracks

    pub async fn export_graph_json(&self) -> Result<serde_json::Value>
    // bv --robot-graph --json
    // Dùng cho web UI DAG visualization
}
```

---

## 4. Session Management (grove-session)

### 4.1 Spawn

```rust
// spawn.rs
pub struct SessionConfig {
    pub node_id: NodeId,
    pub prompt: String,
    pub model: String,
    pub attempt: u32,
}

pub struct ActiveSession {
    pub node_id: NodeId,
    pub attempt: u32,
    child: Child,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

pub async fn spawn_session(config: &SessionConfig, claude_bin: &Path) -> Result<ActiveSession> {
    let mut child = Command::new(claude_bin)
        .args(["-p", &config.prompt, "--model", &config.model])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stderr = BufReader::new(child.stderr.take().unwrap());

    Ok(ActiveSession {
        node_id: config.node_id.clone(),
        attempt: config.attempt,
        child,
        stdout,
        stderr,
    })
}
```

### 4.2 Output Parser

```rust
// parser.rs
#[derive(Debug)]
pub enum SessionOutput {
    Line(String),                   // regular output
    Result {
        summary: String,
        artifacts: Vec<String>,
        lessons: Vec<String>,
    },
    Checkpoint(CheckpointData),
    ProcessExit { code: i32, max_tokens: bool },
}

pub fn parse_line(line: &str, node_id: &NodeId) -> SessionOutput {
    // GROVE_RESULT: <summary>
    if let Some(rest) = line.strip_prefix("GROVE_RESULT:") {
        // tiếp tục đọc GROVE_ARTIFACTS: và GROVE_LESSONS: từ subsequent lines
        return SessionOutput::Line(line.to_string()); // handled by state machine
    }
    // GROVE_CHECKPOINT: <json>
    if let Some(rest) = line.strip_prefix("GROVE_CHECKPOINT:") {
        if let Ok(cp) = serde_json::from_str::<serde_json::Value>(rest.trim()) {
            return SessionOutput::Checkpoint(CheckpointData {
                node_id: node_id.clone(),
                progress: cp["progress"].as_str().unwrap_or("").to_string(),
                next_step: cp["next_step"].as_str().unwrap_or("").to_string(),
                context: cp["context"].clone(),
                attempt: 0,
                saved_at: Utc::now(),
            });
        }
    }
    SessionOutput::Line(line.to_string())
}
```

### 4.3 Context Monitor

```rust
// monitor.rs
pub struct ContextMonitor {
    estimated_tokens: u32,
    message_count: u32,
    threshold_pct: u8,
    model_limit: u32,
}

impl ContextMonitor {
    pub fn record_output(&mut self, chars: usize) {
        // ~4 chars per token (heuristic từ ntm)
        self.estimated_tokens += (chars / 4) as u32;
        self.message_count += 1;
    }

    pub fn usage_pct(&self) -> u8 {
        ((self.estimated_tokens * 100) / self.model_limit).min(100) as u8
    }

    pub fn should_warn(&self) -> bool {
        self.usage_pct() >= self.threshold_pct
    }
}

// Model limits (tokens)
pub fn model_context_limit(model: &str) -> u32 {
    match model {
        "opus" | "sonnet" | "haiku" => 200_000,
        _ => 200_000,
    }
}
```

---

## 5. Memory Integration (grove-memory)

### 5.1 cass Client

```rust
// cass_client.rs
pub struct CassClient {
    bin: PathBuf,
}

impl CassClient {
    pub async fn health(&self) -> bool {
        Command::new(&self.bin)
            .arg("health")
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub async fn search(&self, query: &str, limit: u8) -> Result<Vec<CassResult>> {
        let output = Command::new(&self.bin)
            .args(["search", query, "--robot", "--mode", "hybrid",
                   "--limit", &limit.to_string(), "--fields", "minimal"])
            .output()
            .await?;

        let response: CassResponse = serde_json::from_slice(&output.stdout)?;
        Ok(response.results)
    }

    pub async fn index_incremental(&self) -> Result<()> {
        // Gọi sau mỗi node done để cass index session mới
        Command::new(&self.bin)
            .arg("index")
            .output()
            .await?;
        Ok(())
    }

    pub async fn search_for_task(&self, issue: &BrIssue, limit: u8) -> Result<String> {
        let query = format!("{} {}", issue.title,
            issue.description.as_deref().unwrap_or(""));
        let results = self.search(&query, limit).await?;

        if results.is_empty() {
            return Ok("(no relevant past sessions found)".to_string());
        }

        Ok(results.iter()
            .map(|r| format!("Score {:.2}: {}", r.score, r.snippet))
            .collect::<Vec<_>>()
            .join("\n\n"))
    }
}

#[derive(Debug, Deserialize)]
pub struct CassResponse {
    pub results: Vec<CassResult>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct CassResult {
    pub session_path: String,
    pub line_number: u32,
    pub score: f32,
    pub snippet: String,
    pub agent: String,
}
```

### 5.2 cm Client

```rust
// cm_client.rs
pub struct CmClient {
    bin: PathBuf,
}

impl CmClient {
    pub async fn health(&self) -> bool {
        Command::new(&self.bin)
            .args(["onboard", "status"])
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub async fn recall(&self, context: &str) -> Result<String> {
        let output = Command::new(&self.bin)
            .args(["recall", context, "--json"])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok("(no relevant memories)".to_string())
        }
    }

    pub async fn store(&self, lesson: &str) -> Result<()> {
        Command::new(&self.bin)
            .args(["store", lesson])
            .output()
            .await?;
        Ok(())
    }
}
```

### 5.3 Handoff Store

```rust
// handoff_store.rs
pub struct HandoffStore {
    dir: PathBuf,
}

impl HandoffStore {
    pub async fn write(&self, handoff: &HandoffData) -> Result<()> {
        let path = self.dir.join(format!("handoff_{}.json", handoff.node_id.0));
        let tmp = path.with_extension("json.tmp");

        // Atomic write
        let json = serde_json::to_string_pretty(handoff)?;
        tokio::fs::write(&tmp, &json).await?;
        tokio::fs::rename(&tmp, &path).await?;
        Ok(())
    }

    pub async fn read(&self, node_id: &NodeId) -> Result<Option<HandoffData>> {
        let path = self.dir.join(format!("handoff_{}.json", node_id.0));
        if !path.exists() {
            return Ok(None);
        }
        let json = tokio::fs::read(&path).await?;
        Ok(Some(serde_json::from_slice(&json)?))
    }

    pub async fn read_parents(&self, parent_ids: &[NodeId]) -> Result<Vec<HandoffData>> {
        let mut handoffs = Vec::new();
        for id in parent_ids {
            if let Some(h) = self.read(id).await? {
                handoffs.push(h);
            }
        }
        Ok(handoffs)
    }
}
```

### 5.4 Context Builder

```rust
// context_builder.rs
pub struct ContextBuilder {
    cass: CassClient,
    cm: CmClient,
    handoff_store: HandoffStore,
    config: MemoryConfig,
}

impl ContextBuilder {
    pub async fn build(&self, issue: &BrIssue, checkpoint: Option<&CheckpointData>) -> Result<String> {
        // 1. Parent handoffs
        let parent_ids = issue.blocked_by.iter().map(|s| NodeId(s.clone())).collect::<Vec<_>>();
        let handoffs = self.handoff_store.read_parents(&parent_ids).await?;

        // 2. cass search
        let cass_ctx = self.cass.search_for_task(issue, self.config.cass_search_limit).await
            .unwrap_or_else(|_| "(cass unavailable)".to_string());

        // 3. cm recall
        let cm_ctx = self.cm.recall(&issue.title).await
            .unwrap_or_else(|_| "(cm unavailable)".to_string());

        // 4. Resume context nếu đây là session mới sau checkpoint
        let resume_section = checkpoint.map(|cp| format!(
            "\n[RESUME FROM CHECKPOINT]\nProgress: {}\nNext step: {}\n",
            cp.progress, cp.next_step
        )).unwrap_or_default();

        Ok(format!(
r#"[GROVE NODE]
ID: {id}
Task: {title}
Priority: P{priority}
Parents done: {parents}
{resume}
[PARENT OUTPUTS]
{handoffs}

[RELEVANT PAST SESSIONS]
{cass}

[AGENT MEMORY]
{cm}

[TASK]
{description}

[PROTOCOL]
On completion:
  GROVE_RESULT: <one-line summary>
  GROVE_ARTIFACTS: <files, or "none">
  GROVE_LESSONS: <one lesson, or "none">

If context is filling up:
  GROVE_CHECKPOINT: {{"progress": "...", "next_step": "...", "context": {{}}}}
"#,
            id = issue.id,
            title = issue.title,
            priority = issue.priority,
            parents = parent_ids.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", "),
            resume = resume_section,
            handoffs = format_handoffs(&handoffs),
            cass = cass_ctx,
            cm = cm_ctx,
            description = issue.description.as_deref().unwrap_or("(no description)"),
        ))
    }
}
```

---

## 6. Orchestrator (grove-orchestrator)

### 6.1 Main Loop

```rust
// orchestrator.rs
pub struct Orchestrator {
    config: GroveConfig,
    br: BrClient,
    bv: BvClient,
    spawner: Arc<SessionSpawner>,
    ctx_builder: Arc<ContextBuilder>,
    locks: Arc<LockRegistry>,
    node_states: Arc<RwLock<HashMap<NodeId, NodeState>>>,
    semaphore: Arc<Semaphore>,
    event_tx: broadcast::Sender<NodeEvent>,  // cho web UI SSE
}

impl Orchestrator {
    pub async fn run(&self) -> Result<()> {
        self.check_dependencies().await?;

        loop {
            // 1. Poll beads
            let ready = self.br.ready().await?;

            // 2. Filter chưa running
            let states = self.node_states.read().await;
            let to_spawn: Vec<BrIssue> = ready.into_iter()
                .filter(|n| !matches!(states.get(&NodeId(n.id.clone())),
                    Some(NodeState::Running { .. })))
                .collect();
            drop(states);

            // 3. Spawn (bounded bởi semaphore)
            for issue in to_spawn {
                if self.semaphore.available_permits() == 0 {
                    break; // đợi permit ở loop tiếp
                }
                self.spawn_node(issue).await?;
            }

            // 4. Exit nếu không còn gì để làm
            let all_issues = self.br.list_all().await?;
            let open = all_issues.iter().filter(|n| n.status != "closed").count();
            if open == 0 {
                tracing::info!("All nodes completed. grove done.");
                break;
            }

            tokio::time::sleep(Duration::from_secs(self.config.orchestrator.poll_interval_secs)).await;
        }
        Ok(())
    }

    async fn spawn_node(&self, issue: BrIssue) -> Result<()> {
        let permit = Arc::clone(&self.semaphore).acquire_owned().await?;
        let node_id = NodeId(issue.id.clone());

        // Update state
        self.node_states.write().await
            .insert(node_id.clone(), NodeState::Running {
                session_id: uuid::Uuid::new_v4().to_string(),
                started_at: Utc::now(),
                attempt: 1,
            });

        // Mark in beads
        self.br.mark_in_progress(&node_id).await?;

        // Emit event
        let _ = self.event_tx.send(NodeEvent::Started(node_id.clone()));

        // Spawn tokio task
        let orchestrator = self.clone_refs();
        tokio::spawn(async move {
            let _permit = permit;
            let result = orchestrator.run_node_to_completion(issue).await;
            orchestrator.handle_node_result(result).await;
        });

        Ok(())
    }

    async fn check_dependencies(&self) -> Result<()> {
        // br
        if !self.br.health_check().await {
            return Err(anyhow!("br (beads_rust) not found. Install: cargo install --git https://github.com/Dicklesworthstone/beads_rust"));
        }
        // cass
        if !self.ctx_builder.cass.health().await {
            tracing::warn!("cass not ready. Run 'cass index' first. Memory will be degraded.");
        }
        // cm
        if !self.ctx_builder.cm.health().await {
            tracing::warn!("cm not responding. Memory rules will be unavailable.");
        }
        Ok(())
    }
}
```

### 6.2 Node Execution + Checkpoint/Resume

```rust
// checkpoint.rs
impl Orchestrator {
    async fn run_node_to_completion(&self, issue: BrIssue) -> Result<NodeResult> {
        let mut attempt = 0u32;
        let mut checkpoint: Option<CheckpointData> = None;

        loop {
            attempt += 1;
            if attempt > self.config.orchestrator.retry_max {
                return Err(anyhow!("Node {} exceeded max retries", issue.id));
            }

            // Build prompt (với checkpoint nếu là resume)
            let prompt = self.ctx_builder.build(&issue, checkpoint.as_ref()).await?;

            // Spawn session
            let mut session = spawn_session(&SessionConfig {
                node_id: NodeId(issue.id.clone()),
                prompt,
                model: self.config.session.default_model.clone(),
                attempt,
            }, &self.config.session.claude_bin).await?;

            // Monitor + read output
            let mut monitor = ContextMonitor::new(
                self.config.orchestrator.context_threshold,
                model_context_limit(&self.config.session.default_model),
            );

            let mut result_summary = None;
            let mut artifacts = Vec::new();
            let mut lessons = Vec::new();

            loop {
                tokio::select! {
                    line = session.next_line() => {
                        match line? {
                            None => {
                                // Process ended
                                let exit = session.wait().await?;
                                if exit.success() {
                                    if let Some(summary) = result_summary {
                                        return Ok(NodeResult::Done(HandoffData {
                                            node_id: NodeId(issue.id.clone()),
                                            task_title: issue.title.clone(),
                                            result_summary: summary,
                                            artifacts,
                                            git_commits: vec![],
                                            key_decisions: vec![],
                                            warnings: vec![],
                                            completed_at: Utc::now(),
                                        }));
                                    }
                                } else {
                                    // Check if max_tokens via stderr
                                    let stderr = session.stderr_str();
                                    if stderr.contains("max_tokens") || stderr.contains("context_length") {
                                        // Emergency resume
                                        checkpoint = Some(CheckpointData::emergency(
                                            &NodeId(issue.id.clone()), attempt
                                        ));
                                        break; // outer loop sẽ retry
                                    }
                                    return Err(anyhow!("Session failed: {}", stderr));
                                }
                            }
                            Some(output) => {
                                monitor.record_output(output.chars.len());
                                match output.kind {
                                    OutputKind::Result { summary, arts, less } => {
                                        result_summary = Some(summary);
                                        artifacts = arts;
                                        lessons = less;
                                    }
                                    OutputKind::Checkpoint(cp) => {
                                        checkpoint = Some(cp);
                                        // Kill session gracefully
                                        session.kill().await?;
                                        break; // outer loop retry
                                    }
                                    OutputKind::Line(line) => {
                                        // Stream to event bus cho web UI / grove log
                                        let _ = self.event_tx.send(NodeEvent::LogLine {
                                            node_id: NodeId(issue.id.clone()),
                                            line,
                                        });

                                        // Check context threshold
                                        if monitor.should_warn() && checkpoint.is_none() {
                                            tracing::warn!(
                                                "Node {} at {}% context. Watching for GROVE_CHECKPOINT.",
                                                issue.id, monitor.usage_pct()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(self.config.session.timeout_secs)) => {
                        session.kill().await?;
                        return Err(anyhow!("Node {} timed out", issue.id));
                    }
                }
            }

            // Nếu ra khỏi inner loop mà không return → checkpoint, retry
            tracing::info!("Node {} checkpointed at attempt {}. Spawning new session.", issue.id, attempt);
            tokio::time::sleep(Duration::from_secs(self.config.orchestrator.retry_backoff_secs)).await;
        }
    }

    async fn handle_node_result(&self, result: Result<NodeResult>) {
        // ...
        match result {
            Ok(NodeResult::Done(handoff)) => {
                // Write handoff
                self.ctx_builder.handoff_store.write(&handoff).await.ok();

                // Store lesson in cm
                for lesson in &handoff.key_decisions {
                    self.ctx_builder.cm.store(lesson).await.ok();
                }

                // Index cass incremental
                self.ctx_builder.cass.index_incremental().await.ok();

                // Mark done in beads
                self.br.close(&handoff.node_id, &handoff.result_summary).await.ok();

                // Update state
                self.node_states.write().await
                    .insert(handoff.node_id.clone(), NodeState::Done {
                        handoff: handoff.clone(),
                        completed_at: Utc::now(),
                    });

                // Emit event
                let _ = self.event_tx.send(NodeEvent::Done(handoff.node_id));
            }
            Err(e) => {
                // ...handle failure...
            }
        }
    }
}
```

---

## 7. Lock Coordination (grove-lock)

```rust
// lock.rs — advisory lock, không block hard
use fs2::FileExt;

pub struct FileLock {
    _file: std::fs::File,
    path: PathBuf,
}

impl FileLock {
    pub fn try_acquire(resource: &str, node_id: &NodeId, lock_dir: &Path) -> Result<Option<Self>> {
        let hash = sha256_short(resource);
        let path = lock_dir.join(format!("{}.lock", hash));

        let file = OpenOptions::new().create(true).write(true).open(&path)?;

        match file.try_lock_exclusive() {
            Ok(_) => {
                // Write metadata
                serde_json::to_writer(&file, &serde_json::json!({
                    "node_id": node_id.0,
                    "resource": resource,
                    "acquired_at": Utc::now().to_rfc3339(),
                }))?;
                Ok(Some(FileLock { _file: file, path }))
            }
            Err(_) => Ok(None), // someone else holds it
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

---

## 8. Web UI (grove-web) — Phase 3

### API Endpoints

```
GET  /              → serve embedded index.html
GET  /api/nodes     → list tất cả nodes + states
GET  /api/nodes/:id → node detail: task, handoff, config, log tail
GET  /api/dag       → bv --robot-graph JSON cho D3.js
GET  /api/config    → grove.toml as JSON
PUT  /api/config    → update grove.toml
GET  /api/events    → SSE stream: NodeEvent (started, logline, done, failed)
POST /api/retry/:id → retry failed node
```

### Frontend

Single `index.html` embedded vào binary bằng `rust-embed`:
- D3.js force-directed graph cho DAG visualization
- Node màu theo state: gray=pending, blue=running, green=done, red=failed
- Click node → side panel: task description + live log tail + handoff JSON
- Config panel: edit grove.toml fields trực tiếp
- SSE connection cho live updates (không cần refresh)

### Axum Server

```rust
// server.rs
pub async fn start(port: u16, state: Arc<GroveState>) -> Result<()> {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/nodes", get(api_nodes))
        .route("/api/nodes/:id", get(api_node_detail))
        .route("/api/dag", get(api_dag))
        .route("/api/config", get(api_get_config).put(api_update_config))
        .route("/api/events", get(api_sse))
        .route("/api/retry/:id", post(api_retry))
        .with_state(state);

    println!("Grove web UI: http://127.0.0.1:{}", port);
    axum::Server::bind(&format!("127.0.0.1:{}", port).parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

---

## 9. Install Script (`install.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== grove installer ==="

# 1. grove binary
echo "Installing grove..."
cargo install grove 2>/dev/null || \
  cargo install --git https://github.com/quangdang46/grove

# 2. cass (required)
if ! command -v cass &>/dev/null; then
    echo "Installing cass (required)..."
    curl -fsSL \
      "https://raw.githubusercontent.com/Dicklesworthstone/coding_agent_session_search/main/install.sh" \
      | bash
else
    echo "✓ cass already installed"
fi

# 3. Initial cass index
if ! cass health &>/dev/null; then
    echo "Running initial cass index..."
    cass index || true
fi

# 4. cm (required)
if ! command -v cm &>/dev/null; then
    echo "Installing cm (required)..."
    curl -fsSL \
      "https://raw.githubusercontent.com/Dicklesworthstone/cass_memory_system/main/install.sh" \
      | bash
else
    echo "✓ cm already installed"
fi

# 5. br check
if ! command -v br &>/dev/null; then
    echo "WARNING: br (beads_rust) not found."
    echo "Install: cargo install --git https://github.com/Dicklesworthstone/beads_rust"
    echo "Or: curl -fsSL https://raw.githubusercontent.com/Dicklesworthstone/beads_rust/main/install.sh | bash"
    exit 1
fi
echo "✓ br available"

echo ""
echo "=== grove ready ==="
echo "  1. cd <your-project>"
echo "  2. br init"
echo "  3. br create 'Your task' --type task"
echo "  4. grove run"
```

---

## 10. grove.toml

```toml
[orchestrator]
max_parallel = 5
context_threshold = 80       # % estimated tokens → warn
poll_interval_secs = 5
retry_max = 3
retry_backoff_secs = 30

[session]
claude_bin = "claude"
default_model = "sonnet"     # sonnet | opus | haiku
timeout_secs = 3600

[memory]
handoff_dir = ".grove/handoffs"
lock_dir = ".grove/locks"
cass_search_limit = 5
cass_days = 30
cm_recall_limit = 10

[beads]
br_bin = "br"
bv_bin = "bv"
project_dir = "."

[web]
enabled = false
port = 3030
host = "127.0.0.1"
```

---

## 11. Dependency Table (Updated)

| Tool | Status | Install |
|------|--------|---------|
| `br` (beads_rust) | **Required** | user cài trước, grove check |
| `bv` (beads_viewer) | **Required** | grove install cài |
| `cass` | **Required** | grove install cài |
| `cm` | **Required** | grove install cài |
| `claude` CLI | **Required** | user cài trước |

Không có optional deps. Nếu thiếu cái nào → grove exit với error message rõ ràng + install command.

---

## 12. Implementation Phases

### Phase 1 — Sequential MVP

- [ ] grove-core: NodeId, NodeState, HandoffData, CheckpointData, GroveConfig
- [ ] grove-beads: BrClient (ready, show, mark_in_progress, close), BrIssue schema
- [ ] grove-session: spawn_session, parse_line (GROVE_*), ContextMonitor
- [ ] grove-memory: CassClient, CmClient, HandoffStore, ContextBuilder
- [ ] grove-orchestrator: sequential loop, checkpoint/resume
- [ ] grove-cli: `grove run`, `grove status`
- [ ] install.sh

**Milestone:** Sequential DAG end-to-end với checkpoint/resume + memory.

### Phase 2 — Parallel

- [ ] grove-lock: FileLock (fs2), LockRegistry
- [ ] grove-beads: BvClient parallel_tracks
- [ ] grove-orchestrator: tokio Semaphore, parallel spawn, NodeEvent bus
- [ ] grove-cli: `grove tui` (ratatui), `grove log`, `grove retry`, `grove tree`

**Milestone:** Parallel DAG, stable, 0 file conflicts.

### Phase 3 — Web UI

- [ ] grove-web: axum server, REST + SSE API, embedded HTML
- [ ] Frontend: D3.js DAG viz, node detail, live log, config editor
- [ ] grove-cli: `grove web`, `grove run --web`
- [ ] grove-orchestrator: graceful shutdown, events.jsonl audit log
- [ ] cm MCP HTTP mode (faster than CLI)

**Milestone:** Production-ready với web UI.

---

## 13. Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `claude -p` cần TTY (non-TTY fail) | Test trước, fallback sang PTY (portable-pty) nếu cần |
| Token heuristic không chính xác | Default threshold 80% (conservative), user có thể lower |
| cass index stale | Grove gọi `cass index` sau mỗi node done |
| cm recall không relevant | Query = task title + first 100 chars description |
| br close fail | Retry 3 lần với delay, log error, continue |
| Parallel deadlock qua file locks | Advisory lock timeout 30s, warn + continue |
| GROVE_CHECKPOINT trong output không liên quan | Parse chỉ lines bắt đầu exact bằng "GROVE_CHECKPOINT:" |

---

## 14. Success Metrics

- [ ] Phase 1: 10-node sequential DAG, correct order, checkpoint/resume hoạt động
- [ ] Phase 2: 5 parallel nodes, zero conflicts
- [ ] Phase 3: Web UI live DAG, node log stream real-time
- [ ] Memory: child node có context từ parent handoff + cass
- [ ] Orchestrator overhead < 2s per node transition
- [ ] Crash recovery: `grove run` resume từ last state
