use clap::{Parser, Subcommand};

/// Command-line interface definition for rTimelogger
/// CLI application to track working hours with SQLite
#[derive(Parser)]
#[command(
    name = "rtimelogger",
    version = env!("CARGO_PKG_VERSION"),
    about = "A simple time logging CLI: track working hours and calculate surplus using SQLite",
    long_about = None
)]
pub struct Cli {
    /// Override database path (useful for tests or custom DB)
    #[arg(global = true, long = "db")]
    pub db: Option<String>,

    /// Run in test mode (no config file update)
    #[arg(global = true, long = "test", hide = true)]
    pub test: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

        /// (Positional) Position: O=office, R=remote, H=holiday, C=On-Site Client
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
    },

    /// Create a backup copy of the database
    Backup {
        /// Destination file path (absolute path required)
        #[arg(long, value_name = "FILE")]
        file: String,

        /// Compress the backup (zip on Windows, tar.gz on Unix)
        #[arg(long)]
        compress: bool,
    },

    /// Export work session data in various formats
    Export {
        /// Export format: csv, json
        #[arg(long, value_name = "FORMAT", default_value = "csv")]
        format: String,

        /// Output file path (absolute path required)
        #[arg(long, value_name = "FILE")]
        file: String,

        /// Date range for export (e.g., "2025-01" for January 2025, "2025-01:2025-03" for Jan-Mar 2025)
        #[arg(long, value_name = "RANGE")]
        range: Option<String>,

        /// Export EVENTS (from `events` table)
        #[arg(long, conflicts_with = "sessions")]
        events: bool,

        /// Export SESSIONS (from `work_sessions` table)
        #[arg(long, conflicts_with = "events")]
        sessions: bool,

        /// Overwrite output file without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
}
