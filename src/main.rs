mod db;
mod daemon;
mod search;
mod tui;
mod record;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hindsight", version, about = "20/20 vision for your shell history")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query (shorthand for `search`)
    query: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Daemon,

    /// Record a command (called by shell hooks or daemon)
    Add {
        /// The command that was executed
        #[arg(short = 'x', long)]
        command: String,

        /// Working directory
        #[arg(short, long)]
        cwd: String,

        /// Exit code
        #[arg(short, long)]
        exit: Option<i32>,

        /// Duration in milliseconds
        #[arg(short, long)]
        duration: Option<u64>,
    },

    /// Search history (interactive TUI if no query given)
    Search {
        /// Search query (omit for interactive mode)
        query: Option<String>,

        /// Filter by working directory
        #[arg(short, long)]
        cwd: Option<String>,

        /// Filter by exit code
        #[arg(short, long)]
        exit: Option<i32>,

        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Interactive search (TUI)
    #[command(alias = "i")]
    Interactive {
        /// Initial query
        query: Option<String>,
    },

    /// Show top N most used commands
    Top {
        /// Number of results
        #[arg(default_value = "20")]
        n: usize,
    },

    /// Show stats
    Stats,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Daemon) => daemon::run()?,
        Some(Commands::Add { command, cwd, exit, duration }) => {
            let conn = db::open()?;
            db::insert(&conn, &command, &cwd, exit, duration)?;
        }
        Some(Commands::Search { query, cwd, exit, limit }) => {
            let q = query.unwrap_or_default();
            if q.is_empty() {
                // No query — open interactive TUI
                tui::run("")?;
            } else {
                let conn = db::open()?;
                let results = search::search(&conn, &q, cwd.as_deref(), exit, limit)?;
                for r in results {
                    println!("{}", r);
                }
            }
        }
        Some(Commands::Interactive { query }) => {
            tui::run(query.as_deref().unwrap_or(""))?;
        }
        Some(Commands::Top { n }) => {
            let conn = db::open()?;
            let results = search::top(&conn, n)?;
            for (cmd, count) in results {
                println!("{:>5}  {}", count, cmd);
            }
        }
        Some(Commands::Stats) => {
            let conn = db::open()?;
            let stats = search::stats(&conn)?;
            println!("Total commands: {}", stats.total);
            println!("Unique commands: {}", stats.unique);
            println!("Total sessions: {}", stats.sessions);
        }
        None => {
            let query = cli.query.unwrap_or_default();
            if query.is_empty() {
                // No args at all — show recent
                let conn = db::open()?;
                let results = search::recent(&conn, 20)?;
                for r in results {
                    println!("{}", r);
                }
            } else {
                // Has query — search
                let conn = db::open()?;
                let results = search::search(&conn, &query, None, None, 20)?;
                for r in results {
                    println!("{}", r);
                }
            }
        }
    }

    Ok(())
}
