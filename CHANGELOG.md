# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3] - 2026-04-10

### Fixed

- **Self-update version detection:** Replaced silent failure chain with explicit error handling — no more false "Update complete" when the update didn't work.
- **Self-update curl reliability:** Downloads install script to temp file instead of piping (`curl | sh`), so network failures actually surface.
- **Self-update temp file security:** Uses `mktemp` instead of predictable `/tmp` path to prevent symlink attacks.
- **Registry error handling:** Registry write failures now warn to stderr instead of being silently swallowed via `.ok()`.
- **Corrupted registry recovery:** Malformed `registry.json` now warns and preserves the original file instead of silently overwriting it with empty data.
- **User scope without HOME:** `--user` flag now fails with a clear error when `HOME` is not set, instead of silently falling back to workspace scope.
- **Version cache writes:** `version_check` warns on write failures instead of silently discarding errors (which caused re-fetching from GitHub on every command).
- **Diff display:** `show_diff` warns on file read errors instead of showing misleading empty-file diffs.
- **Skill discovery:** Warns on non-UTF-8 directory names instead of silently dropping valid skills.
- **install.sh API curl:** Added `-f` flag so HTTP errors (rate limiting, 404) fail instead of producing garbage version strings.

### Changed

- **Release script rewrite:** `release.sh` now accepts `patch|minor|major` instead of raw version tags. Auto-bumps `Cargo.toml`, commits, tags, builds, and releases.
- **Release script safety:** Added dirty-tree guard, version validation, post-sed verification, portable sed, atomic git push, push failure recovery instructions, and smarter `gh release` error handling.

## [0.2.2] - 2026-03-31

### Changed

- **Default to SSH for git clone:** `owner/repo` installs now try SSH first, falling back to HTTPS on failure. Configurable via `install.use_ssh`.

## [0.2.1] - 2026-03-29

### Added

- **Version check notification:** npm-style update notice shown in interactive terminals when a newer release is available. Checks GitHub API at most once every 24 hours, cached locally.

## [0.2.0] - 2026-03-29

### Added

- **`skillset self-update`** command to update to the latest release.
- **`skillset where`** command with registry tracking — shows where each skill is synced across targets. `--scan` to register existing instances.
- **`skillset validate`** command to check SKILL.md frontmatter for required fields.
- **`skillset config`** command to manage configuration (show, add/remove targets, reset, validate-paths).
- **`skillset completions`** for bash, zsh, and fish shell completions.
- **`--force`** flag to skip all interactive prompts during sync/install.
- **`--from-remote`** flag on install to pull latest from cached repos.
- **`--dry-run`** flag for sync and install to preview changes without writing.
- **`--diff`** flag for sync to show unified diffs before overwriting.
- **`--filter`** and **`--status`** flags for list to filter skills by name or sync status.
- **Incremental sync** — skips skills where SKILL.md is byte-identical at source and target.

## [0.1.6] - 2026-02-04

### Added

- **`skillset list --tool`** flag that shows skills already loaded for a specific tool (e.g., `skillset list --tool=codex`). Works with `-G/--user` to target either workspace or user-level skill directories.

## [0.1.5] - 2025-02-02

### Added

- **Short alias `-G`** for `--user` flag (e.g. `skillset list -G`, `skillset sync -G`).
- **Version flag** `--version` and `-V` to show current version.
- **Install next-steps message** when running `install` without `--sync`: prints guidance to run `skillset sync` (or `skillset sync --user` / `skillset sync -G`) to copy skills to configured tools.

## [0.1.4] - 2025-02-02

### Changed

- **Install skill dirs:** Default lookup order is now `.claude/skills`, then `skills` (Anthropic convention first). Previously checked `.cursor/skills` first.

### Added

- **Config `install.skill_dirs`:** Customize which dirs to look for skills in when installing. Default: `[".claude/skills", "skills"]`.
- **Flag `--dir`:** Override skill dirs for a single install (comma-separated, e.g. `--dir=.cursor/skills,.claude/skills`).

## [0.1.3] - 2025-02-02

### Added

- **Private repo install:** Full Git URLs now supported (`git@github.com:org/repo.git`, `https://...`, `ssh://...`). Use for private repos or non-GitHub hosts.
- **Config option `install.use_ssh`:** When `true`, uses SSH URLs for `owner/repo` format. Add to `~/.config/skillset/config.json` for private GitHub repos when SSH keys are configured.

## [0.1.2] - 2025-02-01

### Changed

- **Sync scope:** Without `--user`, sync now writes to workspace tool dirs (e.g. `.cursor/skills`, `.claude/skills` in the project). With `--user`, sync writes to user-level dirs (`~/.cursor/skills`, etc.). Same tool list in both cases; path is chosen by scope so skills are never written to the wrong level.

### Added

- Unit tests for `add` module: valid name creates files, empty/invalid name errors, existing without force errors, existing with force overwrites.

### Fixed

- Install/sync without `--user` was writing to user-level targets (~/.cursor/skills etc.). Now correctly writes only to workspace paths when scope is workspace.

## [0.1.1] - 2025-02-01

### Changed

- Clippy fixes (`expand_home` strip_prefix, `find_skills_dir` &Path).
- README: updating instructions, first-run wording, pre-release checklist.

### Added

- CHANGELOG.md. Updating section in README.

## [0.1.0] - 2025-02-01

### Added

- **list** – List skills in source (workspace or user) and show status per target (present/missing).
- **sync** – Copy skills from source to configured targets. Interactive checklist to select which tools to sync to (Cursor, Claude Code, Windsurf, Codex, OpenCode, Gemini, GitHub Copilot).
- **install** – Install skills from a GitHub repo (e.g. `anthropics/skills`). Optional `--skill=NAME`, `--user` (user-level store), and `--sync` (run sync after install with same checklist).
- **add** – Scaffold a new skill with template `SKILL.md` and optional `README.md` in `.skillset/skills` (workspace or `~/.skillset/skills` with `--user`).
- **remove** – Remove a skill from all configured targets; optional `--user` to also remove from user store, `--yes` to skip confirmation.
- **doc** – Output AGENTS.md snippet (`skillset doc --agents-md`) for AI agents.
- Config at `~/.config/skillset/config.json` with `source` and `targets`. Default source: `.skillset/skills`; default targets: Cursor, Claude Code, Windsurf, Codex, OpenCode, Gemini, GitHub Copilot (project and personal).
- Scope: `--user` for user-level (`~/.skillset/skills`); otherwise workspace (`./.skillset/skills`).
- Release script `release.sh` for building and creating GitHub releases with OS/arch-specific binaries.

### Platform

- macOS and Linux only. Windows is not supported.

[0.2.3]: https://github.com/webteractive/skillset/releases/tag/v0.2.3
[0.2.2]: https://github.com/webteractive/skillset/releases/tag/v0.2.2
[0.2.1]: https://github.com/webteractive/skillset/releases/tag/v0.2.1
[0.2.0]: https://github.com/webteractive/skillset/releases/tag/v0.2.0
[0.1.6]: https://github.com/webteractive/skillset/releases/tag/v0.1.6
[0.1.5]: https://github.com/webteractive/skillset/releases/tag/v0.1.5
[0.1.4]: https://github.com/webteractive/skillset/releases/tag/v0.1.4
[0.1.3]: https://github.com/webteractive/skillset/releases/tag/v0.1.3
[0.1.2]: https://github.com/webteractive/skillset/releases/tag/v0.1.2
[0.1.1]: https://github.com/webteractive/skillset/releases/tag/v0.1.1
[0.1.0]: https://github.com/webteractive/skillset/releases/tag/v0.1.0
