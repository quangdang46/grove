use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use grove_config::{
    DEFAULT_GROVE_DIR_NAME, DEFAULT_PLAYBOOK_DOCS_DIR_NAME, DEFAULT_WORKFLOW_DOCS_DIR_NAME,
    GroveConfig,
};
use grove_types::{
    BulletId, GroveBeadRecord,
    playbook::{BulletMaturity, BulletScope, BulletState, BulletType, PlaybookBulletRecord},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    sync::{Mutex, OnceLock},
};

#[derive(Debug, Clone, Default)]
pub(crate) struct LoadedOperatorDocuments {
    workflow_guides: Vec<WorkflowGuide>,
    pub(crate) playbook_rules: Vec<PlaybookBulletRecord>,
}

impl LoadedOperatorDocuments {
    #[must_use]
    pub(crate) fn merge_startup_prompt(&self, base_prompt: Option<String>) -> Option<String> {
        let mut sections = Vec::new();
        if let Some(base_prompt) = base_prompt
            && !base_prompt.trim().is_empty()
        {
            sections.push(base_prompt.trim().to_owned());
        }
        for guide in &self.workflow_guides {
            sections.push(format!(
                "Workflow guide: {}\n{}",
                guide.title,
                guide.text.trim()
            ));
        }
        if sections.is_empty() {
            None
        } else {
            Some(sections.join("\n\n"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowGuide {
    title: String,
    text: String,
}

#[derive(Debug, Clone)]
struct CachedOperatorDocuments {
    sources: Vec<MarkdownSource>,
    documents: Vec<MarkdownDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownSource {
    kind: DocumentKind,
    path: Utf8PathBuf,
    raw: String,
}

#[derive(Debug, Clone)]
struct MarkdownDocument {
    kind: DocumentKind,
    path: Utf8PathBuf,
    title: String,
    body: String,
    frontmatter: DocFrontmatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentKind {
    Workflow,
    Playbook,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct DocFrontmatter {
    title: Option<String>,
    enabled: bool,
    phases: Vec<String>,
    issue_types: Vec<String>,
    providers: Vec<String>,
    category: Option<String>,
    tags: Vec<String>,
    pinned: bool,
    scope: Option<String>,
    scope_key: Option<String>,
    bullet_type: Option<String>,
}

impl Default for DocFrontmatter {
    fn default() -> Self {
        Self {
            title: None,
            enabled: true,
            phases: Vec::new(),
            issue_types: Vec::new(),
            providers: Vec::new(),
            category: None,
            tags: Vec::new(),
            pinned: false,
            scope: None,
            scope_key: None,
            bullet_type: None,
        }
    }
}

static OPERATOR_DOC_CACHE: OnceLock<Mutex<HashMap<String, CachedOperatorDocuments>>> =
    OnceLock::new();

pub(crate) fn load_operator_documents(
    config: &GroveConfig,
    working_dir: &Utf8Path,
    bead: &GroveBeadRecord,
) -> Result<LoadedOperatorDocuments> {
    let documents = load_cached_documents(working_dir)?;
    let mut loaded = LoadedOperatorDocuments::default();

    for document in documents {
        if !document_enabled_for(&document.frontmatter, config, bead) {
            continue;
        }

        match document.kind {
            DocumentKind::Workflow => loaded.workflow_guides.push(WorkflowGuide {
                title: document.title,
                text: document.body,
            }),
            DocumentKind::Playbook => loaded
                .playbook_rules
                .extend(playbook_rules_from_document(working_dir, &document)?),
        }
    }

    Ok(loaded)
}

fn load_cached_documents(working_dir: &Utf8Path) -> Result<Vec<MarkdownDocument>> {
    let sources = collect_markdown_sources(working_dir)?;
    let cache = OPERATOR_DOC_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache
        .lock()
        .map_err(|_| anyhow::anyhow!("operator docs cache mutex poisoned"))?;
    let cache_key = working_dir.as_str().to_owned();

    if let Some(cached) = guard.get(&cache_key)
        && cached.sources == sources
    {
        return Ok(cached.documents.clone());
    }

    let documents = sources
        .iter()
        .map(parse_markdown_document)
        .collect::<Result<Vec<_>>>()?;

    guard.insert(
        cache_key,
        CachedOperatorDocuments {
            sources,
            documents: documents.clone(),
        },
    );

    Ok(documents)
}

fn collect_markdown_sources(working_dir: &Utf8Path) -> Result<Vec<MarkdownSource>> {
    let mut sources = Vec::new();
    sources.extend(read_markdown_tree(
        DocumentKind::Workflow,
        &working_dir
            .join(DEFAULT_GROVE_DIR_NAME)
            .join(DEFAULT_WORKFLOW_DOCS_DIR_NAME),
    )?);
    sources.extend(read_markdown_tree(
        DocumentKind::Playbook,
        &working_dir
            .join(DEFAULT_GROVE_DIR_NAME)
            .join(DEFAULT_PLAYBOOK_DOCS_DIR_NAME),
    )?);
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(sources)
}

fn read_markdown_tree(kind: DocumentKind, root: &Utf8Path) -> Result<Vec<MarkdownSource>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut pending = vec![root.to_owned()];
    let mut sources = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir.as_std_path())
            .with_context(|| format!("read operator docs directory {}", dir.as_str()))?
        {
            let entry = entry?;
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                anyhow::anyhow!("operator docs path must be valid UTF-8: {}", path.display())
            })?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if !metadata.is_file() || path.extension() != Some("md") {
                continue;
            }

            let raw = fs::read_to_string(path.as_std_path())
                .with_context(|| format!("read operator doc {}", path.as_str()))?;
            sources.push(MarkdownSource { kind, path, raw });
        }
    }

    Ok(sources)
}

fn parse_markdown_document(source: &MarkdownSource) -> Result<MarkdownDocument> {
    let (frontmatter, body) = split_frontmatter(&source.raw)?;
    let title = frontmatter.title.clone().unwrap_or_else(|| {
        source
            .path
            .file_stem()
            .unwrap_or("operator-doc")
            .replace('-', " ")
    });

    Ok(MarkdownDocument {
        kind: source.kind,
        path: source.path.clone(),
        title,
        body,
        frontmatter,
    })
}

fn split_frontmatter(raw: &str) -> Result<(DocFrontmatter, String)> {
    let normalized = raw.replace("\r\n", "\n");
    let trimmed = normalized.trim();
    if let Some(rest) = trimmed.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let (frontmatter_raw, body_with_delimiter) = rest.split_at(end);
        let frontmatter =
            toml::from_str(frontmatter_raw).with_context(|| "parse operator doc frontmatter")?;
        let body = body_with_delimiter
            .trim_start_matches("\n---\n")
            .trim()
            .to_owned();
        return Ok((frontmatter, body));
    }

    Ok((DocFrontmatter::default(), trimmed.to_owned()))
}

fn document_enabled_for(
    frontmatter: &DocFrontmatter,
    config: &GroveConfig,
    bead: &GroveBeadRecord,
) -> bool {
    if !frontmatter.enabled {
        return false;
    }

    if !frontmatter.providers.is_empty()
        && !frontmatter
            .providers
            .iter()
            .any(|provider| provider.eq_ignore_ascii_case(config.runtime.provider.as_str()))
    {
        return false;
    }

    if !frontmatter.issue_types.is_empty()
        && !frontmatter
            .issue_types
            .iter()
            .any(|issue_type| issue_type.eq_ignore_ascii_case(&bead.bead.issue_type))
    {
        return false;
    }

    if frontmatter.phases.is_empty() {
        return true;
    }

    bead.workflow_state().is_some_and(|state| {
        frontmatter
            .phases
            .iter()
            .any(|phase| phase.eq_ignore_ascii_case(state.phase.as_str()))
    })
}

fn playbook_rules_from_document(
    working_dir: &Utf8Path,
    document: &MarkdownDocument,
) -> Result<Vec<PlaybookBulletRecord>> {
    let bullet_lines = markdown_bullets(&document.body);
    let scope = parse_scope(document.frontmatter.scope.as_deref())?;
    let bullet_type = parse_bullet_type(document.frontmatter.bullet_type.as_deref())?;
    let relative_path = document
        .path
        .strip_prefix(working_dir)
        .unwrap_or(document.path.as_path())
        .as_str()
        .to_owned();
    let created_at = chrono::Utc::now();

    Ok(bullet_lines
        .into_iter()
        .enumerate()
        .map(|(index, text)| PlaybookBulletRecord {
            id: BulletId::new(format!("mdoc-{}-{}", slugify(&relative_path), index + 1)),
            scope,
            scope_key: document.frontmatter.scope_key.clone(),
            category: document
                .frontmatter
                .category
                .clone()
                .unwrap_or_else(|| "workflow".to_owned()),
            text,
            bullet_type,
            state: BulletState::Active,
            maturity: BulletMaturity::Established,
            helpful_count: 0,
            harmful_count: 0,
            feedback_events: Vec::new(),
            confidence_decay_half_life_days: 30,
            pinned: document.frontmatter.pinned,
            deprecated: false,
            replaced_by: None,
            deprecation_reason: None,
            source_bead_ids: Vec::new(),
            source_run_ids: Vec::new(),
            tags: document.frontmatter.tags.clone(),
            effective_score: Some(if document.frontmatter.pinned {
                1_000.0
            } else {
                500.0
            }),
            created_at,
            updated_at: created_at,
        })
        .collect())
}

fn markdown_bullets(body: &str) -> Vec<String> {
    let bullets = body
        .lines()
        .filter_map(markdown_bullet_line)
        .collect::<Vec<_>>();
    if !bullets.is_empty() {
        return bullets;
    }

    let trimmed = body.trim();
    if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![trimmed.to_owned()]
    }
}

fn markdown_bullet_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    for prefix in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_owned());
            }
        }
    }

    let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_count > 0 {
        let remainder = &trimmed[digit_count..];
        if let Some(rest) = remainder.strip_prefix(". ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_owned());
            }
        }
    }

    None
}

fn parse_scope(scope: Option<&str>) -> Result<BulletScope> {
    match scope.map(|value| value.to_ascii_lowercase()) {
        None => Ok(BulletScope::Workspace),
        Some(scope) if scope == "global" => Ok(BulletScope::Global),
        Some(scope) if scope == "workspace" => Ok(BulletScope::Workspace),
        Some(scope) if scope == "language" => Ok(BulletScope::Language),
        Some(scope) if scope == "framework" => Ok(BulletScope::Framework),
        Some(scope) if scope == "bead" => Ok(BulletScope::Bead),
        Some(scope) => Err(anyhow::anyhow!("unsupported playbook scope `{scope}`")),
    }
}

fn parse_bullet_type(bullet_type: Option<&str>) -> Result<BulletType> {
    match bullet_type.map(|value| value.to_ascii_lowercase()) {
        None => Ok(BulletType::Rule),
        Some(bullet_type) if bullet_type == "rule" => Ok(BulletType::Rule),
        Some(bullet_type) if bullet_type == "anti_pattern" => Ok(BulletType::AntiPattern),
        Some(bullet_type) if bullet_type == "antipattern" => Ok(BulletType::AntiPattern),
        Some(bullet_type) => Err(anyhow::anyhow!(
            "unsupported playbook bullet type `{bullet_type}`"
        )),
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::load_operator_documents;
    use camino::Utf8PathBuf;
    use grove_config::GroveConfig;
    use grove_types::{BeadId, BeadPriority, BeadRef, GroveBeadRecord, GroveBeadStatus, Timestamp};
    use tempfile::tempdir;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn workflow_docs_respect_enablement_rules() -> TestResult {
        let dir = tempdir()?;
        let workspace = Utf8PathBuf::from_path_buf(dir.path().to_path_buf())
            .map_err(|_| std::io::Error::other("workspace must be utf8"))?;
        let workflows_dir = workspace.join(".grove/workflows");
        std::fs::create_dir_all(workflows_dir.as_std_path())?;
        std::fs::write(
            workflows_dir.join("review.md").as_std_path(),
            concat!(
                "---\n",
                "title = \"Review gate\"\n",
                "phases = [\"review\"]\n",
                "issue_types = [\"feature\"]\n",
                "providers = [\"claude\"]\n",
                "---\n",
                "Double-check the workflow output before closing the bead.\n",
            ),
        )?;

        let config = GroveConfig::default();
        let bead = sample_bead("feature", &["grove:workflow:review"])?;
        let docs = load_operator_documents(&config, &workspace, &bead)?;
        let startup = docs
            .merge_startup_prompt(None)
            .ok_or_else(|| std::io::Error::other("workflow guide should load"))?;
        assert!(startup.contains("Review gate"));
        assert!(startup.contains("Double-check the workflow output"));

        let non_matching = sample_bead("task", &[])?;
        let docs = load_operator_documents(&config, &workspace, &non_matching)?;
        assert!(docs.merge_startup_prompt(None).is_none());
        Ok(())
    }

    #[test]
    fn playbook_docs_convert_markdown_bullets_and_reload_on_change() -> TestResult {
        let dir = tempdir()?;
        let workspace = Utf8PathBuf::from_path_buf(dir.path().to_path_buf())
            .map_err(|_| std::io::Error::other("workspace must be utf8"))?;
        let playbooks_dir = workspace.join(".grove/playbooks");
        std::fs::create_dir_all(playbooks_dir.as_std_path())?;
        let path = playbooks_dir.join("rules.md");
        std::fs::write(
            path.as_std_path(),
            concat!(
                "---\n",
                "category = \"workflow\"\n",
                "pinned = true\n",
                "---\n",
                "- Keep generated child beads explicit.\n",
                "- Re-read AGENTS before large refactors.\n",
            ),
        )?;

        let config = GroveConfig::default();
        let bead = sample_bead("task", &[])?;
        let docs = load_operator_documents(&config, &workspace, &bead)?;
        assert_eq!(docs.playbook_rules.len(), 2);
        assert_eq!(
            docs.playbook_rules[0].text,
            "Keep generated child beads explicit."
        );
        assert!(docs.playbook_rules[0].pinned);

        std::fs::write(
            path.as_std_path(),
            concat!(
                "---\n",
                "category = \"workflow\"\n",
                "---\n",
                "- Prefer markdown-defined guides over hard-coded prompt glue.\n",
            ),
        )?;

        let docs = load_operator_documents(&config, &workspace, &bead)?;
        assert_eq!(docs.playbook_rules.len(), 1);
        assert_eq!(
            docs.playbook_rules[0].text,
            "Prefer markdown-defined guides over hard-coded prompt glue."
        );
        Ok(())
    }

    fn sample_bead(issue_type: &str, labels: &[&str]) -> TestResult<GroveBeadRecord> {
        let created_at: Timestamp = "2026-03-16T10:00:00Z".parse()?;
        let updated_at: Timestamp = "2026-03-16T11:00:00Z".parse()?;
        Ok(GroveBeadRecord {
            bead: BeadRef {
                id: BeadId::new("grove-1af.3"),
                title: "workflow docs".to_owned(),
                description: None,
                priority: BeadPriority::P1,
                issue_type: issue_type.to_owned(),
                br_status: "open".to_owned(),
                assignee: None,
                labels: labels.iter().map(|label| (*label).to_owned()).collect(),
                created_at,
                updated_at,
            },
            grove_status: GroveBeadStatus::Ready,
            declared_paths: Vec::new(),
            metadata: Default::default(),
            last_run_id: None,
            retry_after: None,
            last_failure_class: None,
            last_failure_detail: None,
            circuit_breaker_state: None,
            synced_at: updated_at,
            runtime_updated_at: updated_at,
        })
    }
}
