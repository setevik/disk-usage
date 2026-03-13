# DiskWise

A fast, interactive TUI tool that scans directories, presents structured disk usage data, and exports prompts for Claude AI analysis.

Built with Rust, Ratatui, and zero unsafe code.

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/diskwise
```

## Usage

```bash
diskwise [PATH]                    # Scan PATH (default: $HOME), launch TUI
diskwise --ai /home                # Export AI analysis prompt, print claude command
diskwise --save-snapshot /home     # Save scan as compressed snapshot
diskwise --diff <snapshot> /home   # Compare current scan against a saved snapshot
diskwise --export json /home       # Print scan result as JSON to stdout
diskwise --exclude "*.log" /home   # Exclude paths matching glob (repeatable)
diskwise --stale-days 90 /home     # Override stale file threshold (default: 180)
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `1`-`4` | Switch tabs (Tree, Types, Diff, Stale) |
| `j`/`k` or `↑`/`↓` | Navigate |
| `Enter` or `l`/`→` | Expand directory |
| `Backspace` or `h`/`←` | Collapse / go to parent |
| `A` | Toggle AI prompt export panel |
| `?` | Toggle help overlay |
| `q` / `Ctrl+C` | Quit |

## Tabs

1. **Tree** — Collapsible directory tree sorted by size. Dirs in blue, large files in red/yellow.
2. **Types** — File type breakdown bar chart (build artifacts, logs, media, archives, docker, VCS, node_modules, other).
3. **Diff** — Snapshot comparison table showing new, deleted, and changed entries sorted by delta.
4. **Stale** — Files not accessed or modified in N+ days (≥10 MiB), sorted by size.

## AI Prompt Export

DiskWise generates structured prompts for Claude AI analysis — no API key needed.

```bash
# Export prompt and get the command to run
diskwise --ai /home

# Or toggle the AI panel in the TUI with 'A'
```

The exported prompt includes top directories, file type breakdown, stale files, and optional snapshot diffs. Prompts are saved to `~/.local/share/diskwise/prompts/`.

## Snapshots

Snapshots are compressed JSON files stored in `~/.local/share/diskwise/snapshots/`.

```bash
# Save a snapshot
diskwise --save-snapshot /home

# Compare against it later
diskwise --diff ~/.local/share/diskwise/snapshots/home_2026-03-13T14-00-00.json.gz /home
```

## Configuration

Optional config file at `~/.config/diskwise/config.toml`:

```toml
[scan]
default_path = "~"
exclude = ["**/node_modules", "**/.git", "**/target"]
stale_days = 180

[ai]
prompt_dir = "~/.local/share/diskwise/prompts"
max_retained_prompts = 20

[snapshots]
max_retained = 10
```

All settings have sensible defaults. The config file is entirely optional.

## License

MIT
