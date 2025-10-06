use clap::{Parser, Subcommand};
use rtimelogger::config::Config;
use rtimelogger::db;
use rusqlite::Connection;

mod commands;

/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(
    name = "rtimelog",
    version = env!("CARGO_PKG_VERSION"),
    about = "A simple time logging CLI in Rust: track working hours and calculate surplus using SQLite",
    long_about = None
)]
struct Cli {
    /// Override database path (useful for tests or custom DB)
    #[arg(global = true, long = "db")]
    db: Option<String>,

    /// Run in test mode (no config file update)
    #[arg(global = true, long = "test", hide = true)]
    test: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database and configuration
    Init,

    /// Manage the configuration file (view or edit)
    Conf {
        /// Print the current configuration file to stdout
        #[arg(long = "print", help = "Print the current configuration file")]
        print_config: bool,

        /// Edit the configuration file with your preferred editor
        #[arg(
            long = "edit",
            help = "Edit the configuration file (default editor: $EDITOR, or nano/vim/notepad)"
        )]
        edit_config: bool,

        /// Specify the editor to use (overrides $EDITOR/$VISUAL).
        /// Common choices: vim, nano.
        #[arg(
            long = "editor",
            help = "Specify the editor to use (vim, nano, or custom path)"
        )]
        editor: Option<String>,
    },

    /// Print or manage the internal log table
    Log {
        /// Print rows from the internal `log` table
        #[arg(long = "print", help = "Print rows from the internal log table")]
        print: bool,
    },

    /// Add or update a work session
    Add {
        /// Date (YYYY-MM-DD)
        date: String,

        /// (Positional)Position: A=office, R=remote
        pos_pos: Option<String>,
        /// (Positional) Start time (HH:MM)
        start_pos: Option<String>,
        /// (Positional) Lunch minutes
        lunch_pos: Option<i32>,
        /// (Positional) End time (HH:MM)
        end_pos: Option<String>,

        /// (Option) Position: A=office, R=remote
        #[arg(long = "pos")]
        pos: Option<String>,
        /// (Option) Start time (HH:MM)
        #[arg(long = "in")]
        start: Option<String>,
        /// (Option) Lunch minutes
        #[arg(long = "lunch")]
        lunch: Option<i32>,
        /// (Option) End time (HH:MM)
        #[arg(long = "out")]
        end: Option<String>,
        /// (Option) Pair id to edit (requires --edit)
        #[arg(long = "pair", help = "Pair id to edit (with --edit)")]
        edit_pair: Option<usize>,
        /// Enable edit mode (together with --pair) to update an existing pair's events instead of creating new ones
        #[arg(long = "edit", help = "Edit existing pair (use with --pair)")]
        edit: bool,
    },
    /// Delete a work session by ID
    Del {
        /// Optional pair id to delete (use with date): deletes only the given pair for the date
        #[arg(long = "pair", help = "Pair id to delete for the given date")]
        pair: Option<usize>,

        /// Date (YYYY-MM-DD) to delete (all sessions/events for this date) or required with --pair
        date: String,
    },
    /// List sessions
    List {
        #[arg(long, short)]
        period: Option<String>,

        /// Filter by position (O=Office, R=Remote, H=Holiday)
        #[arg(long)]
        pos: Option<String>,

        /// Show only today's record (if present)
        #[arg(long = "now", help = "Show only today's record")]
        now: bool,

        /// When used with --now, show the detailed events (in/out) for today instead of aggregated work_sessions
        #[arg(
            long = "details",
            help = "With --now show today's detailed events (in/out) instead of aggregated work_sessions"
        )]
        details: bool,

        /// Show all events (in/out) from the `events` table
        #[arg(
            long = "events",
            help = "List all events (in/out) from the events table"
        )]
        events: bool,

        /// Filter a specific pair id (requires --events); pairs are per-day sequential in/out groupings
        #[arg(long = "pairs", help = "Filter by pair id (only with --events)")]
        pairs: Option<usize>,

        /// Summarize events into per-pair rows (in/out, duration, lunch); use with --events
        #[arg(
            long = "summary",
            help = "Show summarized per-pair rows (requires --events)"
        )]
        summary: bool,

        /// Output in JSON format (applies to sessions or events depending on other flags)
        #[arg(
            long = "json",
            help = "Output data as JSON instead of human-readable text"
        )]
        json: bool,
    },
}

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
