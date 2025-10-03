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
