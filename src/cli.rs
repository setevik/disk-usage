use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "diskwise", about = "AI-assisted disk usage reporter")]
pub struct Cli {
    /// Path to scan (default: $HOME)
    pub path: Option<PathBuf>,

    /// Export prompt for Claude AI analysis
    #[arg(long)]
    pub ai: bool,

    /// Save current scan as a snapshot
    #[arg(long)]
    pub save_snapshot: bool,

    /// Load a snapshot file for diff comparison
    #[arg(long, value_name = "FILE")]
    pub diff: Option<PathBuf>,

    /// Exclude paths matching glob pattern (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Override stale file threshold in days
    #[arg(long, default_value = "180")]
    pub stale_days: u64,

    /// Export scan result as JSON and exit
    #[arg(long, value_name = "FORMAT")]
    pub export: Option<String>,
}
