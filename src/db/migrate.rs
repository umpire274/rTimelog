use crate::config::Config;
use crate::db;
use chrono::Utc;
use rusqlite::{Connection, Error, OptionalExtension, ffi, params};
use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;

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
    Migration {
        version: "20250919_0003_upgrade_config_file",
        description: "Add into config file two new parameters: min_duration_lunch_break=30 e max_duration_lunch_break=90",
        up: migrate_to_033_rel,
    },
    Migration {
        version: "20250930_0004_add_work_sessions_indexes",
        description: "Add indexes on work_sessions.date and work_sessions.position",
        up: migrate_to_034_rel,
    },
    Migration {
        version: "20251001_0005_add_separator_char_to_config",
        description: "Add separator_char default to configuration file if missing",
        up: migrate_to_035_rel,
    },
    Migration {
        version: "20251010_0006_create_events_table",
        description: "Create events table to store time punches (in/out) with position and lunch",
        up: migrate_to_036_create_events,
    },
    Migration {
        version: "20251015_0007_migrate_work_sessions_to_events",
        description: "Migrate existing work_sessions rows into events (idempotent, source='migration')",
        up: migrate_to_037_migrate_work_sessions_to_events,
    },
    Migration {
        version: "20251020_0008_add_M",
        description: "Extend position CHECK to include 'M' (Mixed) and migrate existing tables if necessary",
        up: migrate_to_038_add_m,
    },
];

/// Esegui solo le migrazioni non ancora applicate
pub fn run_pending_migrations(conn: &Connection) -> Result<(), Error> {
    // Ensure base tables exist (defensive): create work_sessions and log if missing so migrations can reference them.
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS work_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            position TEXT NOT NULL DEFAULT 'O' CHECK (position IN ('O','R','H','C','M')),
            start_time TEXT NOT NULL DEFAULT '',
            lunch_break INTEGER NOT NULL DEFAULT 0,
            end_time TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            function TEXT NOT NULL,
            message TEXT NOT NULL
        );
        ",
    )?;

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
    println!();
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

pub fn migrate_to_033_rel(conn: &Connection) -> Result<(), Error> {
    let path = Config::config_file();
    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        Error::SqliteFailure(
            ffi::Error::new(1), // codice "Unknown error"
            Some(format!("Failed to read config: {}", e)),
        )
    })?;
    let mut value: Value = serde_yaml::from_str(&content).map_err(|e| {
        Error::SqliteFailure(
            ffi::Error::new(1),
            Some(format!("Failed to parse config: {}", e)),
        )
    })?;

    // If the YAML root is not a mapping (unexpected), skip the migration instead of failing.
    if let Some(obj) = value.as_mapping_mut() {
        if !obj.contains_key(Value::String("min_duration_lunch_break".to_string())) {
            obj.insert(
                Value::String("min_duration_lunch_break".to_string()),
                Value::Number(30.into()),
            );
        }
        if !obj.contains_key(Value::String("max_duration_lunch_break".to_string())) {
            obj.insert(
                Value::String("max_duration_lunch_break".to_string()),
                Value::Number(90.into()),
            );
        }
    } else {
        println!(
            "⚠️  Config file exists but is not a mapping; skipping config migration (20250919_0003)"
        );
        return Ok(());
    }

    let new_yaml = serde_yaml::to_string(&value).map_err(|e| {
        Error::SqliteFailure(
            ffi::Error::new(1),
            Some(format!("Failed to serialize config: {}", e)),
        )
    })?;

    fs::write(&path, new_yaml).map_err(|e| {
        Error::SqliteFailure(
            ffi::Error::new(1),
            Some(format!("Failed to write config: {}", e)),
        )
    })?;

    db::ttlog(
        conn,
        "migrate_to_033_rel",
        "Migration configuration file completed.",
    )?;
    println!("✅ Config file migrated: {:?}", path);

    Ok(())
}

fn migrate_to_034_rel(conn: &Connection) -> rusqlite::Result<()> {
    // Create indexes to speed up queries filtering by date and position, but only if the
    // `work_sessions` table exists in the database (avoids 'no such table' errors).
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='work_sessions'")?;
    let exists: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;
    if exists.is_some() {
        conn.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_work_sessions_date ON work_sessions(date);
            CREATE INDEX IF NOT EXISTS idx_work_sessions_position ON work_sessions(position);
            ",
        )?;
    } else {
        println!("⚠️  work_sessions table not found; skipping index creation");
    }

    db::ttlog(
        conn,
        "migrate_to_034_rel",
        "Added indexes idx_work_sessions_date and idx_work_sessions_position",
    )?;
    println!("✅ Created indexes for work_sessions (date, position)");
    Ok(())
}

fn migrate_to_035_rel(conn: &Connection) -> rusqlite::Result<()> {
    // Ensure config file exists and add separator_char if missing
    use serde_yaml::Value;
    let path = Config::config_file();
    if !path.exists() {
        // nothing to do
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let mut value: Value = serde_yaml::from_str(&content)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    if let Some(map) = value.as_mapping_mut() {
        let key = Value::String("separator_char".to_string());
        if !map.contains_key(&key) {
            map.insert(key.clone(), Value::String("-".to_string()));
            // write back
            let new_yaml = serde_yaml::to_string(&map)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            std::fs::write(&path, new_yaml)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            db::ttlog(
                conn,
                "migrate_to_035_rel",
                "Inserted separator_char into config file",
            )?;
            println!("✅ Config file updated with separator_char: {:?}", path);
        }
    }

    Ok(())
}

fn migrate_to_036_create_events(conn: &Connection) -> rusqlite::Result<()> {
    // Create a flexible events table that stores in/out punches, associated position and an optional lunch value
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            time TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('in','out')),
            position TEXT NOT NULL DEFAULT 'O' CHECK(position IN ('O','R','H','C','M')),
            lunch_break INTEGER NOT NULL DEFAULT 0,
            source TEXT NOT NULL DEFAULT 'cli',
            meta TEXT DEFAULT '',
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_events_date_time ON events(date, time);
        CREATE INDEX IF NOT EXISTS idx_events_date_kind ON events(date, kind);
        ",
    )?;

    db::ttlog(conn, "migrate_to_036_create_events", "Created events table")?;
    println!("✅ Created events table");

    Ok(())
}

fn migrate_to_037_migrate_work_sessions_to_events(conn: &Connection) -> rusqlite::Result<()> {
    // Idempotent migration: for each work_sessions row, insert corresponding in/out events
    // only if they don't already exist in events. Mark source='migration'.
    let mut select_ws = conn.prepare(
        "SELECT id, date, position, start_time, lunch_break, end_time FROM work_sessions",
    )?;
    let ws_rows = select_ws.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i32>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    let mut inserted = 0i32;
    for r in ws_rows {
        let (_id, date, position, start_time, lunch_break, end_time) = r?;

        // Insert start event if present and not already migrated
        if !start_time.trim().is_empty() {
            let exists: Option<i32> = conn
                .query_row(
                    "SELECT id FROM events WHERE date = ?1 AND time = ?2 AND kind = 'in' AND source = 'migration' LIMIT 1",
                    params![&date, &start_time],
                    |row| row.get(0),
                )
                .optional()?;
            if exists.is_none() {
                conn.execute(
                    "INSERT INTO events (date, time, kind, position, lunch_break, source, meta, created_at) VALUES (?1, ?2, 'in', ?3, 0, 'migration', '', ?4)",
                    params![&date, &start_time, &position, Utc::now().to_rfc3339()],
                )?;
                inserted += 1;
            }
        }

        // Insert end event if present and not already migrated
        if !end_time.trim().is_empty() {
            let exists: Option<i32> = conn
                .query_row(
                    "SELECT id FROM events WHERE date = ?1 AND time = ?2 AND kind = 'out' AND source = 'migration' LIMIT 1",
                    params![&date, &end_time],
                    |row| row.get(0),
                )
                .optional()?;
            if exists.is_none() {
                conn.execute(
                    "INSERT INTO events (date, time, kind, position, lunch_break, source, meta, created_at) VALUES (?1, ?2, 'out', ?3, ?4, 'migration', '', ?5)",
                    params![&date, &end_time, &position, lunch_break, Utc::now().to_rfc3339()],
                )?;
                inserted += 1;
            }
        }
    }

    let msg = format!("migrated {} event(s) from work_sessions", inserted);
    db::ttlog(conn, "migrate_to_037_migrate_work_sessions_to_events", &msg)?;
    println!("✅ {}", msg);

    Ok(())
}

fn migrate_to_038_add_m(conn: &Connection) -> rusqlite::Result<()> {
    // Detect work_sessions table with old CHECK and rewrite to include 'M'
    let mut stmt =
        conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='work_sessions'")?;
    let table_sql: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;

    if let Some(sql) = table_sql {
        // if the existing table still has C but not M, migrate
        if sql.contains("CHECK(position IN ('O','R','H','C'))")
            || sql.contains("CHECK(position IN ('O','R','H','C')")
        {
            println!(
                "⚠️  Old schema detected, migrating work_sessions to support 'M' (Mixed) and updating events table if needed..."
            );

            conn.execute_batch(
                "
                ALTER TABLE work_sessions RENAME TO work_sessions_old;

                CREATE TABLE work_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    position TEXT NOT NULL CHECK(position IN ('O','R','H','C','M')),
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

            // Also update events table if present and missing 'M'
            let mut stmt_ev =
                conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='events'")?;
            let ev_sql: Option<String> = stmt_ev.query_row([], |row| row.get(0)).optional()?;
            if let Some(esql) = ev_sql
                && (esql.contains("CHECK(position IN ('O','R','H','C'))")
                    || esql.contains("CHECK(position IN ('O','R','H','C')"))
            {
                conn.execute_batch(
                    "
                        ALTER TABLE events RENAME TO events_old;

                        CREATE TABLE events (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            date TEXT NOT NULL,
                            time TEXT NOT NULL,
                            kind TEXT NOT NULL CHECK(kind IN ('in','out')),
                            position TEXT NOT NULL DEFAULT 'O' CHECK(position IN ('O','R','H','C','M')),
                            lunch_break INTEGER NOT NULL DEFAULT 0,
                            source TEXT NOT NULL DEFAULT 'cli',
                            meta TEXT DEFAULT '',
                            created_at TEXT NOT NULL
                        );

                        INSERT INTO events (id, date, time, kind, position, lunch_break, source, meta, created_at)
                        SELECT id, date, time, kind, position, lunch_break, source, meta, created_at FROM events_old;

                        DROP TABLE events_old;
                        "
                )?;
            }

            db::ttlog(
                conn,
                "migrate_to_038_add_m",
                "Migration to add 'M' to position CHECK completed.",
            )?;
            println!("✅ Migration completed: added 'M' to position checks.");
        }
    }
    Ok(())
}
