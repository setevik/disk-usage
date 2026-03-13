use crate::tree::{DirEntry, DirNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    BuildArtifacts,
    Logs,
    Media,
    Archives,
    Docker,
    Vcs,
    NodeModules,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::BuildArtifacts => write!(f, "Build Artifacts"),
            Category::Logs => write!(f, "Logs"),
            Category::Media => write!(f, "Media"),
            Category::Archives => write!(f, "Archives"),
            Category::Docker => write!(f, "Docker"),
            Category::Vcs => write!(f, "VCS"),
            Category::NodeModules => write!(f, "node_modules"),
            Category::Other => write!(f, "Other"),
        }
    }
}

impl Category {
    #[allow(dead_code)]
    pub fn all() -> &'static [Category] {
        &[
            Category::BuildArtifacts,
            Category::Logs,
            Category::Media,
            Category::Archives,
            Category::Docker,
            Category::Vcs,
            Category::NodeModules,
            Category::Other,
        ]
    }
}

pub fn categorize(ext: Option<&str>, path: &str) -> Category {
    // Path-based rules first
    if path.contains("node_modules") {
        return Category::NodeModules;
    }
    if path.contains(".git/objects") || path.contains(".git/lfs") {
        return Category::Vcs;
    }
    if path.contains("/var/lib/docker/overlay2") || path.contains("/var/lib/docker/volumes") {
        return Category::Docker;
    }
    if path.contains("__pycache__") {
        return Category::BuildArtifacts;
    }

    let ext = match ext {
        Some(e) => e.to_lowercase(),
        None => return Category::Other,
    };

    match ext.as_str() {
        // Build artifacts
        "o" | "a" | "rlib" | "rmeta" | "pyc" | "class" | "obj" => Category::BuildArtifacts,
        // Logs
        "log" => Category::Logs,
        // Media
        "mp4" | "mkv" | "avi" | "mov" | "png" | "jpg" | "jpeg" | "psd" | "raw" | "gif"
        | "bmp" | "tiff" | "webm" | "wav" | "mp3" | "flac" => Category::Media,
        // Archives
        "zip" | "tar" | "gz" | "xz" | "7z" | "deb" | "appimage" | "rpm" | "bz2" | "zst" => {
            Category::Archives
        }
        _ => Category::Other,
    }
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: Category,
    pub total_size: u64,
    pub file_count: u64,
}

pub fn aggregate_by_category(root: &DirNode) -> Vec<CategoryStats> {
    let mut map: HashMap<Category, (u64, u64)> = HashMap::new();
    collect_categories(root, &mut map);

    let mut stats: Vec<CategoryStats> = map
        .into_iter()
        .map(|(category, (total_size, file_count))| CategoryStats {
            category,
            total_size,
            file_count,
        })
        .collect();

    stats.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    stats
}

fn collect_categories(node: &DirNode, map: &mut HashMap<Category, (u64, u64)>) {
    for child in &node.children {
        match child {
            DirEntry::File(f) => {
                let cat = categorize(f.extension.as_deref(), &f.path.to_string_lossy());
                let entry = map.entry(cat).or_insert((0, 0));
                entry.0 += f.size;
                entry.1 += 1;
            }
            DirEntry::Dir(d) => {
                // Check if the dir itself is a category (e.g. node_modules)
                let path_str = d.path.to_string_lossy();
                if path_str.contains("node_modules")
                    || path_str.contains(".git/objects")
                    || path_str.contains(".git/lfs")
                    || path_str.contains("/var/lib/docker/")
                {
                    let cat = categorize(None, &path_str);
                    let entry = map.entry(cat).or_insert((0, 0));
                    entry.0 += d.size;
                    entry.1 += d.file_count;
                } else {
                    collect_categories(d, map);
                }
            }
        }
    }
}
