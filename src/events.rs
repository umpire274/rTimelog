use crate::config::Config;
use crate::db;
use rusqlite::{Connection, params};

/// Crea un evento mancante (in/out) e restituisce l'evento creato.
/// Se un evento identico (stessa data/time/kind) esiste già, lo restituisce senza duplicare.
/// `pos_opt` può forzare la position (altrimenti usa il default da `config`).
/// `prefer_other` è attualmente ignorato (placeholder per logiche future di fusione/accoppiamento).
pub fn create_missing_event(
    conn: &mut Connection,
    date: &str,
    time_val: &str,
    kind_val: &str,           // "in" | "out"
    pos_opt: &Option<String>, // Some("R") ecc. oppure None
    _prefer_other: Option<&db::Event>,
    config: &Config,
) -> rusqlite::Result<Option<db::Event>> {
    // Normalizza/valida i parametri minimi
    let kind = kind_val.trim().to_lowercase();
    if kind != "in" && kind != "out" {
        // kind non valido → nessun inserimento
        return Ok(None);
    }

    let position = pos_opt
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| config.default_position.as_str())
        .trim()
        .to_string();

    // 1) Verifica se esiste già un evento identico (stessa data, stessa ora, stesso kind)
    if let Some(existing) = get_event_by_uniq(conn, date, time_val, &kind)? {
        return Ok(Some(existing));
    }

    // 2) Inserisci l'evento mancante.
    //    Nota: pair viene lasciato a 0 (DEFAULT) e potrà essere ricalcolato a valle (migrazione/repair).
    //    created_at viene impostato via SQLite per evitare dipendenze lato Rust.
    conn.execute(
        r#"
        INSERT INTO events (date, time, kind, position, lunch_break, pair, source, meta, created_at)
        VALUES (?1, ?2, ?3, ?4, 0, 0, 'cli', '', strftime('%Y-%m-%dT%H:%M:%S','now'))
        "#,
        params![date, time_val, kind, position],
    )?;

    // 3) Recupera l'evento appena creato e restituiscilo.
    let new_id: i64 = conn.query_row("SELECT last_insert_rowid()", [], |r| r.get(0))?;
    let inserted = get_event_by_id(conn, new_id)?;
    Ok(Some(inserted))
}

/// Restituisce un evento per ID (mapping per nome colonna, robusto all'ordine delle colonne).
fn get_event_by_id(conn: &Connection, id: i64) -> rusqlite::Result<db::Event> {
    conn.query_row(
        r#"
        SELECT id, date, time, kind, position, lunch_break, pair, source, meta, created_at
        FROM events
        WHERE id = ?1
        "#,
        [id],
        map_event_row,
    )
}

/// Cerca un evento "unico" per (date, time, kind).
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
        let ev = map_event_row(row)?;
        Ok(Some(ev))
    } else {
        Ok(None)
    }
}

/// Mapping sicuro: legge sempre per **nome colonna**.
fn map_event_row(row: &rusqlite::Row) -> rusqlite::Result<db::Event> {
    Ok(db::Event {
        id: row.get("id")?,
        date: row.get("date")?,
        time: row.get("time")?,
        kind: row.get("kind")?,
        position: row.get("position")?,
        lunch_break: row.get("lunch_break")?,
        pair: row.get("pair")?,
        source: row.get("source")?,
        meta: row.get("meta")?,
        created_at: row.get("created_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::db;
    use rusqlite::Connection;

    /// Verifica che la creazione dell'evento mancante "in" su DB in-memory funzioni
    /// e restituisca un Event coerente con i dati attesi.
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
