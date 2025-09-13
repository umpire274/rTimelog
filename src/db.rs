use rusqlite::{Connection, Result, params};

/// Represents a work session entry
#[derive(Debug, Clone)]
pub struct WorkSession {
    pub id: i32,
    pub date: String,
    pub position: String, // "A" (office) or "R" (remote)
    pub start: String,
    pub lunch: i32,
    pub end: String,
}

/// Initialize the database schema.
/// Ensures table `work_sessions` exists and adds missing `position` column if required.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS work_sessions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            date         TEXT NOT NULL,          -- YYYY-MM-DD
            position     TEXT NOT NULL DEFAULT 'A' CHECK (position IN ('A','R')),
            start_time   TEXT NOT NULL DEFAULT '',
            lunch_break  INTEGER NOT NULL DEFAULT 0,
            end_time     TEXT NOT NULL DEFAULT ''
        );
        ",
    )?;
    ensure_position_column(conn)?; // migration for old databases
    Ok(())
}

/// Ensure the `position` column exists (for database migrations).
fn ensure_position_column(conn: &Connection) -> Result<()> {
    let mut has_col = false;
    let mut stmt = conn.prepare("PRAGMA table_info(work_sessions)")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?; // column name
        if name == "position" {
            has_col = true;
            break;
        }
    }
    if !has_col {
        conn.execute(
            "ALTER TABLE work_sessions ADD COLUMN position TEXT NOT NULL DEFAULT 'A' CHECK (position IN ('A','R'))",
            []
        )?;
    }
    Ok(())
}

/// Insert a new work session
pub fn add_session(
    conn: &Connection,
    date: &str,
    position: &str,
    start: &str,
    lunch: u32,
    end: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![date, position, start, lunch, end],
    )?;
    Ok(())
}

/// Return all saved work sessions, optionally filtered by year or year-month.
pub fn list_sessions(conn: &Connection, period: Option<&str>) -> Result<Vec<WorkSession>> {
    let mapper = |row: &rusqlite::Row| {
        Ok(WorkSession {
            id: row.get(0)?,
            date: row.get(1)?,
            position: row.get(2)?,
            start: row.get(3)?,
            lunch: row.get(4)?,
            end: row.get(5)?,
        })
    };

    let mut stmt;
    let rows;

    if let Some(p) = period {
        if p.len() == 4 {
            stmt = conn.prepare(
                "SELECT id, date, position, start_time, lunch_break, end_time
                 FROM work_sessions
                 WHERE strftime('%Y', date) = ?1
                 ORDER BY date ASC",
            )?;
            rows = stmt.query_map([p], mapper)?;
        } else if p.len() == 7 {
            stmt = conn.prepare(
                "SELECT id, date, position, start_time, lunch_break, end_time
                 FROM work_sessions
                 WHERE strftime('%Y-%m', date) = ?1
                 ORDER BY date ASC",
            )?;
            rows = stmt.query_map([p], mapper)?;
        } else {
            return Err(rusqlite::Error::InvalidQuery);
        }
    } else {
        stmt = conn.prepare(
            "SELECT id, date, position, start_time, lunch_break, end_time
             FROM work_sessions
             ORDER BY date ASC",
        )?;
        rows = stmt.query_map([], mapper)?;
    }

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Insert or update the position (A=office, R=remote) for a given date.
pub fn upsert_position(conn: &Connection, date: &str, pos: &str) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET position = ?1 WHERE date = ?2",
        (pos, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, ?2, '', 0, '')",
            (date, pos),
        )?;
    }
    Ok(())
}

/// Insert or update the start time (HH:MM) for a given date.
pub fn upsert_start(conn: &Connection, date: &str, start: &str) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET start_time = ?1 WHERE date = ?2",
        (start, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', ?2, 0, '')",
            (date, start),
        )?;
    }
    Ok(())
}

/// Insert or update the lunch break (minutes) for a given date.
pub fn upsert_lunch(conn: &Connection, date: &str, lunch: i32) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET lunch_break = ?1 WHERE date = ?2",
        (lunch, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', '', ?2, '')",
            (date, lunch),
        )?;
    }
    Ok(())
}

/// Insert or update the end time (HH:MM) for a given date.
pub fn upsert_end(conn: &Connection, date: &str, end: &str) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET end_time = ?1 WHERE date = ?2",
        (end, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', '', 0, ?2)",
            (date, end),
        )?;
    }
    Ok(())
}
