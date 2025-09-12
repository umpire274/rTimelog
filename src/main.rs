use r_timelog::db;
use r_timelog::logic;

use chrono::{NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(name = "rTimelog")]
#[command(version = "0.1.1")]
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
        /// Date of the session (YYYY-MM-DD)
        #[arg(index = 1)]
        date: String,

        /// Start time (HH:MM)
        #[arg(index = 2)]
        start: String,

        /// Lunch break in minutes (30–90)
        #[arg(index = 3)]
        lunch: u32,

        /// End time (HH:MM)
        #[arg(index = 4)]
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
                let surplus_minutes = surplus.num_minutes();

                // Pick color depending on surplus
                let color_code = if surplus_minutes < 0 {
                    "\x1b[31m" // red
                } else if surplus_minutes > 0 {
                    "\x1b[32m" // green
                } else {
                    "\x1b[0m" // default (no color)
                };

                println!(
                    "{:>3}: {} | Start {} | Lunch {:>2} min | End {} | Expected {} | Surplus {}{:>4} \x1b[0mmin",
                    s.id,
                    s.date,
                    s.start,
                    s.lunch,
                    s.end,
                    expected.format("%H:%M"),
                    color_code,
                    surplus_minutes
                );
            }
        }
    }

    Ok(())
}
