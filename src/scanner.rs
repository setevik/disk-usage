use crate::config::Config;
use crate::tree::{DirEntry, DirNode, FileEntry, ScanResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub fn scan(config: &Config) -> ScanResult {
    let target = &config.target_path;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut permission_errors: u64 = 0;
    let mut dirs_map: HashMap<PathBuf, Vec<DirEntry>> = HashMap::new();
    let mut dir_meta: HashMap<PathBuf, String> = HashMap::new();

    // Initialize the root
    let root_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| target.to_string_lossy().to_string());
    dirs_map.insert(target.to_path_buf(), Vec::new());
    dir_meta.insert(target.to_path_buf(), root_name);

    let walker = WalkDir::new(target)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), target, &config.exclude));

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                permission_errors += 1;
                continue;
            }
        };

        let path = entry.path().to_path_buf();
        if path == *target {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                permission_errors += 1;
                continue;
            }
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let parent = path.parent().unwrap_or(target).to_path_buf();

        if metadata.is_dir() {
            dirs_map.entry(path.clone()).or_default();
            dir_meta.insert(path, name);
            // Ensure parent knows about this dir — we'll assemble later
        } else if metadata.is_file() {
            let size = metadata.len();
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let atime = metadata
                .accessed()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let file_entry = FileEntry {
                name,
                path: path.clone(),
                size,
                mtime,
                atime,
                extension,
            };

            dirs_map
                .entry(parent)
                .or_default()
                .push(DirEntry::File(file_entry));
        }
    }

    // Build tree bottom-up: sort paths by depth descending
    let mut dir_paths: Vec<PathBuf> = dirs_map.keys().cloned().collect();
    dir_paths.sort_by(|a, b| {
        let da = a.components().count();
        let db = b.components().count();
        db.cmp(&da).then_with(|| a.cmp(b))
    });

    let mut built_dirs: HashMap<PathBuf, DirNode> = HashMap::new();

    for dir_path in &dir_paths {
        let children = dirs_map.remove(dir_path).unwrap_or_default();
        let name = dir_meta
            .get(dir_path)
            .cloned()
            .unwrap_or_else(|| dir_path.to_string_lossy().to_string());

        let mut node = DirNode {
            name,
            path: dir_path.clone(),
            size: 0,
            file_count: 0,
            children,
        };

        // Add any child dirs that were already built
        let child_dir_paths: Vec<PathBuf> = built_dirs
            .keys()
            .filter(|p| p.parent() == Some(dir_path.as_path()))
            .cloned()
            .collect();

        for child_path in child_dir_paths {
            if let Some(child_node) = built_dirs.remove(&child_path) {
                node.children.push(DirEntry::Dir(child_node));
            }
        }

        // Compute size and file_count
        for child in &node.children {
            match child {
                DirEntry::Dir(d) => {
                    node.size += d.size;
                    node.file_count += d.file_count;
                }
                DirEntry::File(f) => {
                    node.size += f.size;
                    node.file_count += 1;
                }
            }
        }

        // Sort children by size descending
        node.children
            .sort_by_key(|b| std::cmp::Reverse(b.size()));

        if *dir_path == *target {
            let total_size = node.size;
            let total_files = node.file_count;
            return ScanResult {
                target_path: target.clone(),
                timestamp: now,
                total_size,
                total_files,
                root: node,
                permission_errors,
            };
        }

        built_dirs.insert(dir_path.clone(), node);
    }

    // Fallback (shouldn't happen)
    let root = built_dirs.remove(target).unwrap_or(DirNode {
        name: root_name_from_path(target),
        path: target.clone(),
        size: 0,
        file_count: 0,
        children: Vec::new(),
    });
    let total_size = root.size;
    let total_files = root.file_count;
    ScanResult {
        target_path: target.clone(),
        timestamp: now,
        total_size,
        total_files,
        root,
        permission_errors,
    }
}

fn root_name_from_path(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.to_string_lossy().to_string())
}

fn should_exclude(path: &Path, target: &Path, excludes: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in excludes {
        // Absolute path exclusions
        if !pattern.contains('*') && path_str.starts_with(pattern.as_str()) {
            return true;
        }

        // Glob-style exclusions
        let rel = path
            .strip_prefix(target)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if glob_match::glob_match(pattern, &rel) {
            return true;
        }

        // Also match against just the file/dir name for patterns like "node_modules"
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if glob_match::glob_match(pattern, name_str.as_ref()) {
                return true;
            }
        }
    }
    false
}
