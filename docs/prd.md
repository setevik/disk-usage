# PRD: DiskWise — AI-Assisted Disk Usage Reporter

**Version:** 0.1  
**Status:** Draft  
**Stack:** Rust · Ratatui TUI · Anthropic Claude API  
**Platform:** Linux workstations (primary)

---

## 1. Problem

Developers and power users on Linux workstations regularly lose track of disk consumption. Existing tools (`du`, `ncdu`, `baobab`) surface raw data but provide no interpretation — the user must manually decide what to clean up, what's safe to delete, and what warrants attention over time. There's no synthesis layer.

---

## 2. Goal

A fast, self-contained TUI tool that scans a target directory, presents structured disk usage data interactively, and uses the Claude API to generate a plain-English summary with prioritized, actionable cleanup suggestions.

---

## 3. Non-Goals (v1)

- No Windows/macOS support
- No remote/network filesystem scanning
- No automated deletion (read-only; recommendations only)
- No cloud storage integration
- No multi-user / daemon mode

---

## 4. User Stories

| # | As a… | I want to… | So that… |
|---|-------|-----------|----------|
| 1 | Developer | See largest dirs/files in a tree view | I know where space is actually going |
| 2 | Developer | View a breakdown by file type | I can identify bloat categories (e.g. logs, build artifacts, media) |
| 3 | Developer | Compare two snapshots over time | I can see what grew and by how much |
| 4 | Developer | Find old / stale files | I can reclaim space from files untouched for months |
| 5 | Developer | Get an AI-generated summary and action items | I don't have to manually interpret the data |

---

## 5. Features (v1 Scope)

### 5.1 Scan Engine
- Recursive directory walker using `walkdir` or native `std::fs`
- Collects: size, last-modified, last-accessed, file extension, inode type
- Excludes: symlinks, `/proc`, `/sys`, `/dev` by default; configurable via `--exclude`
- Outputs a structured in-memory tree, serializable to JSON for snapshot storage

### 5.2 TUI (Ratatui)
- **Tree panel** — collapsible directory tree sorted by size (desc); keyboard nav (`j/k`, `Enter`, `Backspace`)
- **Detail panel** — per-node stats: size, % of parent, file count, last modified
- **Tab bar** — switch between four views:
  - `[1] Tree` — largest dirs/files
  - `[2] Types` — file type breakdown (bar chart by extension group)
  - `[3] Diff` — compare current scan vs. saved snapshot
  - `[4] Stale` — files not accessed/modified in N days (configurable, default 180d)
- **AI panel** — docked bottom drawer, toggled with `A`; shows Claude summary + action items
- **Status bar** — scan progress, total size, last scan timestamp, keybindings hint

### 5.3 Snapshot System
- Snapshots stored as compressed JSON in `~/.local/share/diskwise/snapshots/`
- Named by path + timestamp, e.g. `home_daniel_2025-03-13.json.gz`
- Diff view shows: new entries, deleted entries, size delta per directory
- CLI: `diskwise --save-snapshot` to persist current scan explicitly; last scan auto-saved

### 5.4 AI Summary (Prompt Export)
- Triggered manually via `A` key or `--ai` flag
- Assembles the full structured prompt (see §11) from live scan data
- Writes prompt to a timestamped file: `~/.local/share/diskwise/prompts/prompt_<timestamp>.txt`
- Prints a ready-to-run command to the status bar and to stdout on exit:
  ```
  claude < ~/.local/share/diskwise/prompts/prompt_2025-03-13T14:32:00.txt
  ```
- No API key, no HTTP client, no async required in v1
- v2 upgrade path: swap `PromptBuilder::export()` for `PromptBuilder::send()` against the Claude API — same prompt, different sink

---

## 6. CLI Interface

```
diskwise [PATH]                  # Scan PATH (default: $HOME)
diskwise --ai                    # Run scan + export prompt, print claude CLI command
diskwise --save-snapshot         # Save current scan as a named snapshot
diskwise --diff <snapshot-file>  # Load a specific snapshot for diff view
diskwise --exclude <glob>        # Exclude paths matching glob (repeatable)
diskwise --stale-days <N>        # Override stale threshold (default: 180)
diskwise --export json           # Print scan result as JSON and exit (no TUI)
```

---

## 7. Data Flow

```
[Scan Engine] ──► [In-memory tree]
                        │
           ┌────────────┼────────────────┐
           ▼            ▼                ▼
       [TUI Views]  [Snapshot JSON]  [Prompt Builder]
                                          │
                                  [prompt_<ts>.txt]
                                          │
                              print: "claude < prompt_<ts>.txt"
```

---

## 8. Architecture

| Component | Crate(s) |
|-----------|----------|
| TUI framework | `ratatui`, `crossterm` |
| Directory walking | `walkdir` |
| Snapshot serialization | `serde`, `serde_json`, `flate2` |
| Prompt export | `std::fs` (no HTTP client needed in v1) |
| Config file | `toml`, `dirs` |
| CLI parsing | `clap` |

Single binary, no runtime dependencies, no API key required.

---

## 9. Configuration (`~/.config/diskwise/config.toml`)

```toml
[scan]
default_path = "~"
exclude = ["**/node_modules", "**/.git", "**/target"]
stale_days = 180

[ai]
prompt_dir = "~/.local/share/diskwise/prompts"
max_retained_prompts = 20   # auto-prune oldest beyond this

[snapshots]
max_retained = 10     # auto-prune oldest beyond this
```

---

## 10. Out-of-Scope Risks / Future Considerations

| Item | Notes |
|------|-------|
| Permission errors on restricted dirs | Handle gracefully — skip and report count |
| Very large trees (>1M files) | Cap scan at configurable limit; warn user |
| v2: Claude API direct integration | `PromptBuilder` already decoupled — add `send()` alongside `export()` |
| v2: local LLM (Ollama) | Same abstraction; swap CLI command hint for `ollama run` |
| v2: watch mode | `inotify`-based live updates for the TUI |

---

## 11. AI Prompt Template

The prompt is assembled by `PromptBuilder` from live scan data and sent as a single user message with a structured system prompt.

---

### System Prompt

```
You are a disk usage analyst assistant. You will be given structured data about a Linux filesystem scan.
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
• <anything that looks risky or ambiguous, or "None." if nothing warrants flagging>
```

---

### User Message Template

```
Scan target: {{target_path}}
Scan timestamp: {{timestamp}}
Total scanned size: {{total_size}}
Total files: {{file_count}}

--- TOP 10 LARGEST DIRECTORIES ---
{{#each top_dirs}}
{{rank}}. {{path}} — {{size}} ({{pct_of_total}}% of total)
{{/each}}

--- FILE TYPE BREAKDOWN (top 8 by size) ---
{{#each type_groups}}
{{extension_group}}: {{total_size}} across {{file_count}} files
{{/each}}

--- STALE FILES (not accessed in {{stale_days}}+ days, top 10 by size) ---
{{#each stale_files}}
{{path}} — {{size}}, last accessed {{last_accessed}}, last modified {{last_modified}}
{{/each}}

{{#if has_diff}}
--- SNAPSHOT DIFF (vs. {{snapshot_date}}) ---
Total growth: {{diff_total}}
Top 5 grown directories:
{{#each diff_top_grown}}
  {{path}}: +{{delta_size}} ({{delta_pct}}%)
{{/each}}
New since last snapshot (>100 MB):
{{#each diff_new_large}}
  {{path}}: {{size}}
{{/each}}
{{/if}}
```

---

### Variable Definitions

| Variable | Source | Notes |
|----------|--------|-------|
| `top_dirs` | Scan tree, sorted by size desc | Directories only; excludes excluded paths |
| `type_groups` | Extension → category mapping | Groups like `build_artifacts` = `.o .a .rlib target/`; `logs` = `.log .log.*`; `media` = `.mp4 .mkv .png` etc. |
| `stale_files` | Files where `atime` and `mtime` both exceed threshold | Only includes files ≥ 10 MB to reduce noise |
| `has_diff` | Boolean — true if a previous snapshot exists for the same path | Diff block omitted entirely if no prior snapshot |
| `diff_top_grown` | Directories with largest positive delta | Capped at 5 entries |

---

### Extension → Category Mapping (hardcoded, v1)

| Category Label | Extensions / Patterns |
|---|---|
| `build_artifacts` | `.o`, `.a`, `.rlib`, `.rmeta`, `target/`, `dist/`, `.pyc`, `__pycache__/` |
| `logs` | `.log`, `.log.*`, `.gz` inside log dirs |
| `media` | `.mp4`, `.mkv`, `.avi`, `.mov`, `.png`, `.jpg`, `.psd`, `.raw` |
| `archives` | `.zip`, `.tar`, `.tar.gz`, `.tar.xz`, `.7z`, `.deb`, `.AppImage` |
| `docker` | `overlay2/`, `volumes/` under `/var/lib/docker` |
| `vcs` | `.git/objects/`, `.git/lfs/` |
| `node_modules` | `node_modules/` (always called out separately regardless of size) |
| `other` | Everything else |

---

## 12. Success Metrics (v1)

- Scans a typical `$HOME` (~100k files) in under **5 seconds**
- AI summary generated in under **8 seconds** (streaming first token < 2s)
- Zero unsafe Rust; passes `cargo clippy` clean
- Usable without reading docs — keybindings visible in status bar at all times
