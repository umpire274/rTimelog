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
            lunch_break INTEGER NOT NULL,
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
pub fn list_sessions(conn: &Connection) -> Result<Vec<WorkSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, date, start_time, lunch_break, end_time FROM work_sessions ORDER BY date ASC",
    )?;

    let sessions = stmt.query_map([], |row| {
        Ok(WorkSession {
            id: row.get(0)?,
            date: row.get(1)?,
            start: row.get(2)?,
            lunch: row.get(3)?,
            end: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();
    for session in sessions {
        result.push(session?);
    }

    Ok(result)
}
