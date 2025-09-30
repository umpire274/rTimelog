use chrono::Utc;
use rusqlite::{Connection, Result, ToSql, params};
mod migrate;
pub use migrate::run_pending_migrations;

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
            position     TEXT NOT NULL DEFAULT 'O' CHECK (position IN ('O','R','H','C')),
            start_time   TEXT NOT NULL DEFAULT '',
            lunch_break  INTEGER NOT NULL DEFAULT 0,
            end_time     TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            function TEXT NOT NULL,
            message TEXT NOT NULL
        );
        ",
    )?;
    run_pending_migrations(conn)?;
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

pub fn delete_session(conn: &Connection, id: i32) -> Result<usize> {
    conn.execute("DELETE FROM work_sessions WHERE id = ?", [id])
}

/// Return all saved work sessions, optionally filtered by year or year-month.
pub fn list_sessions(
    conn: &Connection,
    period: Option<&str>,
    pos: Option<&str>,
) -> Result<Vec<WorkSession>> {
    let mut query = String::from(
        "SELECT id, date, position, start_time, lunch_break, end_time FROM work_sessions",
    );

    let mut conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(p) = period {
        if p.len() == 4 {
            conditions.push("strftime('%Y', date) = ?".to_string());
            params.push(p.to_string());
        } else if p.len() == 7 {
            conditions.push("strftime('%Y-%m', date) = ?".to_string());
            params.push(p.to_string());
        } else {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }

    if let Some(pos_filter) = pos {
        conditions.push("position = ?".to_string());
        params.push(pos_filter.to_string());
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY date ASC");

    // Use cached prepared statement to avoid recompiling the SQL on repeated calls
    let mut stmt = conn.prepare_cached(&query)?;
    let params_refs: Vec<&dyn ToSql> = params.iter().map(|s| s as &dyn ToSql).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(WorkSession {
            id: row.get(0)?,
            date: row.get(1)?,
            position: row.get(2)?,
            start: row.get(3)?,
            lunch: row.get(4)?,
            end: row.get(5)?,
        })
    })?;

    let mut sessions = Vec::new();
    for s in rows {
        sessions.push(s?);
    }
    Ok(sessions)
}

/// Insert or update the position (A=office, R=remote) for a given date.
pub fn upsert_position(conn: &Connection, date: &str, pos: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("UPDATE work_sessions SET position = ?1 WHERE date = ?2")?;
    let rows = stmt.execute(params![pos, date])?;
    if rows == 0 {
        let mut ins = conn.prepare_cached(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, ?2, '', 0, '')",
        )?;
        ins.execute(params![date, pos])?;
    }
    Ok(())
}

/// Insert or update the start time (HH:MM) for a given date.
pub fn upsert_start(conn: &Connection, date: &str, start: &str) -> Result<()> {
    let mut stmt =
        conn.prepare_cached("UPDATE work_sessions SET start_time = ?1 WHERE date = ?2")?;
    let rows = stmt.execute(params![start, date])?;
    if rows == 0 {
        let mut ins = conn.prepare_cached(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', ?2, 0, '')",
        )?;
        ins.execute(params![date, start])?;
    }
    Ok(())
}

/// Insert or update the lunch break (minutes) for a given date.
pub fn upsert_lunch(conn: &Connection, date: &str, lunch: i32) -> Result<()> {
    let mut stmt =
        conn.prepare_cached("UPDATE work_sessions SET lunch_break = ?1 WHERE date = ?2")?;
    let rows = stmt.execute(params![lunch, date])?;
    if rows == 0 {
        let mut ins = conn.prepare_cached(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', '', ?2, '')",
        )?;
        ins.execute(params![date, lunch])?;
    }
    Ok(())
}

/// Insert or update the end time (HH:MM) for a given date.
pub fn upsert_end(conn: &Connection, date: &str, end: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("UPDATE work_sessions SET end_time = ?1 WHERE date = ?2")?;
    let rows = stmt.execute(params![end, date])?;
    if rows == 0 {
        let mut ins = conn.prepare_cached(
            "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
             VALUES (?1, 'A', '', 0, ?2)",
        )?;
        ins.execute(params![date, end])?;
    }
    Ok(())
}

pub fn ttlog(conn: &Connection, function: &str, message: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339(); // ISO 8601
    let mut stmt =
        conn.prepare_cached("INSERT INTO log (date, function, message) VALUES (?1, ?2, ?3)")?;
    stmt.execute(params![&now, function, message])?;
    Ok(())
}

/// Retrieve a single work session by id
pub fn get_session(conn: &Connection, id: i32) -> Result<Option<WorkSession>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, position, start_time, lunch_break, end_time FROM work_sessions WHERE id = ?1",
    )?;

    match stmt.query_row([id], |row| {
        Ok(WorkSession {
            id: row.get(0)?,
            date: row.get(1)?,
            position: row.get(2)?,
            start: row.get(3)?,
            lunch: row.get(4)?,
            end: row.get(5)?,
        })
    }) {
        Ok(ws) => Ok(Some(ws)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
