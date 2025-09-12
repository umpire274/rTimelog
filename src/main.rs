use chrono::{NaiveDate, NaiveTime, Local};
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result, params};

/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(name = "rTimelog")]
#[command(version = "0.1.0")]
#[command(about = "Track working hours and calculate surplus using SQLite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database
    Init,
    /// Add a new work session
    Add {
        /// Date (YYYY-MM-DD, default: today)
        #[arg(default_value_t = Local::now().format("%Y-%m-%d").to_string())]
        date: String,
        /// Start time (HH:MM)
        start: String,
        /// Lunch break duration (minutes)
        lunch: u32,
        /// End time (HH:MM)
        end: String,
    },
    /// List all work sessions
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // DB connection (file local)
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
            println!("✅ Database initialized.");
        }

        Commands::Add {
            date,
            start,
            lunch,
            end,
        } => {
            // Basic input validation
            NaiveDate::parse_from_str(&date, "%Y-%m-%d").expect("Invalid date format (YYYY-MM-DD)");
            NaiveTime::parse_from_str(&start, "%H:%M").expect("Invalid start time format (HH:MM)");
            NaiveTime::parse_from_str(&end, "%H:%M").expect("Invalid end time format (HH:MM)");

            conn.execute(
                "INSERT INTO work_sessions (date, start_time, lunch_break, end_time)
                 VALUES (?1, ?2, ?3, ?4)",
                params![date, start, lunch, end],
            )?;
            println!("💾 Work session saved!");
        }

        Commands::List => {
            let mut stmt = conn
                .prepare("SELECT id, date, start_time, lunch_break, end_time FROM work_sessions")?;
            let sessions = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?;

            println!("📅 Saved sessions:");
            for session in sessions {
                let (id, date, start, lunch, end) = session?;
                println!("{id}: {date} | Inizio: {start}, Pausa: {lunch} min, Uscita: {end}");
            }
        }
    }

    Ok(())
}
