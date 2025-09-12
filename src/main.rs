use r_timelog::db;
use r_timelog::logic;

use chrono::{NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use r_timelog::logic::month_name;
use rusqlite::Connection;

/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(name = "rTimelog")]
#[command(version = "0.1.2")]
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
    List {
        /// Filter by year (e.g., 2025) or year-month (e.g., 2025-09)
        #[arg(long, short)]
        period: Option<String>,
    },
}

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open("worktime.db")?;

    println!();

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

        Commands::List { period } => {
            let conn = Connection::open("worktime.db")?;
            let sessions = db::list_sessions(&conn, period.as_deref())?;

            if let Some(p) = period {
                if p.len() == 4 {
                    println!("📅 Saved sessions for year {}:", p);
                } else if p.len() == 7 {
                    let parts: Vec<&str> = p.split('-').collect();
                    let year = parts[0];
                    let month = parts[1];
                    println!("📅 Saved sessions for {} {}:", month_name(month), year);
                }
            } else {
                println!("📅 Saved sessions:");
            }

            for s in sessions {
                let expected = logic::calculate_expected_exit(&s.start, s.lunch);
                let surplus = logic::calculate_surplus(&s.start, s.lunch, &s.end);
                let surplus_minutes = surplus.num_minutes();

                let color_code = if surplus_minutes < 0 {
                    "\x1b[31m"
                } else if surplus_minutes > 0 {
                    "\x1b[32m"
                } else {
                    "\x1b[0m"
                };

                let formatted_surplus = if surplus_minutes == 0 {
                    "0".to_string()
                } else {
                    format!("{:+}", surplus_minutes)
                };

                println!(
                    "{:>3}: {} | Start {} | Lunch {:02} min | End {} | Expected {} | Surplus {}{:>4} min\x1b[0m",
                    s.id,
                    s.date,
                    s.start,
                    s.lunch,
                    s.end,
                    expected.format("%H:%M"),
                    color_code,
                    formatted_surplus
                );
            }
        }
    }

    Ok(())
}
