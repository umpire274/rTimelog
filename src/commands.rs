use crate::Cli;
use crate::Commands;
use chrono::NaiveTime;
use r_timelog::config::Config;
use r_timelog::utils::{describe_position, mins2hhmm, print_separator};
use r_timelog::{db, logic, utils};
use rusqlite::Connection;
use std::process::Command;

pub fn handle_conf(cmd: &Commands) -> rusqlite::Result<()> {
    if let Commands::Conf {
        print_config,
        edit_config,
        editor,
    } = cmd
    {
        if *print_config {
            let config = Config::load();
            println!("📄 Current configuration:");
            println!("{}", serde_yaml::to_string(&config).unwrap());
        }

        if *edit_config {
            let path = Config::config_file();

            // Editor richiesto dall'utente (se esiste)
            let requested_editor = editor.clone();

            // Editor di default in base alla piattaforma
            let default_editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(target_os = "windows") {
                        "notepad".to_string()
                    } else {
                        "nano".to_string()
                    }
                });

            // Usa quello richiesto se possibile, altrimenti fallback
            let editor_to_use = requested_editor.unwrap_or_else(|| default_editor.clone());

            let status = Command::new(&editor_to_use).arg(&path).status();

            match status {
                Ok(s) if s.success() => {
                    println!(
                        "✅ Configuration file edited successfully with '{}'",
                        editor_to_use
                    );
                }
                Ok(_) | Err(_) => {
                    eprintln!(
                        "⚠️  Editor '{}' not available, falling back to '{}'",
                        editor_to_use, default_editor
                    );
                    // Riprova col default
                    let fallback_status = Command::new(&default_editor).arg(&path).status();
                    match fallback_status {
                        Ok(s) if s.success() => {
                            println!(
                                "✅ Configuration file edited successfully with fallback '{}'",
                                default_editor
                            );
                        }
                        Ok(_) | Err(_) => {
                            eprintln!(
                                "❌ Failed to edit configuration file with fallback '{}'",
                                default_editor
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle the `init` command
pub fn handle_init(cli: &Cli, db_path: &str) -> rusqlite::Result<()> {
    if let Some(custom) = &cli.db {
        Config::init_all(Some(custom.clone()), cli.test).unwrap();
    } else {
        Config::init_all(None, cli.test).unwrap();
    }

    if cli.test {
        // In test mode, use db_path directly
        let conn = Connection::open(db_path)?;
        // Initialize DB (creates tables) and run pending migrations
        db::init_db(&conn)?;
        println!("✅ Test database initialized at {}", db_path);
    } else {
        // Production mode: reload config
        let config = Config::load();
        let conn = Connection::open(&config.database)?;
        // Initialize DB (creates tables) and run pending migrations
        db::init_db(&conn)?;
        println!("✅ Database initialized at {}", config.database);
    }

    Ok(())
}

pub fn handle_del(cmd: &Commands, conn: &Connection) -> rusqlite::Result<()> {
    if let Commands::Del { id } = cmd {
        match db::delete_session(conn, *id) {
            Ok(rows) => {
                if rows > 0 {
                    println!("🗑️  Session with ID {} deleted", id);
                } else {
                    println!("⚠️  No session found with ID {}", id);
                }
            }
            Err(e) => eprintln!("❌ Error deleting session: {}", e),
        }
    }
    Ok(())
}

/// Handle the `add` command
pub fn handle_add(cmd: &Commands, conn: &Connection, config: &Config) -> rusqlite::Result<()> {
    if let Commands::Add {
        date,
        pos_pos,
        start_pos,
        lunch_pos,
        end_pos,
        pos,
        start,
        lunch,
        end,
    } = cmd
    {
        // validate date
        if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
            eprintln!("❌ Invalid date format: {} (expected YYYY-MM-DD)", date);
            return Ok(());
        }

        // merge positional and option values
        let pos = pos.clone().or(pos_pos.clone());
        let start = start.clone().or(start_pos.clone());
        let lunch = (*lunch).or(*lunch_pos);
        let end = end.clone().or(end_pos.clone());

        // Handle position
        if let Some(p) = pos.as_ref() {
            let p = p.trim().to_uppercase();
            if p != "O" && p != "R" && p != "H" && p != "C" {
                eprintln!(
                    "❌ Invalid position: {} (use O=office or R=remote or H=Holiday or C=On-Site)",
                    p
                );
                return Ok(());
            }
            db::upsert_position(conn, date, &p)?;
            let (pos_string, _) = describe_position(&p);
            println!("✅ Position {} set for {}", pos_string, date);
        }

        // Handle start time
        if let Some(s) = start.as_ref() {
            if NaiveTime::parse_from_str(s, "%H:%M").is_err() {
                eprintln!("❌ Invalid start time: {} (expected HH:MM)", s);
                return Ok(());
            }
            db::upsert_start(conn, date, s)?;
            println!("✅ Start time {} registered for {}", s, date);
        }

        // Handle lunch
        if let Some(l) = lunch {
            if !(0..=90).contains(&l) {
                eprintln!(
                    "❌ Invalid lunch break: {} (must be between 0 and 90 minutes)",
                    l
                );
                return Ok(());
            }
            db::upsert_lunch(conn, date, l)?;
            println!("✅ Lunch {} min registered for {}", l, date);
        }

        // Handle end time
        if let Some(e) = end.as_ref() {
            if NaiveTime::parse_from_str(e, "%H:%M").is_err() {
                eprintln!("❌ Invalid end time: {} (expected HH:MM)", e);
                return Ok(());
            }
            db::upsert_end(conn, date, e)?;
            println!("✅ End time {} registered for {}", e, date);
        }

        // Warn if no field provided
        if pos.is_none() && start.is_none() && lunch.is_none() && end.is_none() {
            eprintln!("⚠️ Please provide at least one of: position, start, lunch, end");
        }

        // Recupera l'id dell'ultima sessione per la data fornita e invoca la stampa dettagliata
        match conn.prepare("SELECT id FROM work_sessions WHERE date = ?1 ORDER BY id DESC LIMIT 1")
        {
            Ok(mut stmt) => match stmt.query_row([date], |row| row.get::<_, i32>(0)) {
                Ok(last_id) => {
                    // Use the provided connection and config to print the updated record
                    let _ = handle_list_with_highlight(None, None, conn, config, Some(last_id));
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    // nessuna sessione da stampare
                }
                Err(e) => {
                    eprintln!(
                        "❌ Errore durante il recupero dell'id della sessione: {}",
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "❌ Impossibile preparare la query per recuperare l'id: {}",
                    e
                );
            }
        }
    }

    Ok(())
}

/// Compatibile: wrapper che mantiene la firma esistente e chiama la versione con highlight = None
pub fn handle_list(
    period: Option<String>,
    pos: Option<String>,
    conn: &Connection,
    config: &Config,
) -> rusqlite::Result<()> {
    handle_list_with_highlight(period, pos, conn, config, None)
}

/// Nuova versione: supporta la stampa con `highlight_id: Option<i32`
pub fn handle_list_with_highlight(
    period: Option<String>,
    pos: Option<String>,
    conn: &Connection,
    config: &Config,
    highlight_id: Option<i32>,
) -> rusqlite::Result<()> {
    // Normalize pos to uppercase
    let pos_upper = pos.as_ref().map(|p| p.trim().to_uppercase());

    // If highlight_id is Some(id) -> retrieve only that session (efficient single-row query).
    // Otherwise, retrieve the full list based on filters.
    let sessions = if let Some(id) = highlight_id {
        match db::get_session(conn, id)? {
            Some(s) => vec![s],
            None => Vec::new(),
        }
    } else {
        db::list_sessions(conn, period.as_deref(), pos_upper.as_deref())?
    };

    if sessions.is_empty() {
        if highlight_id.is_some() {
            println!("⚠️  No recorded session found with the requested id");
        } else {
            println!("⚠️  No recorded sessions found");
        }
        return Ok(());
    }

    if highlight_id.is_none() {
        if let Some(p) = period {
            if p.len() == 4 {
                println!("📅 Saved sessions for year {}:", p);
            } else if p.len() == 7 {
                let parts: Vec<&str> = p.split('-').collect();
                let year = parts[0];
                let month = parts[1];
                println!(
                    "📅 Saved sessions for {} {}:",
                    logic::month_name(month),
                    year
                );
            }
        } else if let Some(p) = pos.as_deref() {
            println!("📅 Saved sessions for position {}:", p);
        } else {
            println!("📅 Saved sessions:");
        }
    } else {
        // When highlighting a single record (called from handle_add), avoid printing any header
        // to output exclusively the single record.
    }

    let mut total_surplus = 0;
    // Parse work_minutes once to avoid repeated parsing inside the loop
    let work_minutes = utils::parse_work_duration_to_minutes(&config.min_work_duration);

    for s in sessions {
        let (pos_string, pos_color) = describe_position(s.position.as_str());
        let has_start = !s.start.trim().is_empty();
        let has_end = !s.end.trim().is_empty();

        if has_start && !has_end {
            // Only start → calculate expected end
            let expected = logic::calculate_expected_exit(&s.start, work_minutes, s.lunch);

            let lunch_color = if s.lunch > 0 { "\x1b[0m" } else { "\x1b[90m" };
            let lunch_str = if s.lunch > 0 {
                mins2hhmm(s.lunch)
            } else {
                "-".to_string()
            };
            let lunch_fmt = format!("{:^5}", lunch_str);

            let end_color = if !s.end.is_empty() {
                "\x1b[0m"
            } else {
                "\x1b[90m"
            };
            let end_str = if !s.end.is_empty() {
                s.end
            } else {
                "-".to_string()
            };
            let end_fmt = format!("{:^5}", end_str);

            println!(
                "{:>3}: {} | {}{:<16}\x1b[0m | Start {} | {}Lunch {}\x1b[0m | {}End {}\x1b[0m | Expected {} | \x1b[90mSurplus {:^8}\x1b[0m",
                s.id,
                s.date,
                pos_color,
                pos_string,
                s.start,
                lunch_color,
                lunch_fmt,
                end_color,
                end_fmt,
                expected.format("%H:%M"),
                "-",
            );
        } else if has_start && has_end {
            let start_time = NaiveTime::parse_from_str(&s.start, "%H:%M").unwrap();
            let end_time = NaiveTime::parse_from_str(&s.end, "%H:%M").unwrap();
            let pos_char = s.position.chars().next().unwrap_or('O');
            let crosses_lunch = logic::crosses_lunch_window(&s.start, &s.end);

            // Compute effective lunch
            let effective_lunch =
                logic::effective_lunch_minutes(s.lunch, &s.start, &s.end, pos_char, config);

            if crosses_lunch && effective_lunch > 0 {
                // Case with lunch (inserted or automatic)
                let expected =
                    logic::calculate_expected_exit(&s.start, work_minutes, effective_lunch);
                let surplus =
                    logic::calculate_surplus(&s.start, effective_lunch, &s.end, work_minutes);
                let surplus_minutes = surplus.num_minutes();
                total_surplus += surplus_minutes;

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

                let lunch_str = if effective_lunch > 0 {
                    mins2hhmm(effective_lunch)
                } else {
                    "-".to_string()
                };
                let lunch_fmt = format!("{:^5}", lunch_str);

                println!(
                    "{:>3}: {} | {}{:<16}\x1b[0m | Start {} | Lunch {} | End {} | Expected {} | Surplus {}{:>4} min\x1b[0m",
                    s.id,
                    s.date,
                    pos_color,
                    pos_string,
                    s.start,
                    lunch_fmt,
                    s.end,
                    expected.format("%H:%M"),
                    color_code,
                    formatted_surplus
                );
            } else {
                let duration = end_time - start_time;
                let lunch_fmt = format!("{:^5}", "-".to_string());

                println!(
                    "{:>3}: {} | {}{:<16}\x1b[0m | Start {} | \x1b[90mLunch {}\x1b[0m | End {} | \x1b[36mWorked {:>2} h {:02} min\x1b[0m",
                    s.id,
                    s.date,
                    pos_color,
                    pos_string,
                    s.start,
                    lunch_fmt,
                    s.end,
                    duration.num_hours(),
                    duration.num_minutes() % 60
                );
            }
        } else {
            let lunch_str = if s.lunch > 0 {
                mins2hhmm(s.lunch)
            } else {
                "-".to_string()
            };

            let lunch_fmt = format!("{:^5}", lunch_str);

            println!(
                "{:>3}: {} | {}{:<16}\x1b[0m | \x1b[90mStart {:^5} | Lunch {} | End {:^5} | Expected {:^5} | Surplus {:>4} min\x1b[0m",
                s.id,
                s.date,
                pos_color,
                pos_string,
                if has_start { &s.start } else { "-" },
                lunch_fmt,
                if has_end { &s.end } else { "-" },
                "-",
                "-",
            );
        }
    }

    if highlight_id.is_none() {
        println!();
        print_separator('-', 25, 110);

        if total_surplus != 0 {
            let color_code = if total_surplus < 0 {
                "\x1b[31m" // rosso
            } else {
                "\x1b[32m" // verde
            };

            let formatted_total = format!("{:+}", total_surplus);

            println!(
                "{:>119}",
                format!(
                    "Σ Total surplus: {}{:>4} min\x1b[0m",
                    color_code, formatted_total
                ),
            );
        } else {
            println!("{:>119}", format!("Σ Total surplus: {:>4} min", 0));
        }
    }

    Ok(())
}
