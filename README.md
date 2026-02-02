# Skillset

A small CLI that lets you **manage** AI agent skills in one place and **load them** on the tools you use (Cursor, Claude, Codex, Gemini, OpenCode, etc.). The **source of truth** for skills is **`.skillset/`** (e.g. `.skillset/skills` in a project or `~/.skillset/skills` globally).

## Supported Platforms

**macOS and Linux.** WSL 2 is supported (it runs Linux). Native Windows (cmd/PowerShell) is not supported.

## Installation

### Without Cargo (install script)

**Download and install** (no cargo, no clone):

```bash
curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download
```

Or clone and run:

```bash
# Download latest release from GitHub and install
./install.sh --download

# Or use a local binary (after downloading from a release, or after building):
./install.sh skillset-darwin-arm64
./install.sh                    # uses ./target/release/skillset (after cargo build --release)
./install.sh /path/to/skillset
```

The script installs to `~/.local/bin` or `~/bin`; ensure that directory is on your PATH.

### Updating

- **If you installed via the install script** (download or a release binary): run the same command again. It overwrites the existing binary.
  ```bash
  curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download
  ```
  Or from a clone: `./install.sh --download`.
- **If you installed via Cargo** (from the repo): pull the latest code and reinstall:
  ```bash
  cd skillset
  git pull
  cargo install --path .
  ```
  Cargo overwrites the previous binary in `~/.cargo/bin/skillset`.

Your config (`~/.config/skillset/config.json`) and skills (e.g. `~/.skillset/skills`) are left unchanged when you update the binary.

### With Cargo

```bash
git clone https://github.com/webteractive/skillset.git
cd skillset
cargo install --path .
```

Or build and install locally:

```bash
cargo build --release
./install.sh
```

## First Run

On first run, Skillset creates a default config at `~/.config/skillset/config.json` with all supported tools as default targets. The config path is printed on first run.

```bash
skillset list
```

## Configuration

Edit `~/.config/skillset/config.json` to customize:

```json
{
  "source": ".skillset/skills",
  "targets": [
    { "label": "Cursor", "path": "~/.cursor/skills" },
    { "label": "Claude Code", "path": "~/.claude/skills" },
    { "label": "Windsurf", "path": "~/.windsurf/skills" },
    { "label": "Codex", "path": "~/.codex/skills" },
    { "label": "OpenCode", "path": "~/.opencode/skills" },
    { "label": "Gemini", "path": "~/.gemini/skills" },
    { "label": "GitHub Copilot (project)", "path": ".github/skills" },
    { "label": "GitHub Copilot (personal)", "path": "~/.copilot/skills" }
  ]
}
```

- `source`: Path template for skills directory (resolved relative to cwd or `~/.skillset/skills` with `--user`)
- `targets`: List of tool directories where skills should be synced. Supported tools (skills-capable CLIs/editors): **Cursor**, **Claude Code**, **Windsurf**, **Codex**, **OpenCode**, **Gemini**, **GitHub Copilot** (project: `.github/skills`, personal: `~/.copilot/skills`). New configs default to all; remove or add paths as needed.
- `install.use_ssh`: When `true`, use SSH URLs (`git@github.com:owner/repo.git`) for `owner/repo` package specs. Useful for private repos when SSH keys are configured. Default: `false`.

## Usage

### Scope

By default, commands operate on the workspace source: `./.skillset/skills` (current working directory).

Use **`--user`** to target the user-level dir **`~/.skillset/skills`** (source of truth and install destination); without it, commands use the workspace **`.skillset/skills`**.

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
Source: /path/to/.skillset/skills
Config: /Users/username/.config/skillset/config.json

Skills:
  my-skill  cursor ✓  claude —
  use-skillset  cursor ✓  claude —
```

#### `sync`

Copy skills from source to configured targets. Shows a **checklist** of supported tools (from your config); choose which targets to sync to (e.g. `1,3,5` or `all`). Then prompts for any skill that already exists at a target.

```bash
skillset sync
skillset sync --user
```

Example checklist (press Enter or type `all` to sync to every target):

```
Sync skills to (supported tools):
  [1] Cursor  (~/.cursor/skills)
  [2] Claude Code  (~/.claude/skills)
  ...
Select targets (e.g. 1,3,5 or 'all') [all]:
```

For each skill that already exists at a selected target, you'll be prompted:

```
Skill 'my-skill' already exists at cursor. Overwrite? [y/n/all]
```

#### `install <vendor/package>`

Install skills from a GitHub repository into the **source of truth only** (workspace or user store). Does not copy to AI tool dirs (Cursor, Claude Code, etc.); use **`skillset sync`** or **`install --sync`** to copy to tools.

Packages must have a skills directory at **`.cursor/skills`** or **`skills`** at repo root; this repo provides a **use-skillset** skill (how to use the CLI) at `skills/use-skillset/`.

**Package spec format:**

- **`owner/repo`** — GitHub shorthand (e.g., `webteractive/skillset`). Uses HTTPS by default.
- **Full Git URL** — For private repos or non-GitHub hosts: `git@github.com:org/repo.git`, `https://github.com/org/repo.git`, or `ssh://git@gitlab.com/org/repo.git`. Use whatever URL works with your git credentials.

**Private repositories:**

```bash
# Use full SSH URL (works with SSH keys)
skillset install git@github.com:org/private-repo.git

# Use full HTTPS URL (works with credential helper or PAT in URL)
skillset install https://github.com/org/private-repo.git

# Or set "install": { "use_ssh": true } in config.json to use SSH for owner/repo format
skillset install org/private-repo
```

```bash
# Install all skills from a package (including this repo's use-skillset skill)
skillset install webteractive/skillset
skillset install anthropics/skills

# Install a specific skill
skillset install webteractive/skillset --skill=use-skillset
skillset install anthropics/skills --skill=frontend-design

# Install and sync to all configured targets (Cursor, Claude Code, Windsurf, Codex, OpenCode, Gemini, GitHub Copilot)
skillset install anthropics/skills --sync

# Install and add to user-level store
skillset install anthropics/skills --user
```

Without `--user`, skills are copied to the workspace source (`./.skillset/skills`), creating it if needed. With `--user`, skills go to the user-level store (`~/.skillset/skills`) only. To copy into Cursor, Claude Code, etc., run **`skillset sync`** (or **`skillset install ... --sync`**) after installing.

The package is cloned to `~/.cache/skillset/repos/owner-repo` (or `~/Library/Caches/skillset/repos/` on macOS). Full URLs use a hashed subdir name.

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
skillset remove my-skill

# Remove from targets and user store
skillset remove my-skill --user

# Skip confirmation
skillset remove my-skill --yes
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

## Releasing (maintainers)

Before cutting a release:

1. Update **CHANGELOG.md** with the new version and changes.
2. Bump **version** in `Cargo.toml` if needed.
3. Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, and `cargo build --release`.

Then build and publish the binary to GitHub via `gh`:

```bash
# Create release and upload binary for this OS/arch (requires gh and gh auth login)
./release.sh v0.1.0
```

Asset names are per platform (e.g. `skillset-darwin-arm64`, `skillset-linux-x86_64`) so you can add more from other machines:

```bash
# On another machine (e.g. Linux): build and upload to the same release
./release.sh v0.1.0 --upload
```

If the release already exists (e.g. re-running on the same machine), the script uploads the asset with `--clobber`.

## Requirements

- **Rust 1.70+** for building from source
- **git** for `skillset install` command (to clone packages)
- macOS, Linux, or WSL 2

## License

MIT
