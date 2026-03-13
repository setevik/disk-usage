use crate::config::Config;
use crate::tree::{DirEntry, DirNode, ScanResult};
use crate::util::format_size;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub old_timestamp: u64,
    pub new_timestamp: u64,
    pub total_delta: i64,
    pub entries: Vec<DiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub path: String,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub delta: i64,
    pub status: DiffStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffStatus {
    New,
    Deleted,
    Changed,
    Unchanged,
}

impl std::fmt::Display for DiffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffStatus::New => write!(f, "NEW"),
            DiffStatus::Deleted => write!(f, "DEL"),
            DiffStatus::Changed => write!(f, "CHG"),
            DiffStatus::Unchanged => write!(f, "---"),
        }
    }
}

impl DiffResult {
    pub fn format_total_delta(&self) -> String {
        if self.total_delta >= 0 {
            format!("+{}", format_size(self.total_delta as u64))
        } else {
            format!("-{}", format_size((-self.total_delta) as u64))
        }
    }

    pub fn top_grown(&self, n: usize) -> Vec<&DiffEntry> {
        let mut grown: Vec<&DiffEntry> = self
            .entries
            .iter()
            .filter(|e| e.delta > 0)
            .collect();
        grown.sort_by(|a, b| b.delta.cmp(&a.delta));
        grown.truncate(n);
        grown
    }

    pub fn new_large(&self, min_size: u64) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.status == DiffStatus::New && e.new_size.unwrap_or(0) >= min_size)
            .collect()
    }
}

pub fn save_snapshot(scan: &ScanResult, config: &Config) -> std::io::Result<PathBuf> {
    fs::create_dir_all(&config.snapshot_dir)?;

    let path_slug = scan
        .target_path
        .to_string_lossy()
        .replace('/', "_")
        .trim_start_matches('_')
        .to_string();
    let ts = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("{}_{}.json.gz", path_slug, ts);
    let filepath = config.snapshot_dir.join(&filename);

    let json = serde_json::to_vec(scan).map_err(std::io::Error::other)?;
    let file = fs::File::create(&filepath)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&json)?;
    encoder.finish()?;

    // Prune old snapshots
    prune_snapshots(&config.snapshot_dir, config.max_retained_snapshots)?;

    Ok(filepath)
}

pub fn load_snapshot(path: &Path) -> std::io::Result<ScanResult> {
    let file = fs::File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut json = String::new();
    decoder.read_to_string(&mut json)?;
    serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn diff_snapshots(old: &ScanResult, new: &ScanResult) -> DiffResult {
    let old_flat = flatten_dirs(&old.root);
    let new_flat = flatten_dirs(&new.root);

    let mut entries = Vec::new();
    let total_delta = new.total_size as i64 - old.total_size as i64;

    // Merge walk
    let mut all_paths: Vec<&String> = old_flat.keys().chain(new_flat.keys()).collect();
    all_paths.sort();
    all_paths.dedup();

    for path in all_paths {
        let old_size = old_flat.get(path).copied();
        let new_size = new_flat.get(path).copied();

        let (delta, status) = match (old_size, new_size) {
            (Some(o), Some(n)) => {
                let d = n as i64 - o as i64;
                if d == 0 {
                    (0, DiffStatus::Unchanged)
                } else {
                    (d, DiffStatus::Changed)
                }
            }
            (None, Some(n)) => (n as i64, DiffStatus::New),
            (Some(o), None) => (-(o as i64), DiffStatus::Deleted),
            (None, None) => continue,
        };

        // Skip unchanged entries
        if status == DiffStatus::Unchanged {
            continue;
        }

        entries.push(DiffEntry {
            path: path.clone(),
            old_size,
            new_size,
            delta,
            status,
        });
    }

    // Sort by absolute delta descending
    entries.sort_by(|a, b| b.delta.abs().cmp(&a.delta.abs()));

    DiffResult {
        old_timestamp: old.timestamp,
        new_timestamp: new.timestamp,
        total_delta,
        entries,
    }
}

fn flatten_dirs(node: &DirNode) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    flatten_dirs_recursive(node, &mut map);
    map
}

fn flatten_dirs_recursive(node: &DirNode, map: &mut BTreeMap<String, u64>) {
    map.insert(node.path.to_string_lossy().to_string(), node.size);
    for child in &node.children {
        if let DirEntry::Dir(d) = child {
            flatten_dirs_recursive(d, map);
        }
    }
}

fn prune_snapshots(dir: &Path, max: usize) -> std::io::Result<()> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "gz"))
        .collect();

    if files.len() <= max {
        return Ok(());
    }

    files.sort();
    let to_remove = files.len() - max;
    for path in files.iter().take(to_remove) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}
