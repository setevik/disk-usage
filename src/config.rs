use crate::cli::Cli;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
struct TomlConfig {
    scan: Option<ScanConfig>,
    ai: Option<AiConfig>,
    snapshots: Option<SnapshotConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct ScanConfig {
    default_path: Option<String>,
    exclude: Option<Vec<String>>,
    stale_days: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct AiConfig {
    prompt_dir: Option<String>,
    max_retained_prompts: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
struct SnapshotConfig {
    max_retained: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub target_path: PathBuf,
    pub exclude: Vec<String>,
    pub stale_days: u64,
    pub prompt_dir: PathBuf,
    pub max_retained_prompts: usize,
    pub snapshot_dir: PathBuf,
    pub max_retained_snapshots: usize,
    pub ai_mode: bool,
    pub save_snapshot: bool,
    pub diff_file: Option<PathBuf>,
    pub export_json: bool,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Self {
        let toml_cfg = load_toml_config();

        let default_excludes = vec![
            "**/node_modules".to_string(),
            "**/.git".to_string(),
            "**/target".to_string(),
        ];

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let target_path = cli.path.clone().unwrap_or_else(|| {
            toml_cfg
                .scan
                .as_ref()
                .and_then(|s| s.default_path.as_ref())
                .map(|p| {
                    if p.starts_with('~') {
                        home.join(&p[2..])
                    } else {
                        PathBuf::from(p)
                    }
                })
                .unwrap_or_else(|| home.clone())
        });

        let mut exclude = if !cli.exclude.is_empty() {
            cli.exclude.clone()
        } else {
            toml_cfg
                .scan
                .as_ref()
                .and_then(|s| s.exclude.clone())
                .unwrap_or(default_excludes)
        };
        // Always exclude system dirs
        for sys in &["/proc", "/sys", "/dev"] {
            if !exclude.iter().any(|e| e == *sys) {
                exclude.push(sys.to_string());
            }
        }

        let stale_days = if cli.stale_days != 180 {
            cli.stale_days
        } else {
            toml_cfg
                .scan
                .as_ref()
                .and_then(|s| s.stale_days)
                .unwrap_or(180)
        };

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| home.join(".local/share"))
            .join("diskwise");

        let prompt_dir = toml_cfg
            .ai
            .as_ref()
            .and_then(|a| a.prompt_dir.as_ref())
            .map(|p| {
                if p.starts_with('~') {
                    home.join(&p[2..])
                } else {
                    PathBuf::from(p)
                }
            })
            .unwrap_or_else(|| data_dir.join("prompts"));

        let max_retained_prompts = toml_cfg
            .ai
            .as_ref()
            .and_then(|a| a.max_retained_prompts)
            .unwrap_or(20);

        let snapshot_dir = data_dir.join("snapshots");

        let max_retained_snapshots = toml_cfg
            .snapshots
            .as_ref()
            .and_then(|s| s.max_retained)
            .unwrap_or(10);

        Config {
            target_path,
            exclude,
            stale_days,
            prompt_dir,
            max_retained_prompts,
            snapshot_dir,
            max_retained_snapshots,
            ai_mode: cli.ai,
            save_snapshot: cli.save_snapshot,
            diff_file: cli.diff.clone(),
            export_json: cli.export.as_deref() == Some("json"),
        }
    }
}

fn load_toml_config() -> TomlConfig {
    let config_path = dirs::config_dir()
        .map(|d| d.join("diskwise/config.toml"))
        .unwrap_or_else(|| PathBuf::from(""));

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(cfg) = toml::from_str(&content) {
                return cfg;
            }
        }
    }
    TomlConfig::default()
}
