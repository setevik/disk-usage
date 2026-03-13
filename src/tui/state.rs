use crate::config::Config;
use crate::snapshot::DiffResult;
use crate::tree::{DirEntry, DirNode, ScanResult};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tree,
    Types,
    Diff,
    Stale,
}

impl Tab {
    pub fn label(&self) -> &str {
        match self {
            Tab::Tree => "1:Tree",
            Tab::Types => "2:Types",
            Tab::Diff => "3:Diff",
            Tab::Stale => "4:Stale",
        }
    }

    pub fn all() -> &'static [Tab] {
        &[Tab::Tree, Tab::Types, Tab::Diff, Tab::Stale]
    }
}

#[derive(Debug, Clone)]
pub struct FlatItem {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub size: u64,
    pub is_dir: bool,
    pub has_children: bool,
    pub is_expanded: bool,
    pub pct_of_parent: f64,
}

pub struct TreeState {
    pub cursor: usize,
    pub expanded: HashSet<PathBuf>,
    pub visible_items: Vec<FlatItem>,
    pub scroll_offset: usize,
}

impl TreeState {
    pub fn new(root: &DirNode) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root.path.clone());
        let mut state = TreeState {
            cursor: 0,
            expanded,
            visible_items: Vec::new(),
            scroll_offset: 0,
        };
        state.rebuild(root);
        state
    }

    pub fn rebuild(&mut self, root: &DirNode) {
        self.visible_items.clear();
        self.flatten(root, 0, root.size);
        // Clamp cursor
        if !self.visible_items.is_empty() && self.cursor >= self.visible_items.len() {
            self.cursor = self.visible_items.len() - 1;
        }
    }

    fn flatten(&mut self, node: &DirNode, depth: usize, parent_size: u64) {
        let is_expanded = self.expanded.contains(&node.path);
        let has_children = !node.children.is_empty();
        let pct = if parent_size > 0 {
            (node.size as f64 / parent_size as f64) * 100.0
        } else {
            0.0
        };

        self.visible_items.push(FlatItem {
            path: node.path.clone(),
            name: node.name.clone(),
            depth,
            size: node.size,
            is_dir: true,
            has_children,
            is_expanded,
            pct_of_parent: pct,
        });

        if is_expanded {
            for child in &node.children {
                match child {
                    DirEntry::Dir(d) => {
                        self.flatten(d, depth + 1, node.size);
                    }
                    DirEntry::File(f) => {
                        let pct = if node.size > 0 {
                            (f.size as f64 / node.size as f64) * 100.0
                        } else {
                            0.0
                        };
                        self.visible_items.push(FlatItem {
                            path: f.path.clone(),
                            name: f.name.clone(),
                            depth: depth + 1,
                            size: f.size,
                            is_dir: false,
                            has_children: false,
                            is_expanded: false,
                            pct_of_parent: pct,
                        });
                    }
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.visible_items.len() {
            self.cursor += 1;
        }
    }

    pub fn toggle_expand(&mut self, root: &DirNode) {
        if let Some(item) = self.visible_items.get(self.cursor) {
            if item.is_dir && item.has_children {
                let path = item.path.clone();
                if self.expanded.contains(&path) {
                    self.expanded.remove(&path);
                } else {
                    self.expanded.insert(path);
                }
                self.rebuild(root);
            }
        }
    }

    pub fn go_to_parent(&mut self, root: &DirNode) {
        if let Some(item) = self.visible_items.get(self.cursor) {
            if let Some(parent) = item.path.parent() {
                let parent = parent.to_path_buf();
                // Find parent in visible items
                if let Some(idx) = self.visible_items.iter().position(|i| i.path == parent) {
                    self.cursor = idx;
                }
                // Collapse current if it's an expanded dir
                if item.is_dir && self.expanded.contains(&item.path) {
                    let path = item.path.clone();
                    self.expanded.remove(&path);
                    self.rebuild(root);
                }
            }
        }
    }

    pub fn ensure_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + height {
            self.scroll_offset = self.cursor - height + 1;
        }
    }
}

pub struct AppState {
    pub scan: ScanResult,
    pub diff: Option<DiffResult>,
    pub config: Config,
    pub active_tab: Tab,
    pub tree_state: TreeState,
    pub ai_drawer_open: bool,
    pub ai_prompt_path: Option<PathBuf>,
    pub quit: bool,
    pub show_help: bool,
    pub stale_cursor: usize,
    pub diff_cursor: usize,
}

impl AppState {
    pub fn new(scan: ScanResult, diff: Option<DiffResult>, config: Config) -> Self {
        let tree_state = TreeState::new(&scan.root);
        AppState {
            scan,
            diff,
            config,
            active_tab: Tab::Tree,
            tree_state,
            ai_drawer_open: false,
            ai_prompt_path: None,
            quit: false,
            show_help: false,
            stale_cursor: 0,
            diff_cursor: 0,
        }
    }
}
