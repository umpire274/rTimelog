use chrono::{NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, Result};

/// Applicazione CLI per tracciare orari di lavoro
#[derive(Parser)]
#[command(name = "worktime")]
#[command(version = "0.1.0")]
#[command(about = "Gestione orari di lavoro con SQLite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inizializza il database
    Init,

    /// Aggiunge una sessione lavorativa
    Add {
        /// Data nel formato YYYY-MM-DD
        date: String,
        /// Ora di ingresso (HH:MM)
        start: String,
        /// Durata pausa pranzo (in minuti)
        lunch: u32,
        /// Ora di uscita effettiva (HH:MM)
        end: String,
    },

    /// Mostra tutte le sessioni salvate
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Connessione al DB (file locale)
    let conn = Connection::open("worktime.db")?;

    match cli.command {
        Commands::Init => {
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
            println!("Database inizializzato.");
        }

        Commands::Add { date, start, lunch, end } => {
            // Validazione basilare (potresti renderla più robusta)
            let _ = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .expect("Formato data non valido (usa YYYY-MM-DD)");
            let _ = NaiveTime::parse_from_str(&start, "%H:%M")
                .expect("Formato ora ingresso non valido (usa HH:MM)");
            let _ = NaiveTime::parse_from_str(&end, "%H:%M")
                .expect("Formato ora uscita non valido (usa HH:MM)");

            conn.execute(
                "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
                 VALUES (?1, ?2, ?3, ?4)",
                params![date, start, lunch, end],
            )?;
            println!("Sessione salvata con successo!");
        }

        Commands::List => {
            let mut stmt = conn.prepare("SELECT id, date, start_time, lunch_break, end_time FROM work_sessions")?;
            let sessions = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?;

            println!("📅 Elenco sessioni:");
            for session in sessions {
                let (id, date, start, lunch, end) = session?;
                println!("{id}: {date} | Inizio: {start}, Pausa: {lunch} min, Uscita: {end}");
            }
        }
    }

    Ok(())
}
