use rusqlite::{Connection, Result, params};

/// Represents a work session entry
pub struct WorkSession {
    pub id: i32,
    pub date: String,
    pub start: String,
    pub lunch: i32,
    pub end: String,
}

/// Initialize the database (create table if it does not exist)
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS work_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            start_time TEXT NOT NULL,
            lunch_break INTEGER NOT NULL DEFAULT 0,
            end_time TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// Insert a new work session
pub fn add_session(
    conn: &Connection,
    date: &str,
    start: &str,
    lunch: u32,
    end: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
         VALUES (?1, ?2, ?3, ?4)",
        params![date, start, lunch, end],
    )?;
    Ok(())
}

/// Retrieve all stored sessions
/// Retrieve all stored sessions, optionally filtered by month (01-12)
pub fn list_sessions(conn: &Connection, period: Option<&str>) -> Result<Vec<WorkSession>> {
    let mapper = |row: &rusqlite::Row| {
        Ok(WorkSession {
            id: row.get(0)?,
            date: row.get(1)?,
            start: row.get(2)?,
            lunch: row.get(3)?,
            end: row.get(4)?,
        })
    };

    let mut stmt;
    let rows;

    if let Some(p) = period {
        if p.len() == 4 {
            // Filter only by year
            stmt = conn.prepare(
                "SELECT id, date, start_time, lunch_break, end_time
                 FROM work_sessions
                 WHERE strftime('%Y', date) = ?1
                 ORDER BY date ASC",
            )?;
            rows = stmt.query_map([p], mapper)?;
        } else if p.len() == 7 {
            // Filter by year + month
            stmt = conn.prepare(
                "SELECT id, date, start_time, lunch_break, end_time
                 FROM work_sessions
                 WHERE strftime('%Y-%m', date) = ?1
                 ORDER BY date ASC",
            )?;
            rows = stmt.query_map([p], mapper)?;
        } else {
            return Err(rusqlite::Error::InvalidQuery); // o un Result custom con messaggio
        }
    } else {
        stmt = conn.prepare(
            "SELECT id, date, start_time, lunch_break, end_time
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

pub fn upsert_start(conn: &Connection, date: &str, start: &str) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET start_time = ?1 WHERE date = ?2",
        (start, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
             VALUES (?1, ?2, 0, '')",
            (date, start),
        )?;
    }
    Ok(())
}

pub fn upsert_lunch(conn: &Connection, date: &str, lunch: i32) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET lunch_break = ?1 WHERE date = ?2",
        (lunch, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
             VALUES (?1, '', ?2, '')",
            (date, lunch),
        )?;
    }
    Ok(())
}

pub fn upsert_end(conn: &Connection, date: &str, end: &str) -> Result<()> {
    let rows = conn.execute(
        "UPDATE work_sessions SET end_time = ?1 WHERE date = ?2",
        (end, date),
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
             VALUES (?1, '', 0, ?2)",
            (date, end),
        )?;
    }
    Ok(())
}
