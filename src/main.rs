use r_timelog::db;
use r_timelog::logic;

use chrono::{NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use r_timelog::logic::month_name;
use rusqlite::Connection;

/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(name = "rTimelog")]
#[command(version = "0.1.5")]
#[command(about = "Track working hours and calculate surplus using SQLite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add or update a work session
    Add {
        /// The date (YYYY-MM-DD)
        date: String,

        /// Start time (HH:MM)
        start_pos: Option<String>,

        /// Lunch break in minutes (30–90)
        lunch_pos: Option<i32>,

        /// End time (HH:MM) - optional positional arg
        end_pos: Option<String>,

        /// Start time (HH:MM) via option
        #[arg(long = "in")]
        start: Option<String>,

        /// Lunch break in minutes via option
        #[arg(long = "lunch")]
        lunch: Option<i32>,

        /// End time (HH:MM) via option
        #[arg(long = "out")]
        end: Option<String>,
    },

    /// List sessions
    List {
        #[arg(long, short)]
        period: Option<String>,
    },

    Init,
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
            start_pos,
            lunch_pos,
            end_pos,
            start,
            lunch,
            end,
        } => {
            let conn = Connection::open("worktime.db")?;

            // ✅ Data
            if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
                eprintln!("❌ Invalid date format: {} (expected YYYY-MM-DD)", date);
                return Ok(());
            }

            // Unifica posizionali e opzioni
            let start = start.or(start_pos);
            let lunch = lunch.or(lunch_pos);
            let end = end.or(end_pos);

            // ⏰ Start
            if let Some(s) = start.as_ref() {
                if NaiveTime::parse_from_str(s, "%H:%M").is_err() {
                    eprintln!("❌ Invalid start time: {} (expected HH:MM)", s);
                    return Ok(());
                }
                db::upsert_start(&conn, &date, s)?;
                println!("✅ Start time {} registered for {}", s, date);
            }

            // 🍽 Lunch
            if let Some(l) = lunch {
                if !(30..=90).contains(&l) {
                    eprintln!("❌ Invalid lunch break: {} (must be 30–90 minutes)", l);
                    return Ok(());
                }
                db::upsert_lunch(&conn, &date, l)?;
                println!("✅ Lunch {} min registered for {}", l, date);
            }

            // 🏁 End
            if let Some(e) = end.as_ref() {
                if NaiveTime::parse_from_str(e, "%H:%M").is_err() {
                    eprintln!("❌ Invalid end time: {} (expected HH:MM)", e);
                    return Ok(());
                }
                db::upsert_end(&conn, &date, e)?;
                println!("✅ End time {} registered for {}", e, date);
            }

            // 🚨 Nessun parametro
            if start.is_none() && lunch.is_none() && end.is_none() {
                eprintln!("⚠️ Please provide at least one of: start, lunch, end");
            }

            return Ok(());
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
                let has_start = !s.start.trim().is_empty();
                let has_end = !s.end.trim().is_empty();

                if has_start && has_end {
                    let start_time = NaiveTime::parse_from_str(&s.start, "%H:%M").unwrap();
                    let end_time = NaiveTime::parse_from_str(&s.end, "%H:%M").unwrap();

                    let crosses_lunch = logic::crosses_lunch_window(&s.start, &s.end);

                    // pausa da usare nel calcolo
                    let effective_lunch = if crosses_lunch {
                        if s.lunch > 0 {
                            s.lunch
                        } else {
                            30 // pausa minima automatica
                        }
                    } else {
                        0
                    };

                    if crosses_lunch && effective_lunch > 0 {
                        // ✅ Caso completo con pausa (inserita o automatica)
                        let expected = logic::calculate_expected_exit(&s.start, effective_lunch);
                        let surplus = logic::calculate_surplus(&s.start, effective_lunch, &s.end);
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
                            "{:>3}: {} | Start {} | Lunch {:02} min | End {} | Expected {} | Surplus {}{:>3} min\x1b[0m",
                            s.id,
                            s.date,
                            s.start,
                            effective_lunch,
                            s.end,
                            expected.format("%H:%M"),
                            color_code,
                            formatted_surplus
                        );
                    } else {
                        // ✅ Caso senza pausa (lavoro interamente fuori dalla finestra)
                        let duration = end_time - start_time;
                        println!(
                            "{:>3}: {} | Start {} | \x1b[90mLunch   -\x1b[0m    | End {} | \x1b[36mWorked {:>2} h {:02} min\x1b[0m",
                            s.id,
                            s.date,
                            s.start,
                            s.end,
                            duration.num_hours(),
                            duration.num_minutes() % 60
                        );
                    }
                } else {
                    // ⚠️ informazioni incomplete
                    println!(
                        "{:>3}: {} | Start {} | Lunch {} | End {}",
                        s.id,
                        s.date,
                        if has_start { &s.start } else { "-" },
                        if s.lunch > 0 {
                            format!("{} min", s.lunch)
                        } else {
                            "-".to_string()
                        },
                        if has_end { &s.end } else { "-" }
                    );
                }
            }

            return Ok(());
        }
    }

    Ok(())
}
