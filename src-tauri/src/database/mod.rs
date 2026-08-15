pub mod commands;
pub mod event_journal;
pub mod history_codec;
pub mod history_query;
pub mod models;
pub mod schema;

use std::path::{Path, PathBuf};
use std::sync::{OnceLock, mpsc};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};

use crate::database::schema as sch;

use self::event_journal::{FinalizeEncounter, FinalizeOutcome, InsertOutcome, RecordingEncounter};
use self::history_codec::EncodedHistoryChunk;
use self::history_query::{
    EncounterDetailData, EncounterDetailQuery, EncounterRangeData, EncounterRangeQuery,
};

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();
const MAX_ENCOUNTER_HISTORY: i64 = 200;
const DATABASE_QUEUE_CAPACITY: usize = 256;

type DatabaseOperation = Box<dyn FnOnce(&mut SqliteConnection) + Send + 'static>;

enum DatabaseRequest {
    Execute(DatabaseOperation),
    BeginRecording {
        recording: RecordingEncounter,
        reply: mpsc::SyncSender<Result<i32, String>>,
    },
    AppendChunk {
        chunk: EncodedHistoryChunk,
        reply: mpsc::SyncSender<Result<InsertOutcome, String>>,
    },
    Finalize {
        finalize: FinalizeEncounter,
        reply: mpsc::SyncSender<Result<FinalizeOutcome, String>>,
    },
    LoadDetail {
        encounter_id: i32,
        reply: mpsc::SyncSender<Result<EncounterDetailQuery, String>>,
    },
    LoadRange {
        encounter_id: i32,
        start_ms: u64,
        end_ms_exclusive: u64,
        reply: mpsc::SyncSender<Result<EncounterRangeQuery, String>>,
    },
    Barrier(mpsc::SyncSender<Result<(), String>>),
    Shutdown(mpsc::SyncSender<Result<(), String>>),
}

static DATABASE_SENDER: OnceLock<mpsc::SyncSender<DatabaseRequest>> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum DbInitError {
    #[error("DB connection error: {0}")]
    Connection(String),
    #[error("DB migration error: {0}")]
    Migration(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerNameEntry {
    pub name: String,
    pub class_id: i32,
}

pub fn default_db_path() -> PathBuf {
    if let Some(mut dir) = dirs::data_local_dir() {
        dir.push("resonance-logs-cn");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("resonance-logs-cn.db")
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("resonance-logs-cn.db")
    }
}

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn init_db() -> Result<(), DbInitError> {
    if DATABASE_SENDER.get().is_some() {
        return Ok(());
    }
    let db_path = default_db_path();
    log::info!(target: "app::db", "db_path={}", db_path.display());
    ensure_parent_dir(&db_path).map_err(|error| {
        DbInitError::Connection(format!("failed to create DB directory: {error}"))
    })?;
    let mut conn = SqliteConnection::establish(&db_path.to_string_lossy())
        .map_err(|error| DbInitError::Connection(error.to_string()))?;
    apply_sqlite_pragmas(&mut conn).map_err(DbInitError::Connection)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|error| DbInitError::Migration(error.to_string()))?;

    let (sender, receiver) = mpsc::sync_channel(DATABASE_QUEUE_CAPACITY);
    std::thread::Builder::new()
        .name("db-worker".to_string())
        .spawn(move || database_actor(conn, receiver))
        .map_err(|error| {
            DbInitError::Connection(format!("failed to spawn database actor: {error}"))
        })?;
    DATABASE_SENDER
        .set(sender)
        .map_err(|_| DbInitError::Connection("database actor already initialized".to_string()))?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn init_in_memory_actor_for_test() -> Result<(), String> {
    if DATABASE_SENDER.get().is_some() {
        return Err("database actor is already initialized".to_string());
    }

    let mut conn = SqliteConnection::establish(":memory:").map_err(|error| error.to_string())?;
    apply_sqlite_pragmas(&mut conn)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|error| error.to_string())?;

    let (sender, receiver) = mpsc::sync_channel(DATABASE_QUEUE_CAPACITY);
    std::thread::Builder::new()
        .name("db-worker-test".to_string())
        .spawn(move || database_actor(conn, receiver))
        .map_err(|error| format!("failed to spawn test database actor: {error}"))?;
    DATABASE_SENDER
        .set(sender)
        .map_err(|_| "database actor was initialized concurrently".to_string())
}

fn apply_sqlite_pragmas(conn: &mut SqliteConnection) -> Result<(), String> {
    for (name, statement) in [
        ("busy_timeout", "PRAGMA busy_timeout=30000;"),
        ("journal_mode", "PRAGMA journal_mode=WAL;"),
        ("synchronous", "PRAGMA synchronous=NORMAL;"),
        ("foreign_keys", "PRAGMA foreign_keys=ON;"),
    ] {
        diesel::sql_query(statement)
            .execute(conn)
            .map_err(|error| format!("failed to apply SQLite {name} pragma: {error}"))?;
    }
    Ok(())
}

fn database_actor(mut conn: SqliteConnection, receiver: mpsc::Receiver<DatabaseRequest>) {
    let mut shutdown_reply = None;
    while let Ok(request) = receiver.recv() {
        match request {
            DatabaseRequest::Execute(operation) => operation(&mut conn),
            DatabaseRequest::BeginRecording { recording, reply } => {
                let result = event_journal::begin_recording_encounter(&mut conn, &recording)
                    .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            DatabaseRequest::AppendChunk { chunk, reply } => {
                let result = event_journal::append_chunk(&mut conn, &chunk)
                    .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            DatabaseRequest::Finalize { finalize, reply } => {
                let result = event_journal::finalize_encounter(&mut conn, &finalize)
                    .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            DatabaseRequest::LoadDetail {
                encounter_id,
                reply,
            } => {
                let result =
                    commands::load_encounter_summary(&mut conn, encounter_id).and_then(|summary| {
                        history_query::load_encounter_detail_query(&mut conn, summary)
                            .map_err(|error| error.to_string())
                    });
                let _ = reply.send(result);
            }
            DatabaseRequest::LoadRange {
                encounter_id,
                start_ms,
                end_ms_exclusive,
                reply,
            } => {
                let result = history_query::load_encounter_range_query(
                    &mut conn,
                    encounter_id,
                    start_ms,
                    end_ms_exclusive,
                )
                .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            DatabaseRequest::Barrier(reply) => {
                let _ = reply.send(Ok(()));
            }
            DatabaseRequest::Shutdown(reply) => {
                shutdown_reply = Some(reply);
                break;
            }
        }
    }
    drop(conn);
    if let Some(reply) = shutdown_reply {
        let _ = reply.send(Ok(()));
    }
    log::info!(target: "app::db", "database_actor_exiting");
}

fn database_sender() -> Result<mpsc::SyncSender<DatabaseRequest>, String> {
    DATABASE_SENDER
        .get()
        .cloned()
        .ok_or_else(|| "database actor is not initialized".to_string())
}

fn request<T>(
    build: impl FnOnce(mpsc::SyncSender<Result<T, String>>) -> DatabaseRequest,
) -> Result<T, String> {
    let sender = database_sender()?;
    let (reply, receiver) = mpsc::sync_channel(0);
    sender
        .send(build(reply))
        .map_err(|_| "database actor queue is closed".to_string())?;
    receiver
        .recv()
        .map_err(|_| "database actor dropped the response".to_string())?
}

/// Runs a non-history operation on the actor-owned connection.
pub fn db_exec<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut SqliteConnection) -> Result<T, String> + Send + 'static,
{
    let sender = database_sender()?;
    let (reply, receiver) = mpsc::sync_channel(0);
    sender
        .send(DatabaseRequest::Execute(Box::new(move |conn| {
            let _ = reply.send(operation(conn));
        })))
        .map_err(|_| "database actor queue is closed".to_string())?;
    receiver
        .recv()
        .map_err(|_| "database actor dropped the response".to_string())?
}

pub fn db_send<F>(operation: F)
where
    F: FnOnce(&mut SqliteConnection) + Send + 'static,
{
    let Ok(sender) = database_sender() else {
        log::error!(target: "app::db", "database_send_failed reason=not_initialized");
        return;
    };
    if sender
        .send(DatabaseRequest::Execute(Box::new(operation)))
        .is_err()
    {
        log::error!(target: "app::db", "database_send_failed reason=queue_closed");
    }
}

pub fn begin_history_recording(recording: RecordingEncounter) -> Result<i32, String> {
    request(|reply| DatabaseRequest::BeginRecording { recording, reply })
}

pub fn append_history_chunk(chunk: EncodedHistoryChunk) -> Result<InsertOutcome, String> {
    request(|reply| DatabaseRequest::AppendChunk { chunk, reply })
}

pub fn finalize_history_recording(finalize: FinalizeEncounter) -> Result<FinalizeOutcome, String> {
    request(|reply| DatabaseRequest::Finalize { finalize, reply })
}

pub fn load_history_detail(
    encounter_id: i32,
    target_points: u32,
) -> Result<EncounterDetailData, String> {
    let query = request(|reply| DatabaseRequest::LoadDetail {
        encounter_id,
        reply,
    })?;
    history_query::project_encounter_detail(query, target_points).map_err(|error| error.to_string())
}

pub fn load_history_range(
    encounter_id: i32,
    start_ms: u64,
    end_ms_exclusive: u64,
) -> Result<EncounterRangeData, String> {
    let query = request(|reply| DatabaseRequest::LoadRange {
        encounter_id,
        start_ms,
        end_ms_exclusive,
        reply,
    })?;
    history_query::project_encounter_range(query, start_ms, end_ms_exclusive)
        .map_err(|error| error.to_string())
}

pub fn flush_database() -> Result<(), String> {
    request(DatabaseRequest::Barrier)
}

pub fn shutdown_database() -> Result<(), String> {
    if DATABASE_SENDER.get().is_none() {
        return Ok(());
    }
    request(DatabaseRequest::Shutdown)
}

/// Delete old non-favorite summaries. Primary keys are stable and are never
/// rewritten; child chunks/projections are removed by foreign-key cascade.
pub fn startup_maintenance() {
    db_send(|conn| {
        use sch::encounters::dsl as e;
        match diesel::delete(e::encounters.filter(e::ended_at_ms.is_null())).execute(conn) {
            Ok(deleted) if deleted > 0 => {
                log::info!(target: "app::db", "startup_maintenance_removed_recording deleted={deleted}");
            }
            Ok(_) => {}
            Err(error) => {
                log::warn!(target: "app::db", "startup_maintenance_recording_cleanup_failed error={error}");
                return;
            }
        }
        if let Err(error) = prune_encounters(conn, MAX_ENCOUNTER_HISTORY) {
            log::warn!(target: "app::db", "startup_maintenance_failed error={error}");
        }
    });
}

fn prune_encounters(conn: &mut SqliteConnection, keep: i64) -> Result<(), String> {
    use sch::encounters::dsl as e;
    let delete_ids = e::encounters
        .select(e::id)
        .filter(e::is_favorite.eq(0))
        .order((e::started_at_ms.desc(), e::id.desc()))
        .offset(keep.max(0))
        .load::<i32>(conn)
        .map_err(|error| error.to_string())?;
    if delete_ids.is_empty() {
        return Ok(());
    }
    let deleted = diesel::delete(e::encounters.filter(e::id.eq_any(delete_ids)))
        .execute(conn)
        .map_err(|error| error.to_string())?;
    log::info!(target: "app::db", "startup_maintenance_pruned deleted={deleted} keep={keep}");
    Ok(())
}
