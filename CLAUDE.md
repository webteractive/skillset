# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Skillset is a Rust CLI that manages AI agent skills across multiple tools (Cursor, Claude Code, Codex, Gemini, Windsurf, OpenCode, GitHub Copilot). Skills live in a single source of truth (`.skillset/skills/` per-workspace or `~/.skillset/skills/` user-level) and get synced to each tool's skills directory.

## Build & Development

```bash
cargo build --release          # release binary
cargo run -- <subcommand>      # run without installing (e.g. cargo run -- list)
cargo test                     # all tests
cargo test <test_name>         # single test
cargo fmt                      # format
cargo clippy -- -D warnings    # lint (must pass before release)
```

## Releasing

1. Update version in `Cargo.toml` and update `CHANGELOG.md`
2. Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
3. Run `./release.sh v0.x.x` — builds binary, creates GitHub release via `gh`, uploads platform-specific asset
4. For additional OS/arch: `./release.sh v0.x.x --upload` on that machine
5. No CI — releases are manual. Always build and upload the binary asset.

## Architecture

### Scope Model

Every command respects a two-tier scope controlled by `--user` / `-G`:
- **Workspace** (default): source is `cwd/.skillset/skills/`, targets use workspace-relative paths
- **User-level** (`--user`): source is `~/.skillset/skills/`, targets use `~/`-prefixed paths

`path::resolve_source()` is the single function that resolves which source directory to use based on scope.

### Data Flow

```
Source (.skillset/skills/)  ──sync──►  Targets (~/.cursor/skills, ~/.claude/skills, etc.)
         ▲                                         │
    install (git clone)                    registry tracks instances
```

- `install` clones repos to `~/Library/Caches/skillset/repos/` (or platform equivalent), copies skills into the source directory only
- `sync` copies from source to selected tool targets — prompts for overwrite unless `--force`
- `registry` (JSON at config dir) tracks where each skill is installed; auto-cleans stale paths on load

### Module Responsibilities

| Module | Role |
|--------|------|
| `main.rs` | CLI parsing (clap derive), dispatches to command functions defined inline |
| `config.rs` | `Config` struct, load/save from `~/.config/skillset/config.json`, `supported_tools()` list, `expand_home()` |
| `skills.rs` | `discover_skills()` (finds dirs with SKILL.md), `copy_skill()`, `sync_skills()` with overwrite policy |
| `install.rs` | `resolve_package()` (git clone/cache), `find_skills_dir()`, `install_package()` |
| `path.rs` | `resolve_source()` — scope-aware path resolution |
| `registry.rs` | `Registry` JSON store — `record()`, `remove_path()`, `where_all()` |
| `add.rs` | Scaffold new skill with template SKILL.md + README.md |
| `remove.rs` | Remove skill from targets and optionally user store |
| `validate.rs` | `parse_frontmatter()` (hand-rolled YAML-ish parser), validate required fields |
| `doc.rs` | Static AGENTS.md snippet for AI agent integration |
| `version_check.rs` | 24h-cached GitHub release check, prints update notice to stderr |

### Key Design Decisions

- **No async** — all I/O is synchronous (`std::fs`, `std::process::Command` for git/curl)
- **Skill discovery** — a skill is any subdirectory containing `SKILL.md`; no other convention required
- **Incremental sync** — skips skills where source and target SKILL.md are byte-identical
- **Install protocol fallback** — `owner/repo` tries SSH first (configurable via `install.use_ssh`), falls back to the other protocol on failure
- **Frontmatter parsing** is hand-rolled (no YAML dependency) — `validate.rs::parse_frontmatter()`
- **Config auto-creates** on first run with all `supported_tools()` as default targets

### Config & State Files

| File | Location | Purpose |
|------|----------|---------|
| Config | `~/.config/skillset/config.json` | Source path, targets, install settings |
| Registry | `~/.config/skillset/registry.json` | Tracks synced skill instances per target |
| Version cache | `~/.config/skillset/.version_check` | Last-checked version + timestamp |
| Repo cache | `~/Library/Caches/skillset/repos/` | Shallow-cloned repos from `install` |
