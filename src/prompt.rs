use crate::categories::aggregate_by_category;
use crate::config::Config;
use crate::snapshot::DiffResult;
use crate::tree::{DirEntry, DirNode, FileEntry, ScanResult};
use crate::util::{epoch_to_iso, epoch_to_string, format_size};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_system_prompt() -> &'static str {
    r#"You are a disk usage analyst assistant. You will be given structured data about a Linux filesystem scan.
Your job is to produce a concise, actionable report for a developer or power user.

Rules:
- Be direct and specific. Mention actual paths and sizes.
- Prioritize by impact (largest space savings first).
- Flag anything that looks risky to delete (active project dirs, system paths, recently modified files).
- Do not suggest deleting anything under 50 MB — not worth the cognitive load.
- Use plain English. No markdown headers. No bullet nesting deeper than one level.
- Keep the full response under 400 words.

Response format (strictly follow this structure):

SUMMARY
<2–3 sentences describing the overall disk situation>

ACTION ITEMS
• <action> — <path> (<size>, <reason>)
• ...

CAUTIONS
• <anything that looks risky or ambiguous, or "None." if nothing warrants flagging>"#
}

pub fn build_user_message(
    scan: &ScanResult,
    diff: Option<&DiffResult>,
    stale_days: u64,
) -> String {
    let mut msg = String::new();

    writeln!(msg, "Scan target: {}", scan.target_path.display()).unwrap();
    writeln!(msg, "Scan timestamp: {}", epoch_to_iso(scan.timestamp)).unwrap();
    writeln!(msg, "Total scanned size: {}", format_size(scan.total_size)).unwrap();
    writeln!(msg, "Total files: {}", scan.total_files).unwrap();
    writeln!(msg).unwrap();

    // Top 10 largest directories
    writeln!(msg, "--- TOP 10 LARGEST DIRECTORIES ---").unwrap();
    let mut top_dirs: Vec<(&DirNode, f64)> = Vec::new();
    collect_top_dirs(&scan.root, scan.total_size, &mut top_dirs);
    top_dirs.sort_by(|a, b| b.0.size.cmp(&a.0.size));
    top_dirs.truncate(10);
    for (i, (dir, pct)) in top_dirs.iter().enumerate() {
        writeln!(
            msg,
            "{}. {} — {} ({:.1}% of total)",
            i + 1,
            dir.path.display(),
            format_size(dir.size),
            pct
        )
        .unwrap();
    }
    writeln!(msg).unwrap();

    // File type breakdown
    writeln!(msg, "--- FILE TYPE BREAKDOWN (top 8 by size) ---").unwrap();
    let categories = aggregate_by_category(&scan.root);
    for cat in categories.iter().take(8) {
        writeln!(
            msg,
            "{}: {} across {} files",
            cat.category,
            format_size(cat.total_size),
            cat.file_count
        )
        .unwrap();
    }
    writeln!(msg).unwrap();

    // Stale files
    writeln!(
        msg,
        "--- STALE FILES (not accessed in {}+ days, top 10 by size) ---",
        stale_days
    )
    .unwrap();
    let stale_threshold = scan.timestamp.saturating_sub(stale_days * 86400);
    let mut stale_files: Vec<&FileEntry> = Vec::new();
    collect_stale_files(&scan.root, stale_threshold, &mut stale_files);
    stale_files.sort_by(|a, b| b.size.cmp(&a.size));
    stale_files.truncate(10);
    if stale_files.is_empty() {
        writeln!(msg, "No stale files found matching criteria.").unwrap();
    } else {
        for f in &stale_files {
            writeln!(
                msg,
                "{} — {}, last accessed {}, last modified {}",
                f.path.display(),
                format_size(f.size),
                epoch_to_string(f.atime),
                epoch_to_string(f.mtime)
            )
            .unwrap();
        }
    }
    writeln!(msg).unwrap();

    // Snapshot diff
    if let Some(diff) = diff {
        writeln!(msg, "--- SNAPSHOT DIFF (vs. {}) ---", epoch_to_string(diff.old_timestamp)).unwrap();
        writeln!(msg, "Total growth: {}", diff.format_total_delta()).unwrap();
        writeln!(msg, "Top 5 grown directories:").unwrap();
        for entry in diff.top_grown(5) {
            let pct = entry
                .old_size
                .map(|o| {
                    if o > 0 {
                        format!("{:.1}", (entry.delta as f64 / o as f64) * 100.0)
                    } else {
                        "∞".to_string()
                    }
                })
                .unwrap_or_else(|| "new".to_string());
            writeln!(
                msg,
                "  {}: +{} ({}%)",
                entry.path,
                format_size(entry.delta as u64),
                pct
            )
            .unwrap();
        }
        writeln!(msg, "New since last snapshot (>100 MB):").unwrap();
        let new_large = diff.new_large(100 * 1024 * 1024);
        if new_large.is_empty() {
            writeln!(msg, "  None.").unwrap();
        } else {
            for entry in new_large {
                writeln!(
                    msg,
                    "  {}: {}",
                    entry.path,
                    format_size(entry.new_size.unwrap_or(0))
                )
                .unwrap();
            }
        }
    }

    msg
}

pub fn export(
    scan: &ScanResult,
    diff: Option<&DiffResult>,
    config: &Config,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(&config.prompt_dir)?;

    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    let filename = format!("prompt_{}.txt", ts);
    let filepath = config.prompt_dir.join(&filename);

    let system = build_system_prompt();
    let user_msg = build_user_message(scan, diff, config.stale_days);

    let mut content = String::new();
    writeln!(content, "[SYSTEM PROMPT]").unwrap();
    writeln!(content, "{}", system).unwrap();
    writeln!(content).unwrap();
    writeln!(content, "[USER MESSAGE]").unwrap();
    writeln!(content, "{}", user_msg).unwrap();

    fs::write(&filepath, &content)?;

    // Prune old prompts
    prune_prompts(&config.prompt_dir, config.max_retained_prompts)?;

    Ok(filepath)
}

fn collect_top_dirs<'a>(
    node: &'a DirNode,
    total: u64,
    result: &mut Vec<(&'a DirNode, f64)>,
) {
    let pct = if total > 0 {
        (node.size as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    result.push((node, pct));
    for child in &node.children {
        if let DirEntry::Dir(d) = child {
            collect_top_dirs(d, total, result);
        }
    }
}

fn collect_stale_files<'a>(
    node: &'a DirNode,
    threshold: u64,
    result: &mut Vec<&'a FileEntry>,
) {
    for child in &node.children {
        match child {
            DirEntry::File(f) => {
                // Both atime and mtime must be below threshold, and size >= 10MB
                if f.atime < threshold && f.mtime < threshold && f.size >= 10 * 1024 * 1024 {
                    result.push(f);
                }
            }
            DirEntry::Dir(d) => collect_stale_files(d, threshold, result),
        }
    }
}

fn prune_prompts(dir: &Path, max: usize) -> std::io::Result<()> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
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
