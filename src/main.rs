use r_timelog::logic;
use r_timelog::db;

use chrono::{Local, NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

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

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open("worktime.db")?;

    match cli.command {
        Commands::Init => {
            db::init_db(&conn)?;
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

            db::add_session(&conn, &date, &start, lunch, &end)?;
            println!("💾 Work session saved!");
        }

        Commands::List => {
            let sessions = db::list_sessions(&conn)?;
            println!("📅 Saved sessions:");
            for s in sessions {
                let expected = logic::calculate_expected_exit(&s.start, s.lunch);
                let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);

                println!(
                    "{:>3}: {} | Start {} | Lunch {} min | End {} | Expected {} | Surplus {} min",
                    s.id,
                    s.date,
                    s.start,
                    s.lunch,
                    s.end,
                    expected.format("%H:%M"),
                    surplus.num_minutes()
                );
            }
        }
    }

    Ok(())
}
