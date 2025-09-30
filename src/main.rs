use clap::{Parser, Subcommand};
use r_timelog::config::Config;
use r_timelog::db;
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
    },
    /// Delete a work session by ID
    Del {
        /// ID of the session to delete
        id: i32,
    },
    /// List sessions
    List {
        #[arg(long, short)]
        period: Option<String>,

        /// Filter by position (O=Office, R=Remote, H=Holiday)
        #[arg(long)]
        pos: Option<String>,
    },
}

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();

    let config = Config::load();

    // Choose DB path: --db overrides config
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
        // In test mode: usa comunque il file di default ma NON chiama Config::load()
        Config::config_dir()
            .join("rtimelog.sqlite")
            .to_string_lossy()
            .to_string()
    } else {
        // Produzione: usa la configurazione già caricata
        config.database.clone()
    };

    println!();

    // Handle `init` separately because it may need to create config/db files first
    if let Commands::Init = &cli.command {
        return commands::handle_init(&cli, &db_path);
    }

    // For other commands, open a single shared connection, set useful PRAGMA and run migrations once
    let conn = Connection::open(&db_path)?;
    // Improve write concurrency and performance on SQLite
    let _ = conn.pragma_update(None, "journal_mode", &"WAL");
    let _ = conn.pragma_update(None, "synchronous", &"NORMAL");
    // Ensure DB schema is up-to-date once
    db::run_pending_migrations(&conn)?;

    match &cli.command {
        Commands::Conf { .. } => commands::handle_conf(&cli.command),
        Commands::Add { .. } => commands::handle_add(&cli.command, &conn, &config),
        Commands::Del { .. } => commands::handle_del(&cli.command, &conn),
        Commands::List { period, pos } => {
            commands::handle_list(period.clone(), pos.clone(), &conn, &config)
        }
        _ => Ok(()),
    }
}
