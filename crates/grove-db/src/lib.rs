use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};

pub const CRATE_PURPOSE: &str = "SQLite bootstrap, migrations, and runtime persistence.";

const PRAGMAS: &[&str] = &[
    "PRAGMA journal_mode = WAL;",
    "PRAGMA foreign_keys = ON;",
    "PRAGMA synchronous = NORMAL;",
    "PRAGMA temp_store = MEMORY;",
    "PRAGMA busy_timeout = 5000;",
];

const MIGRATION_MANIFEST: &[Migration<'_>] = &[Migration {
    version: 1,
    name: "0001_init.sql",
    sql: include_str!("../migrations/0001_init.sql"),
}];

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationState {
    pub version: i64,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
struct Migration<'a> {
    version: i64,
    name: &'a str,
    sql: &'a str,
}

impl Database {
    pub fn open(path: &Utf8Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create database parent directory: {parent}"))?;
        }

        let connection = Connection::open_with_flags(
            utf8_to_std_path(path)?,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("open SQLite database at {path}"))?;

        apply_pragmas(&connection)?;

        Ok(Self { conn: connection })
    }

    pub fn migrate(&mut self) -> Result<()> {
        ensure_migration_table(&self.conn)?;

        for migration in MIGRATION_MANIFEST {
            let applied_name = self.applied_migration_name(migration.version)?;
            match applied_name {
                Some(existing_name) if existing_name == migration.name => continue,
                Some(existing_name) => {
                    bail!(
                        "migration version {} already applied with different name: {} != {}",
                        migration.version,
                        existing_name,
                        migration.name
                    );
                }
                None => self.apply_migration(*migration)?,
            }
        }

        Ok(())
    }

    pub fn with_tx<T>(&mut self, f: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        let tx = self.conn.transaction().context("begin transaction")?;
        let value = f(&tx)?;
        tx.commit().context("commit transaction")?;
        Ok(value)
    }

    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn applied_migrations(&self) -> Result<Vec<MigrationState>> {
        let mut stmt = self
            .conn
            .prepare("SELECT version, name FROM _migrations ORDER BY version")
            .context("prepare applied migrations query")?;

        let rows = stmt
            .query_map([], |row| {
                let version = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((version, name))
            })
            .context("query applied migrations")?;

        let pairs = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("collect applied migrations")?;

        Ok(pairs
            .into_iter()
            .map(|(version, name)| MigrationState { version, name })
            .collect())
    }

    fn applied_migration_name(&self, version: i64) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT name FROM _migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .optional()
            .with_context(|| format!("lookup applied migration version {version}"))
    }

    fn apply_migration(&mut self, migration: Migration<'_>) -> Result<()> {
        let tx = self
            .conn
            .transaction()
            .with_context(|| format!("begin migration {} transaction", migration.name))?;
        tx.execute_batch(migration.sql)
            .with_context(|| format!("execute migration {}", migration.name))?;
        tx.execute(
            "INSERT INTO _migrations(version, name) VALUES (?1, ?2)",
            (migration.version, migration.name),
        )
        .with_context(|| format!("record migration {}", migration.name))?;
        tx.commit()
            .with_context(|| format!("commit migration {}", migration.name))?;
        Ok(())
    }
}

pub fn migrations_dir() -> &'static str {
    "migrations"
}

fn apply_pragmas(conn: &Connection) -> Result<()> {
    for pragma in PRAGMAS {
        conn.execute_batch(pragma)
            .with_context(|| format!("apply SQLite pragma {pragma}"))?;
    }
    Ok(())
}

fn ensure_migration_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (\
            version INTEGER PRIMARY KEY,\
            name TEXT NOT NULL,\
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP\
        );",
    )
    .context("ensure _migrations table exists")?;
    Ok(())
}

fn utf8_to_std_path(path: &Utf8Path) -> Result<PathBuf> {
    let std_path = Utf8PathBuf::from(path).into_std_path_buf();
    if std_path.as_os_str().is_empty() {
        bail!("database path resolved to an empty path from {path}");
    }
    Ok(std_path)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use camino::Utf8PathBuf;
    use rusqlite::OptionalExtension;
    use tempfile::tempdir;

    use super::Database;

    #[test]
    fn open_creates_database_parent_directory() -> Result<()> {
        let dir = tempdir()?;
        let db_path = Utf8PathBuf::from_path_buf(dir.path().join("nested/.grove/grove.db"))
            .map_err(|_| anyhow::anyhow!("temp path was not valid UTF-8"))?;

        let _db = Database::open(&db_path)?;

        assert!(db_path.exists());
        Ok(())
    }

    #[test]
    fn migrate_applies_manifest_once() -> Result<()> {
        let dir = tempdir()?;
        let db_path = Utf8PathBuf::from_path_buf(dir.path().join("grove.db"))
            .map_err(|_| anyhow::anyhow!("temp path was not valid UTF-8"))?;
        let mut db = Database::open(&db_path)?;

        db.migrate()?;
        db.migrate()?;

        let migrations = db.applied_migrations()?;
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].name, "0001_init.sql");
        Ok(())
    }

    #[test]
    fn migrate_creates_runtime_tables() -> Result<()> {
        let dir = tempdir()?;
        let db_path = Utf8PathBuf::from_path_buf(dir.path().join("grove.db"))
            .map_err(|_| anyhow::anyhow!("temp path was not valid UTF-8"))?;
        let mut db = Database::open(&db_path)?;

        db.migrate()?;

        for table in [
            "_migrations",
            "bead_cache",
            "bead_runtime",
            "bead_dependencies",
            "task_runs",
            "claude_sessions",
            "checkpoints",
            "handoffs",
            "reservations",
            "event_log",
        ] {
            let exists: Option<String> = db
                .connection()
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .optional()?;
            assert_eq!(exists.as_deref(), Some(table));
        }

        Ok(())
    }

    #[test]
    fn with_tx_commits_changes() -> Result<()> {
        let dir = tempdir()?;
        let db_path = Utf8PathBuf::from_path_buf(dir.path().join("grove.db"))
            .map_err(|_| anyhow::anyhow!("temp path was not valid UTF-8"))?;
        let mut db = Database::open(&db_path)?;
        db.migrate()?;

        db.with_tx(|tx| {
            tx.execute(
                "INSERT INTO bead_cache(\
                    bead_id, title, description, priority, issue_type, status, assignee,\
                    labels_json, parent_ids_json, dependency_ids_json, dependent_ids_json,\
                    raw_json, synced_at\
                ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, NULL, '[]', '[]', '[]', '[]', ?6, CURRENT_TIMESTAMP)",
                (
                    "grove-123",
                    "Example bead",
                    0,
                    "task",
                    "open",
                    "{}",
                ),
            )?;
            Ok(())
        })?;

        let count: i64 =
            db.connection()
                .query_row("SELECT COUNT(*) FROM bead_cache", [], |row| row.get(0))?;
        assert_eq!(count, 1);
        Ok(())
    }
}
