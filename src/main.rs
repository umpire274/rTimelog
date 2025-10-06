use clap::Parser;
use rtimelogger::config::Config;
use rtimelogger::db;
use rusqlite::Connection;

mod commands;
use rtimelogger::cli::{Cli, Commands};

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();

    // Determine DB path without loading the full config (Config::load may read files under
    // $HOME or %APPDATA% which tests may control); prefer to avoid reading it when --test is set.
    let db_path = if let Some(custom) = &cli.db {
        let custom_path = std::path::Path::new(custom);
        if custom_path.is_absolute() {
            custom.to_string()
        } else {
            Config::config_dir()
                .join(custom_path)
                .to_string_lossy()
                .to_string()
        }
    } else if cli.test {
        // In test mode: use the default file name under the test config dir, but DO NOT call Config::load()
        Config::config_dir()
            .join("rtimelog.sqlite")
            .to_string_lossy()
            .to_string()
    } else {
        // Production: load the configuration and use the database path from it
        let config = Config::load();
        config.database.clone()
    };

    // Now prepare a `config` object for use by commands; when running under --test or when --db is
    // provided we construct a default config (matching Config::load() defaults) and point its
    // `database` to the resolved db_path. Only when neither `--db` nor `--test` are used we call
    // `Config::load()` to read possible overrides from disk.
    let config = if cli.test || cli.db.is_some() {
        Config {
            database: db_path.clone(),
            default_position: "O".to_string(),
            min_work_duration: "8h".to_string(),
            min_duration_lunch_break: 30,
            max_duration_lunch_break: 90,
            separator_char: "-".to_string(),
        }
    } else {
        // For production, we load the configuration from disk.
        Config::load()
    };

    println!();

    // Handle `init` separately because it may need to create config/db files first
    if let Commands::Init = &cli.command {
        return commands::handle_init(&cli, &db_path);
    }

    // For other commands, open a single shared connection, set useful PRAGMA and ensure DB is initialized (creates
    // base tables and runs pending migrations).
    let mut conn = Connection::open(&db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    db::init_db(&conn)?;

    match &cli.command {
        Commands::Add { .. } => commands::handle_add(&cli.command, &mut conn, &config)?,
        Commands::Del { .. } => commands::handle_del(&cli.command, &mut conn)?,
        Commands::List {
            period,
            pos,
            now,
            details,
            events,
            pairs,
            summary,
            json,
        } => {
            let args = commands::HandleListArgs {
                period: period.clone(),
                pos: pos.clone(),
                now: *now,
                details: *details,
                events: *events,
                pairs: *pairs,
                summary: *summary,
                json: *json,
            };
            commands::handle_list(&args, &conn, &config)?
        }
        Commands::Conf { .. } => commands::handle_conf(&cli.command)?,
        Commands::Log { .. } => commands::handle_log(&cli.command, &conn)?,
        Commands::Init => {
            // Already handled, but included for exhaustiveness
        }
    }

    Ok(())
}
