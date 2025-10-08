use crate::cli::Commands;
use crate::db;
use rusqlite::Connection;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Clone)]
struct EventExport {
    id: i32,
    date: String,
    time: String,
    kind: String,
    position: String,
    lunch_break: i32,
    pair: i32,
    source: String,
}

#[derive(Serialize, Clone)]
struct SessionExport {
    date: String,
    position: String,
    start: Option<String>,
    lunch_break: i32,
    end: Option<String>,
    duration_min: Option<i64>,
}

/// Main export handler
pub fn handle_export(cmd: &Commands, conn: &Connection) -> Result<(), Box<dyn Error>> {
    if let Commands::Export {
        format,
        file,
        range,
        events,
        sessions,
        force,
    } = cmd
    {
        // Verifica formato
        let fmt = format.to_lowercase();
        if !["csv", "json"].contains(&fmt.as_str()) {
            eprintln!("❌ Unsupported format '{}'. Use one of: csv, json", format);
            std::process::exit(1);
        }

        // Controlla percorso file di output
        let path = Path::new(file);
        if !path.is_absolute() {
            eprintln!("❌ Output file path must be absolute: {}", file);
            std::process::exit(1);
        }

        // ⬇️ nuovo controllo
        ensure_writable(path, *force)?;

        // parse range (stub: filtro non ancora implementato)
        let _range = range.as_deref();

        // selezione dataset (default: events)
        let export_events = if *events { true } else { !(*sessions) };

        if export_events {
            let data = load_events(conn, _range)?;
            match fmt.as_str() {
                "csv" => export_csv(&data, path)?,
                "json" => export_json(&data, path)?,
                _ => unreachable!(),
            }
        } else {
            let data = load_sessions(conn, _range)?;
            match fmt.as_str() {
                "csv" => export_csv(&data, path)?,
                "json" => export_json(&data, path)?,
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

fn load_events(conn: &Connection, _range: Option<&str>) -> rusqlite::Result<Vec<EventExport>> {
    // TODO: applicare filtro range
    let mut stmt = conn.prepare(
        r#"
        SELECT id, date, time, kind, position, lunch_break, pair, source, meta, created_at
        FROM events
        ORDER BY date, time
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        db::row_to_event(row).map(|ev| EventExport {
            id: ev.id,
            date: ev.date,
            time: ev.time,
            kind: ev.kind,
            position: ev.position,
            lunch_break: ev.lunch_break,
            pair: ev.pair,
            source: ev.source,
        })
    })?;

    rows.collect()
}

fn load_sessions(conn: &Connection, _range: Option<&str>) -> rusqlite::Result<Vec<SessionExport>> {
    // Nota: questa query assume schema legacy `work_sessions`
    // con colonne: date, position, start, end, lunch (minuti).
    // La duration è calcolata in minuti quando start & end sono presenti.
    let mut stmt = conn.prepare(
        r#"
        SELECT
          date,
          position,
          start,
          COALESCE(lunch, 0) AS lunch_break,
          end,
          CASE
            WHEN start IS NOT NULL AND end IS NOT NULL
            THEN CAST((strftime('%s', end) - strftime('%s', start)) / 60 AS INTEGER)
            ELSE NULL
          END AS duration_min
        FROM work_sessions
        ORDER BY date
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SessionExport {
            date: row.get("date")?,
            position: row.get("position")?,
            start: row.get::<_, Option<String>>("start")?,
            lunch_break: row.get::<_, i32>("lunch_break")?,
            end: row.get::<_, Option<String>>("end")?,
            duration_min: row.get::<_, Option<i64>>("duration_min")?,
        })
    })?;

    rows.collect()
}

fn ensure_writable(path: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Ok(());
    }
    if force {
        return Ok(());
    }

    // Prompt interattivo
    eprint!(
        "⚠️  File '{}' esiste già. Sovrascrivere? [y/N]: ",
        path.display()
    );
    io::stderr().flush().ok();

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let ans = answer.trim().to_ascii_lowercase();

    if ans == "y" || ans == "yes" {
        Ok(())
    } else {
        Err("Export cancelled: existing file not overwritten".to_string().into())
    }
}

/// Export data as JSON
fn export_json<T: Serialize>(data: &[T], path: &Path) -> Result<(), Box<dyn Error>> {
    let json_data = serde_json::to_string_pretty(data)?;
    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())?;
    println!("✅ Exported data to {}", path.display());
    Ok(())
}

fn export_csv<T: Serialize>(data: &[T], path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for item in data {
        wtr.serialize(item)?;
    }
    wtr.flush()?;
    println!("✅ Exported data to {}", path.display());
    Ok(())
}
