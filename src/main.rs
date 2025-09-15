use clap::{Parser, Subcommand};
use r_timelog::config::Config;

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

    /// List sessions
    List {
        #[arg(long, short)]
        period: Option<String>,
    },

    /// Initialize the database and configuration
    Init,

    /// Print current file configuration
    Conf {
        #[arg(long = "print")]
        print_config: bool,
    },
}

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();

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
        // Produzione: carica dal file di configurazione
        Config::load().database
    };

    println!();

    match &cli.command {
        Commands::Init => commands::handle_init(&cli, &db_path),
        Commands::Add { .. } => commands::handle_add(&cli.command, &db_path),
        Commands::List { period } => commands::handle_list(period.clone(), &db_path),
        Commands::Conf { .. } => commands::handle_conf(&cli.command),
    }
}
