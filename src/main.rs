mod categories;
mod cli;
mod config;
mod prompt;
mod scanner;
mod snapshot;
mod tree;
mod tui;
mod util;

use clap::Parser;
use cli::Cli;
use config::Config;

fn main() {
    let cli = Cli::parse();
    let config = Config::from_cli(&cli);

    eprintln!(
        "Scanning {}...",
        config.target_path.display()
    );
    let scan = scanner::scan(&config);
    eprintln!(
        "Scanned {} files, total size: {} ({} permission errors)",
        scan.total_files,
        util::format_size(scan.total_size),
        scan.permission_errors
    );

    // --export json: print and exit
    if config.export_json {
        println!("{}", serde_json::to_string_pretty(&scan).unwrap());
        return;
    }

    // --save-snapshot
    if config.save_snapshot {
        match snapshot::save_snapshot(&scan, &config) {
            Ok(path) => eprintln!("Snapshot saved: {}", path.display()),
            Err(e) => eprintln!("Failed to save snapshot: {}", e),
        }
    }

    // Load diff snapshot if requested
    let diff = config.diff_file.as_ref().and_then(|path| {
        match snapshot::load_snapshot(path) {
            Ok(old) => Some(snapshot::diff_snapshots(&old, &scan)),
            Err(e) => {
                eprintln!("Failed to load snapshot: {}", e);
                None
            }
        }
    });

    // --ai: export prompt and exit
    if config.ai_mode {
        match prompt::export(&scan, diff.as_ref(), &config) {
            Ok(path) => {
                println!("Prompt exported to: {}", path.display());
                println!("Run:  claude < {}", path.display());
            }
            Err(e) => eprintln!("Failed to export prompt: {}", e),
        }
        return;
    }

    // Launch TUI
    if let Err(e) = tui::run(scan, diff, config) {
        eprintln!("TUI error: {}", e);
        std::process::exit(1);
    }
}
