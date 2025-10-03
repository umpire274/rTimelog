use crate::config::Config;
use crate::db;
use rusqlite::Connection;

/// Create a missing event (in/out) and return the created event if found.
/// This is intentionally public to make it testable from unit/integration tests.
pub fn create_missing_event(
    conn: &mut Connection,
    date: &str,
    time_val: &str,
    kind_val: &str,
    pos_opt: &Option<String>,
    prefer_other: Option<&db::Event>,
    config: &Config,
) -> rusqlite::Result<Option<db::Event>> {
    // Determine preferred position: explicit CLI pos has priority, then other event's position, then config default
    let p_norm = pos_opt.clone().unwrap_or_else(|| {
        prefer_other
            .map(|e| e.position.clone())
            .unwrap_or_else(|| config.default_position.clone())
    });

    let args = db::AddEventArgs {
        date,
        time: time_val,
        kind: kind_val,
        position: Some(p_norm.as_str()),
        source: "cli",
        meta: None,
    };

    if let Err(e) = db::add_event(conn, &args, config) {
        eprintln!("⚠️  Failed to create missing {} event: {}", kind_val, e);
    }

    let found = db::list_events_by_date(conn, date)?
        .into_iter()
        .find(|ev| ev.kind == kind_val && ev.time == time_val);
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::db;
    use rusqlite::Connection;

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
        };

        // Ensure no events initially
        let list0 = db::list_events_by_date(&conn, "2025-10-03").expect("list events");
        assert!(list0.is_empty());

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
    }
}
