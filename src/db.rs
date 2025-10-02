use chrono::{NaiveTime, Utc};
use rusqlite::{Connection, OptionalExtension, Result, ToSql, params};
use serde::Serialize;
mod migrate;
pub use migrate::run_pending_migrations;

/// Represents a work session entry
#[derive(Debug, Clone, Serialize)]
pub struct WorkSession {
    pub id: i32,
    pub date: String,
    pub position: String, // "A" (office) or "R" (remote)
    pub start: String,
    pub lunch: i32,
    pub end: String,
}

/// Represents a single punch event (in/out)
#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: i32,
    pub date: String,
    pub time: String,     // HH:MM
    pub kind: String,     // "in" or "out"
    pub position: String, // O,R,H,C
    pub lunch_break: i32, // minutes, typically set on out
    pub source: String,
    pub meta: String,
    pub created_at: String, // ISO timestamp
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

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,          -- YYYY-MM-DD
            time TEXT NOT NULL,          -- HH:MM
            kind TEXT NOT NULL CHECK (kind IN ('in','out')),
            position TEXT NOT NULL CHECK (position IN ('O','R','H','C')),
            lunch_break INTEGER NOT NULL DEFAULT 0, -- minutes, typically set on out
            source TEXT NOT NULL,
            meta TEXT,
            created_at TEXT NOT NULL     -- ISO 8601 timestamp
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
    // Update only if start_time is empty (preserve first start of the day)
    let mut stmt = conn.prepare_cached(
        "UPDATE work_sessions SET start_time = ?1 WHERE date = ?2 AND (start_time = '' OR start_time IS NULL)",
    )?;
    let rows = stmt.execute(params![start, date])?;
    if rows == 0 {
        // If no existing row was updated, ensure a row exists; if not, insert with provided start
        let mut check = conn.prepare_cached("SELECT id FROM work_sessions WHERE date = ?1")?;
        let exists = check.query_row([date], |_| Ok(() as ())).optional()?;
        if exists.is_none() {
            let mut ins = conn.prepare_cached(
                "INSERT INTO work_sessions (date, position, start_time, lunch_break, end_time)
                 VALUES (?1, 'A', ?2, 0, '')",
            )?;
            ins.execute(params![date, start])?;
        }
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

/// Retrieve work sessions for a specific date
pub fn list_sessions_by_date(conn: &Connection, date: &str) -> Result<Vec<WorkSession>> {
    let query = "SELECT id, date, position, start_time, lunch_break, end_time FROM work_sessions WHERE date = ?1 ORDER BY date ASC";
    let mut stmt = conn.prepare_cached(query)?;
    let rows = stmt.query_map([date], |row| {
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

/// List events for a specific date (ordered by time asc)
pub fn list_events_by_date(conn: &Connection, date: &str) -> Result<Vec<Event>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, time, kind, position, lunch_break, source, meta, created_at FROM events WHERE date = ?1 ORDER BY time ASC",
    )?;
    let rows = stmt.query_map([date], |row| {
        Ok(Event {
            id: row.get(0)?,
            date: row.get(1)?,
            time: row.get(2)?,
            kind: row.get(3)?,
            position: row.get(4)?,
            lunch_break: row.get(5)?,
            source: row.get(6)?,
            meta: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut evs = Vec::new();
    for e in rows {
        evs.push(e?);
    }
    Ok(evs)
}

/// List all events in the database ordered by date and time
pub fn list_events(conn: &Connection) -> Result<Vec<Event>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, time, kind, position, lunch_break, source, meta, created_at FROM events ORDER BY date ASC, time ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Event {
            id: row.get(0)?,
            date: row.get(1)?,
            time: row.get(2)?,
            kind: row.get(3)?,
            position: row.get(4)?,
            lunch_break: row.get(5)?,
            source: row.get(6)?,
            meta: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut evs = Vec::new();
    for e in rows {
        evs.push(e?);
    }
    Ok(evs)
}

/// List events filtered by optional period (YYYY or YYYY-MM) and position
pub fn list_events_filtered(
    conn: &Connection,
    period: Option<&str>,
    pos: Option<&str>,
) -> Result<Vec<Event>> {
    let mut query = String::from(
        "SELECT id, date, time, kind, position, lunch_break, source, meta, created_at FROM events",
    );
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(p) = period {
        if p.len() == 4 {
            // year
            conditions.push("strftime('%Y', date) = ?".to_string());
            params.push(p.to_string());
        } else if p.len() == 7 {
            // year-month
            conditions.push("strftime('%Y-%m', date) = ?".to_string());
            params.push(p.to_string());
        } else {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    if let Some(pos_filter) = pos {
        let up = pos_filter.trim().to_uppercase();
        conditions.push("position = ?".to_string());
        params.push(up);
    }
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    query.push_str(" ORDER BY date ASC, time ASC");

    let mut stmt = conn.prepare_cached(&query)?;
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|s| s as &dyn ToSql).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Event {
            id: row.get(0)?,
            date: row.get(1)?,
            time: row.get(2)?,
            kind: row.get(3)?,
            position: row.get(4)?,
            lunch_break: row.get(5)?,
            source: row.get(6)?,
            meta: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    let mut evs = Vec::new();
    for e in rows {
        evs.push(e?);
    }
    Ok(evs)
}

/// Find last out event before a given time on the same date
pub fn last_out_before(conn: &Connection, date: &str, time: &str) -> Result<Option<Event>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, time, kind, position, lunch_break, source, meta, created_at FROM events WHERE date = ?1 AND kind = 'out' AND time < ?2 ORDER BY time DESC LIMIT 1",
    )?;
    match stmt.query_row([date, time], |row| {
        Ok(Event {
            id: row.get(0)?,
            date: row.get(1)?,
            time: row.get(2)?,
            kind: row.get(3)?,
            position: row.get(4)?,
            lunch_break: row.get(5)?,
            source: row.get(6)?,
            meta: row.get(7)?,
            created_at: row.get(8)?,
        })
    }) {
        Ok(ev) => Ok(Some(ev)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Update lunch_break for a specific event (typically an 'out')
pub fn set_event_lunch(conn: &Connection, event_id: i32, lunch: i32) -> Result<()> {
    conn.execute(
        "UPDATE events SET lunch_break = ?1 WHERE id = ?2",
        params![lunch, event_id],
    )?;
    Ok(())
}

/// Insert an event and run auto-lunch logic if kind == 'in'.
/// This function uses a transaction to ensure atomicity. It also performs dual-write
/// to legacy `work_sessions` via existing upsert_* helpers to keep backwards compatibility.
pub fn add_event(
    conn: &mut Connection,
    date: &str,
    time: &str,
    kind: &str,
    position: Option<&str>,
    source: &str,
    meta: Option<&str>,
    config: &crate::config::Config,
) -> Result<i64> {
    let tx = conn.transaction()?;

    // Determine position_to_use:
    // - if user provided position (Some) -> use it
    // - else if kind == 'out' -> try to inherit from last 'in' on the same date
    // - otherwise fallback to config.default_position
    let mut position_to_use = if let Some(p) = position {
        p.to_string()
    } else {
        config.default_position.clone()
    };
    if position.is_none() && kind == "out" {
        let mut stmt = tx.prepare_cached(
            "SELECT position FROM events WHERE date = ?1 AND kind = 'in' AND time <= ?2 ORDER BY time DESC LIMIT 1",
        )?;
        if let Some(found_pos) = stmt
            .query_row([date, time], |row| row.get::<_, String>(0))
            .optional()?
        {
            position_to_use = found_pos;
        }
    }

    tx.execute(
        "INSERT INTO events (date, time, kind, position, lunch_break, source, meta, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7)",
        params![date, time, kind, position_to_use, source, meta.unwrap_or(""), Utc::now().to_rfc3339()],
    )?;

    let event_id = tx.last_insert_rowid();

    // Dual-write to legacy table to ease rollout
    if kind == "in" {
        // store start in legacy place
        let _ = upsert_start(&tx, date, time);
    } else if kind == "out" {
        let _ = upsert_end(&tx, date, time);
    }

    // If this is an 'in' event, attempt to populate lunch on the previous 'out' (auto-lunch)
    if kind == "in" {
        if let Some(prev_out) = last_out_before(&tx, date, time)? {
            // Exclude holiday positions
            if prev_out.position != "H" && position_to_use != "H" {
                // Ensure previous out has no lunch yet
                if prev_out.lunch_break == 0 {
                    // Parse times
                    if let (Ok(prev_time), Ok(new_time)) = (
                        NaiveTime::parse_from_str(&prev_out.time, "%H:%M"),
                        NaiveTime::parse_from_str(time, "%H:%M"),
                    ) {
                        let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
                        let latest = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
                        if prev_time >= noon && new_time <= latest && new_time > prev_time {
                            let delta = (new_time - prev_time).num_minutes() as i32;
                            let mut lunch_val = delta;
                            if lunch_val < config.min_duration_lunch_break {
                                lunch_val = config.min_duration_lunch_break;
                            }
                            if lunch_val > config.max_duration_lunch_break {
                                lunch_val = config.max_duration_lunch_break;
                            }
                            if lunch_val > 0 {
                                tx.execute(
                                    "UPDATE events SET lunch_break = ?1 WHERE id = ?2",
                                    params![lunch_val, prev_out.id],
                                )?;
                                // also update legacy work_sessions lunch for compatibility
                                let _ = upsert_lunch(&tx, date, lunch_val);
                                // write an audit log entry inside the same transaction
                                let msg = format!(
                                    "auto_lunch {} min for out_event {} (date={})",
                                    lunch_val, prev_out.id, date
                                );
                                tx.execute(
                                    "INSERT INTO log (date, function, message) VALUES (?1, ?2, ?3)",
                                    params![Utc::now().to_rfc3339(), "auto_lunch", msg],
                                )?;
                            }
                        }
                    }
                }
            }
        }
    }

    tx.commit()?;
    Ok(event_id)
}

/// Reconstruct work sessions from events for a given date.
/// Produces one WorkSession per matched in/out pair (or partial if unmatched).
pub fn reconstruct_sessions_from_events(conn: &Connection, date: &str) -> Result<Vec<WorkSession>> {
    let events = list_events_by_date(conn, date)?;
    let mut sessions: Vec<WorkSession> = Vec::new();

    let mut pending_in: Option<Event> = None;

    for e in events {
        if e.kind == "in" {
            // treat latest 'in' as the pending entry
            pending_in = Some(e);
        } else if e.kind == "out" {
            if let Some(in_ev) = pending_in.take() {
                // matched pair
                let ws = WorkSession {
                    id: e.id, // use out event id as session id
                    date: date.to_string(),
                    position: in_ev.position.clone(),
                    start: in_ev.time.clone(),
                    lunch: e.lunch_break,
                    end: e.time.clone(),
                };
                sessions.push(ws);
            } else {
                // out without in -> partial session
                let ws = WorkSession {
                    id: e.id,
                    date: date.to_string(),
                    position: e.position.clone(),
                    start: "".to_string(),
                    lunch: e.lunch_break,
                    end: e.time.clone(),
                };
                sessions.push(ws);
            }
        }
    }

    // any remaining pending_in -> incomplete session
    if let Some(in_ev) = pending_in {
        let ws = WorkSession {
            id: in_ev.id,
            date: date.to_string(),
            position: in_ev.position.clone(),
            start: in_ev.time.clone(),
            lunch: 0,
            end: "".to_string(),
        };
        sessions.push(ws);
    }

    Ok(sessions)
}
