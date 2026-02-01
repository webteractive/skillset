# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.1]: https://github.com/webteractive/skillset/releases/tag/v0.1.1
[0.1.0]: https://github.com/webteractive/skillset/releases/tag/v0.1.0
