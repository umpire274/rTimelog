use crate::cli::Commands;
use crate::db;
use crate::utils::mins2hhmm;
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
    id: i32,
    date: String,
    position: String,
    start: String,
    lunch_break: i32,
    end: String,
    work_duration: Option<String>,
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

        let date_bounds: Option<(String, String)> = if let Some(r) = range.as_deref() {
            Some(parse_range(r).map_err(|e| format!("invalid --range: {e}"))?)
        } else {
            None
        };

        // selezione dataset (default: events)
        let export_events = if *events { true } else { !(*sessions) };

        if export_events {
            let data = load_events(conn, date_bounds)?;
            match fmt.as_str() {
                "csv" => export_csv(&data, path)?,
                "json" => export_json(&data, path)?,
                _ => unreachable!(),
            }
        } else {
            let data = load_sessions(conn, date_bounds)?;
            match fmt.as_str() {
                "csv" => export_csv(&data, path)?,
                "json" => export_json(&data, path)?,
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

fn load_events(
    conn: &Connection,
    bounds: Option<(String, String)>,
) -> rusqlite::Result<Vec<EventExport>> {
    let mut sql = String::from(
        r#"
        SELECT id, date, time, kind, position, lunch_break, pair, source, meta, created_at
        FROM events
        "#,
    );
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some((start, end)) = bounds {
        sql.push_str(" WHERE date BETWEEN ?1 AND ?2");
        params.push(&start);
        params.push(&end);
    }
    sql.push_str(" ORDER BY date, time");

    let mut stmt = conn.prepare(&sql)?;
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

fn load_sessions(
    conn: &Connection,
    bounds: Option<(String, String)>,
) -> rusqlite::Result<Vec<SessionExport>> {
    let mut sql = String::from(
        r#"
        SELECT
          id,
          date,
          position,
          start_time,
          COALESCE(lunch_break, 0) AS lunch_break,
          end_time
        FROM work_sessions
        "#,
    );
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some((start, end)) = bounds {
        sql.push_str(" WHERE date BETWEEN ?1 AND ?2");
        params.push(&start);
        params.push(&end);
    }
    sql.push_str(" ORDER BY date, start_time");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        db::row_to_worksession(row).map(|ws| SessionExport {
            id: ws.id,
            date: ws.date,
            position: ws.position,
            start: ws.start,
            lunch_break: ws.lunch,
            end: ws.end,
            work_duration: ws.work_duration.map(mins2hhmm),
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
        Err("Export cancelled: existing file not overwritten"
            .to_string()
            .into())
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

fn parse_range(range: &str) -> Result<(String, String), String> {
    // YYYY
    if range.len() == 4 && range.chars().all(|c| c.is_ascii_digit()) {
        let y = range.to_string();
        return Ok((format!("{y}-01-01"), format!("{y}-12-31")));
    }

    // YYYY-MM
    if range.len() == 7 && &range[4..5] == "-" {
        let y: i32 = range[0..4].parse().map_err(|_| "invalid year")?;
        let m: u32 = range[5..7].parse().map_err(|_| "invalid month")?;
        let last = month_last_day(y, m).ok_or("invalid month in range")?;
        return Ok((format!("{y}-{m:02}-01"), format!("{y}-{m:02}-{last:02}")));
    }

    // YYYY-MM-{dd..dd}
    if range.len() >= 15
        && &range[4..5] == "-"
        && range.contains("..")
        && range.contains('{')
        && range.ends_with('}')
    {
        let y: i32 = range[0..4].parse().map_err(|_| "invalid year")?;
        let m: u32 = range[5..7].parse().map_err(|_| "invalid month")?;
        let inside = &range[8..]; // expected "{dd..dd}"
        if !(inside.starts_with('{') && inside.ends_with('}')) {
            return Err("invalid day range brace".into());
        }
        let inner = &inside[1..inside.len() - 1]; // "dd..dd"
        let parts: Vec<&str> = inner.split("..").collect();
        if parts.len() != 2 {
            return Err("invalid day range syntax".into());
        }
        let d1: u32 = parts[0].parse().map_err(|_| "invalid start day")?;
        let d2: u32 = parts[1].parse().map_err(|_| "invalid end day")?;
        let last = month_last_day(y, m).ok_or("invalid month in range")?;
        if d1 == 0 || d2 == 0 || d1 > d2 || d2 > last {
            return Err("day range out of bounds".into());
        }
        return Ok((format!("{y}-{m:02}-{d1:02}"), format!("{y}-{m:02}-{d2:02}")));
    }

    Err("unsupported --range format (use YYYY, YYYY-MM, or YYYY-MM-{dd..dd})".into())
}

fn month_last_day(y: i32, m: u32) -> Option<u32> {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => {
            let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
            Some(if leap { 29 } else { 28 })
        }
        _ => None,
    }
}
