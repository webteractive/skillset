# Skillset

A small CLI that lets you **manage** AI agent skills in one place and **load them** on the tools you use (Cursor, Claude, Codex, Gemini, OpenCode, etc.). The **source of truth** for skills is **`.ai/`** (e.g. `.ai/skills` in a project or `~/.ai/skills` globally).

## Supported Platforms

**macOS and Linux only.** Windows is not supported.

## Installation

### From source

```bash
cd skillset
cargo install --path .
```

### From binary

Download the pre-built binary for macOS or Linux from releases and add it to your PATH.

## First Run

On first run, Skillset creates a default config at `~/.config/skillset/config.json` with Cursor as the default target. The config path is printed on first run.

```bash
skillset list
```

## Configuration

Edit `~/.config/skillset/config.json` to customize:

```json
{
  "source": ".ai/skills",
  "targets": [
    { "label": "Cursor", "path": "~/.cursor/skills" },
    { "label": "Claude", "path": "~/.claude/skills" }
  ]
}
```

- `source`: Path template for skills directory (resolved relative to cwd or `~/.ai/skills` with `--user`)
- `targets`: List of tool directories where skills should be synced

## Usage

### Scope

By default, commands operate on the workspace source: `./.ai/skills` (current working directory).

Use `--user` to operate on the user-level source: `~/.ai/skills`.

```bash
skillset list          # List workspace skills
skillset list --user   # List user-level skills
```

### Commands

#### `list`

Show skills in source and their status per target (present/missing).

```bash
skillset list
skillset list --user
```

Example output:

```
Source: /path/to/.ai/skills
Config: /Users/username/.config/skillset/config.json

Skills:
  documan  cursor ✓  claude —
  install-skills  cursor ✓  claude —
```

#### `sync`

Copy skills from source to all configured targets. Prompts for existing skills.

```bash
skillset sync
skillset sync --user
```

For each skill that already exists at a target, you'll be prompted:

```
Skill 'documan' already exists at cursor. Overwrite? [y/n/all]
```

#### `install <vendor/package>`

Install skills from a GitHub repository (e.g., `anthropics/skills`).

```bash
# Install all skills from a package
skillset install anthropics/skills

# Install a specific skill
skillset install anthropics/skills --skill=frontend-design

# Install and add to user-level store
skillset install anthropics/skills --user
```

The package is cloned to `~/.cache/skillset/repos/owner-repo`.

#### `add <name>`

Scaffold a new skill with a template `SKILL.md` (and optional `README.md`).

```bash
# Add to workspace
skillset add my-skill

# Add to user-level store
skillset add my-skill --user

# Overwrite if exists
skillset add my-skill --force
```

After adding or editing skills, run `skillset sync` (or `skillset sync --user`) to load them to configured tools.

#### `remove <name>`

Remove a skill from all configured targets (and optionally from user store).

```bash
# Remove from targets only
skillset remove documan

# Remove from targets and user store
skillset remove documan --user

# Skip confirmation
skillset remove documan --yes
```

Prompts before deleting unless `--yes` is used.

#### `doc`

Output documentation snippets.

```bash
skillset doc --agents-md
```

Prints the AGENTS.md snippet for AI coding agents (see below).

## AGENTS.md Integration

Add this snippet to your project or user AGENTS.md so AI agents know where to store generated skills and how to use Skillset:

```bash
skillset doc --agents-md >> AGENTS.md
```

## Development

### Build

```bash
cd skillset
cargo build --release
```

The binary will be at `target/release/skillset`.

### Run

```bash
cargo run -- --help
cargo run -- list
```

### Test

```bash
cargo test
```

## Requirements

- **Rust 1.70+** for building from source
- **git** for `skillset install` command (to clone packages)
- macOS or Linux

## License

MIT
