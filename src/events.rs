use crate::config::Config;
use crate::db;
use crate::db::row_to_event;
use rusqlite::{Connection, params};

/// Create a missing event (in/out) and return the created event.
/// If an identical event (same date/time/kind) already exists, return it without duplicating.
/// `pos_opt` may force the position (otherwise uses the default from `config`).
/// `_prefer_other` is currently ignored (placeholder for future merge/matching logic).
pub fn create_missing_event(
    conn: &mut Connection,
    date: &str,
    time_val: &str,
    kind_val: &str,           // "in" | "out"
    pos_opt: &Option<String>, // Some("R") etc. or None
    _prefer_other: Option<&db::Event>,
    config: &Config,
) -> rusqlite::Result<Option<db::Event>> {
    // Normalize/validate minimal parameters
    let kind = kind_val.trim().to_lowercase();
    if kind != "in" && kind != "out" {
        // invalid kind -> no insertion
        return Ok(None);
    }

    let position = pos_opt
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| config.default_position.as_str())
        .trim()
        .to_string();

    // 1) Check whether an identical event already exists (same date, same time, same kind)
    if let Some(existing) = get_event_by_uniq(conn, date, time_val, &kind)? {
        return Ok(Some(existing));
    }

    // 2) Insert the missing event.
    //    Note: pair is left at 0 (DEFAULT) and can be recalculated later (migration/repair).
    //    created_at is set via SQLite to avoid extra dependencies in Rust.
    conn.execute(
        r#"
        INSERT INTO events (date, time, kind, position, lunch_break, pair, source, meta, created_at)
        VALUES (?1, ?2, ?3, ?4, 0, 0, 'cli', '', strftime('%Y-%m-%dT%H:%M:%S','now'))
        "#,
        params![date, time_val, kind, position],
    )?;

    // 3) Retrieve the newly created event and return it.
    let new_id: i64 = conn.query_row("SELECT last_insert_rowid()", [], |r| r.get(0))?;
    let inserted = get_event_by_id(conn, new_id)?;
    Ok(Some(inserted))
}

/// Return an event by ID (column-name based mapping, robust to column order changes).
fn get_event_by_id(conn: &Connection, id: i64) -> rusqlite::Result<db::Event> {
    conn.query_row(
        r#"
        SELECT id, date, time, kind, position, lunch_break, pair, source, meta, created_at
        FROM events
        WHERE id = ?1
        "#,
        [id],
        row_to_event,
    )
}

/// Find a unique event by (date, time, kind).
fn get_event_by_uniq(
    conn: &Connection,
    date: &str,
    time_val: &str,
    kind: &str,
) -> rusqlite::Result<Option<db::Event>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, date, time, kind, position, lunch_break, pair, source, meta, created_at
        FROM events
        WHERE date = ?1 AND time = ?2 AND kind = ?3
        LIMIT 1
        "#,
    )?;

    let mut rows = stmt.query(params![date, time_val, kind])?;
    if let Some(row) = rows.next()? {
        let ev = row_to_event(row)?;
        Ok(Some(ev))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::db;
    use rusqlite::Connection;

    /// Verify that creating a missing 'in' event on an in-memory DB works
    /// and returns an Event consistent with the provided data.
    #[test]
    fn test_create_missing_event_in_memory() {
        // prepare in-memory DB and config
        let mut conn = Connection::open_in_memory().expect("open in-memory");
        // initialize schema and migrations
        db::init_db(&conn).expect("init_db");

        let config = Config {
            database: ":memory:".to_string(),
            default_position: "O".to_string(),
            min_work_duration: "8h".to_string(),
            min_duration_lunch_break: 30,
            max_duration_lunch_break: 90,
            separator_char: "-".to_string(),
            show_weekday: "None".to_string(),
        };

        // Ensure no events initially
        let list0 = db::list_events_by_date(&conn, "2025-10-03").expect("list events");
        assert!(list0.is_empty(), "expected no events at start");

        // Call helper to create an 'in' missing event
        let pos = Some("R".to_string());
        let created =
            create_missing_event(&mut conn, "2025-10-03", "09:00", "in", &pos, None, &config)
                .expect("create_missing_event");
        assert!(created.is_some(), "expected event to be created");
        let ev = created.unwrap();
        assert_eq!(ev.kind, "in");
        assert_eq!(ev.time, "09:00");
        assert_eq!(ev.position, "R");

        // Double-check listing for that date returns exactly one event
        let list_after = db::list_events_by_date(&conn, "2025-10-03").expect("list events");
        assert_eq!(
            list_after.len(),
            1,
            "expected exactly one event after insert"
        );
        let ev0 = &list_after[0];
        assert_eq!(ev0.kind, "in");
        assert_eq!(ev0.time, "09:00");
        assert_eq!(ev0.position, "R");
    }
}
