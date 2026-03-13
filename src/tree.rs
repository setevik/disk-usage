use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub target_path: PathBuf,
    pub timestamp: u64,
    pub root: DirNode,
    pub total_size: u64,
    pub total_files: u64,
    pub permission_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirEntry {
    Dir(DirNode),
    File(FileEntry),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub mtime: u64,
    pub atime: u64,
    pub extension: Option<String>,
}

impl DirEntry {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            DirEntry::Dir(d) => &d.name,
            DirEntry::File(f) => &f.name,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            DirEntry::Dir(d) => d.size,
            DirEntry::File(f) => f.size,
        }
    }
}
