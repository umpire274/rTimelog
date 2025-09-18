use crate::db;
use chrono::Utc;
use rusqlite::{Connection, Error, OptionalExtension};
use std::collections::HashSet;

pub struct Migration {
    pub version: &'static str,
    pub description: &'static str,
    pub up: fn(&Connection) -> rusqlite::Result<()>, // ✅ giusto
}

/// Assicurati che esista la tabella che traccia le migrazioni applicate
fn ensure_migrations_table(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version     TEXT PRIMARY KEY,
            applied_at  TEXT NOT NULL
        );
        "#,
    )
}

/// Leggi le versioni già applicate
fn applied_versions(conn: &Connection) -> Result<HashSet<String>, Error> {
    let mut set = HashSet::new();
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for r in rows {
        set.insert(r?);
    }
    Ok(set)
}

/// Segna come applicata una migrazione (solo dopo successo)
fn mark_applied(conn: &Connection, version: &str) -> Result<(), Error> {
    conn.execute(
        "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
        (version, Utc::now().to_rfc3339()),
    )?;
    Ok(())
}

/// Elenco delle migrazioni in ORDINE (verranno eseguite in sequenza)
static ALL_MIGRATIONS: &[Migration] = &[
    Migration {
        version: "20250919_0001_create_log_table_and_position_H",
        description: "Create 'log' table and extend position to include 'H'",
        up: migrate_to_030_rel,
    },
    Migration {
        version: "20250919_0002_position_add_C",
        description: "Extend position CHECK to include 'C'",
        up: migrate_to_032_rel,
    },
];

/// Esegui solo le migrazioni non ancora applicate
pub fn run_pending_migrations(conn: &Connection) -> Result<(), Error> {
    ensure_migrations_table(conn)?;

    let applied = applied_versions(conn)?;
    for m in ALL_MIGRATIONS {
        if !applied.contains(m.version) {
            // Applica la migrazione
            (m.up)(conn)?;
            // Marca come applicata
            mark_applied(conn, m.version)?;
            println!("✅ Migration applied: {} — {}", m.version, m.description);
        }
    }
    Ok(())
}

fn migrate_to_030_rel(conn: &Connection) -> rusqlite::Result<()> {
    // create new table log
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            function TEXT NOT NULL,
            message TEXT NOT NULL
        );
        ",
    )?;

    let mut stmt =
        conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='work_sessions'")?;
    let table_sql: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;

    if let Some(sql) = table_sql
        && sql.contains("CHECK (position IN ('O','R'))")
    {
        println!("⚠️  Old schema detected, migrating work_sessions to support 'H' (Holiday)...");

        conn.execute_batch(
            "
                ALTER TABLE work_sessions RENAME TO work_sessions_old;

                CREATE TABLE work_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    position TEXT NOT NULL CHECK(position IN ('O','R','H')),
                    start_time TEXT DEFAULT '',
                    lunch_break INTEGER DEFAULT 0,
                    end_time TEXT DEFAULT ''
                );

                INSERT INTO work_sessions (id, date, position, start_time, lunch_break, end_time)
                SELECT id, date, position, start_time, lunch_break, end_time
                FROM work_sessions_old;

                DROP TABLE work_sessions_old;
                ",
        )?;

        db::ttlog(
            conn,
            "migrate_to_030_rel",
            "Migration table \'work_sessions\' completed.",
        )?;
        println!("✅ Migration completed successfully.");
    }

    Ok(())
}

fn migrate_to_032_rel(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt =
        conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='work_sessions'")?;
    let table_sql: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;

    if let Some(sql) = table_sql
        && sql.contains("CHECK(position IN ('O','R','H'))")
    {
        println!("⚠️  Old schema detected, migrating work_sessions to support 'C' (On-Site)...");

        conn.execute_batch(
            "
                ALTER TABLE work_sessions RENAME TO work_sessions_old;

                CREATE TABLE work_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    position TEXT NOT NULL CHECK(position IN ('O','R','H','C')),
                    start_time TEXT DEFAULT '',
                    lunch_break INTEGER DEFAULT 0,
                    end_time TEXT DEFAULT ''
                );

                INSERT INTO work_sessions (id, date, position, start_time, lunch_break, end_time)
                SELECT id, date, position, start_time, lunch_break, end_time
                FROM work_sessions_old;

                DROP TABLE work_sessions_old;
                ",
        )?;

        db::ttlog(
            conn,
            "migrate_to_032_rel",
            "Migration table \'work_sessions\' completed.",
        )?;
        println!("✅ Migration completed successfully.");
    }

    Ok(())
}
