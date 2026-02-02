# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[0.1.4]: https://github.com/webteractive/skillset/releases/tag/v0.1.4
[0.1.3]: https://github.com/webteractive/skillset/releases/tag/v0.1.3
[0.1.2]: https://github.com/webteractive/skillset/releases/tag/v0.1.2
[0.1.1]: https://github.com/webteractive/skillset/releases/tag/v0.1.1
[0.1.0]: https://github.com/webteractive/skillset/releases/tag/v0.1.0
