---
name: use-skillset
description: Use the Skillset CLI to manage and sync AI agent skills. Use when the user asks to install skills from a repo (e.g. skillset install webteractive/skillset), sync skills to Cursor/Claude/Codex/etc., add or remove skills, list skills, or when working with .skillset/skills or ~/.skillset/skills.
---

# Using Skillset

Skillset is a CLI that manages AI agent skills and syncs them to tools (Cursor, Claude Code, Windsurf, Codex, OpenCode, Gemini, GitHub Copilot). Source of truth is **`.skillset/skills`** (workspace) or **`~/.skillset/skills`** (user-level).

## When to use

Apply this skill when:

- The user says "skillset install …", "install skills from …", or wants to install skills from a GitHub repo (e.g. `webteractive/skillset`)
- The user wants to sync skills to Cursor, Claude, Codex, or other configured tools
- The user asks to add, remove, or list skills, or to use the skillset CLI
- The context involves `.skillset/skills`, `~/.skillset/skills`, or skill directories for AI tools

## Scope

| Flag | Source / destination |
|------|----------------------|
| (none) | Workspace: `./.skillset/skills` (relative to cwd) |
| `--user` / `-G` | User-level: `~/.skillset/skills` |

Default is workspace. Use `--user` or `-G` for global install/list/sync/add/remove.

## Commands

### `skillset list`

Show skills in source and status per target (present/missing).

```bash
skillset list
skillset list --user   # or skillset list -G
```

### `skillset sync`

Copy skills from source to configured targets. Shows a **checklist** of supported tools (from your config); choose which targets to sync to (e.g. `1,3,5` or `all`). Then prompts for any skill that already exists at a selected target.

```bash
skillset sync
skillset sync --user   # or skillset sync -G
```

Example checklist (press Enter or type `all` to sync to every target):

```
Sync skills to (supported tools):
  [1] Cursor  (~/.cursor/skills)
  [2] Claude Code  (~/.claude/skills)
  ...
Select targets (e.g. 1,3,5 or 'all') [all]:
```

For each skill that already exists at a selected target, you'll be prompted: `Skill 'my-skill' already exists at cursor. Overwrite? [y/n/all]`

### `skillset install <owner/repo>`

Install skills from a GitHub repository into the **source of truth only** (workspace or user store). Does not copy to AI tool dirs; use `skillset sync` or `install --sync` for that.

```bash
# Install all skills from a package
skillset install webteractive/skillset

# Install a specific skill
skillset install webteractive/skillset --skill=use-skillset

# Install and sync to all configured targets
skillset install webteractive/skillset --sync

# Install to user-level store
skillset install webteractive/skillset --user   # or skillset install webteractive/skillset -G
```

Package is cloned to `~/.cache/skillset/repos/owner-repo` (or `~/Library/Caches/skillset/repos/` on macOS). The repo must contain a skills directory at **`.cursor/skills`** or **`skills`** at repo root; each skill is a subdirectory with a `SKILL.md` file.

### `skillset add <name>`

Scaffold a new skill with template `SKILL.md` (and optional `README.md`).

```bash
skillset add my-skill
skillset add my-skill --user   # or skillset add my-skill -G
skillset add my-skill --force   # overwrite if exists
```

### `skillset remove <name>`

Remove a skill from all configured targets (and optionally from user store).

```bash
skillset remove my-skill
skillset remove my-skill --user   # or skillset remove my-skill -G
skillset remove my-skill --yes   # skip confirmation
```

### `skillset doc`

Output documentation snippets (e.g. for AGENTS.md).

```bash
skillset doc --agents-md
```

## Configuration

Config path: `~/.config/skillset/config.json`. Contains:

- **source**: Path template for skills directory (e.g. `.skillset/skills`)
- **targets**: List of `{ "label": "Cursor", "path": "~/.cursor/skills" }` etc.

Default targets include Cursor, Claude Code, Windsurf, Codex, OpenCode, Gemini, GitHub Copilot (project and personal). Skills are copied into these paths when running `skillset sync` or `skillset install … --sync`.

## Workflow: Install from GitHub and use in Cursor

1. **Install** into user store and sync to tools:
   ```bash
   skillset install webteractive/skillset --user --sync   # or -G --sync
   ```
   Or install to current workspace then sync:
   ```bash
   skillset install webteractive/skillset
   skillset sync
   ```

2. **List** to verify:
   ```bash
   skillset list --user   # or skillset list -G
   ```

3. After editing skills in source, run **sync** again to update tools:
   ```bash
   skillset sync --user   # or skillset sync -G
   ```

## Requirements

- **macOS or Linux** (Windows not supported)
- **git** for `skillset install` (clones packages)
- Binary on PATH (e.g. `~/.local/bin/skillset` or `~/.cargo/bin/skillset`)

## Paths

| Role   | Path |
|--------|------|
| Workspace source | `.skillset/skills` (relative to cwd) |
| User source      | `~/.skillset/skills` |
| Config           | `~/.config/skillset/config.json` |
| Install cache    | `~/.cache/skillset/repos/` (Linux) or `~/Library/Caches/skillset/repos/` (macOS) |
