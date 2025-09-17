use crate::db;
use rusqlite::{Connection, OptionalExtension};

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

pub fn check_db_and_migrate(conn: &Connection) -> rusqlite::Result<()> {
    migrate_to_030_rel(conn)?;
    migrate_to_032_rel(conn)?;

    Ok(())
}
