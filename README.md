# Skillset

Manage AI agent skills in one place and sync them to Cursor, Claude, Codex, Gemini, OpenCode, and more. The **source of truth** is `.skillset/skills` (project) or `~/.skillset/skills` (user-level).

**macOS & Linux** · WSL 2 supported

---

## Quick Start

```bash
curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download
skillset list
```

On first run, a default config is created at `~/.config/skillset/config.json`.

---

## Installation

### Install script (recommended)

```bash
curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download
```

Installs to `~/.local/bin` or `~/bin`—ensure it's on your PATH.

**From a clone:**
```bash
./install.sh --download                    # Download release binary
./install.sh /path/to/skillset             # Use existing binary
./install.sh                               # Use ./target/release/skillset
```

### Cargo

```bash
git clone https://github.com/webteractive/skillset.git && cd skillset
cargo install --path .
```

**Updating:** Re-run the same install command; config and skills are preserved.

---

## How It Works

- **Source:** Skills live in `.skillset/skills` (workspace) or `~/.skillset/skills` (user-level).
- **Targets:** Config lists where to sync (e.g. `~/.cursor/skills`, `~/.claude/skills`).
- **Scope:** Use `--user` or `-G` to operate on user-level; otherwise, workspace.

---

## Commands

| Command | Description |
|---------|-------------|
| `skillset list` | Show skills and their status per target |
| `skillset sync` | Copy skills from source to selected targets |
| `skillset install <owner/repo>` | Install skills from a GitHub repo |
| `skillset add <name>` | Scaffold a new skill with template |
| `skillset remove <name>` | Remove skill from targets (optionally from source) |
| `skillset doc --agents-md` | Output AGENTS.md snippet |

**Common flags:** `--user` / `-G` (user-level), `--sync` (with install), `--yes` (skip prompts)

#### `install`

Install skills from a GitHub repo. Package format: `owner/repo` (HTTPS by default) or a full Git URL. Skillset clones via git, so **if you can `git clone` a repo, you can install from it**—no extra auth. Your existing SSH keys, credential helper, or PAT in the URL all work as usual.

- **Public repos:** `skillset install owner/repo`
- **Private repos:** Use the same URL you’d use for `git clone`. With SSH keys set up, `git@github.com:org/private-repo.git` works. Or `https://github.com/org/private-repo.git` if you use a credential helper or PAT. Set `install.use_ssh: true` in config to make `owner/repo` resolve to SSH by default.

### Examples

```bash
# Workspace vs user-level
skillset list              # Workspace .skillset/skills
skillset list --user       # User-level ~/.skillset/skills

# Install from repos (find more at https://skills.sh)
skillset install webteractive/skillset
skillset install anthropics/skills --skill=frontend-design
skillset install org/repo --sync --user

# Private repos—use whatever URL works with your git setup
skillset install git@github.com:org/private-repo.git
skillset install https://github.com/org/private-repo.git   # if using credential helper or PAT

# Create and sync
skillset add my-skill
skillset sync
```

---

## Configuration

Edit `~/.config/skillset/config.json`:

| Option | Description |
|--------|-------------|
| `source` | Skills directory path (resolved by scope) |
| `targets` | List of `{ label, path }` for sync destinations |
| `install.use_ssh` | Use SSH URLs for `owner/repo` format |
| `install.skill_dirs` | Dirs to search in repos (default: `[".claude/skills", "skills"]`) |

See `config.example.json` for the full default config.

---

## AGENTS.md Integration

```bash
skillset doc --agents-md >> AGENTS.md
```

Tells AI agents where to store skills and how to use the CLI.

---

## Development

```bash
cargo build --release
cargo run -- list
cargo test
```

**Releasing:** Update CHANGELOG, bump version in `Cargo.toml`, run `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test`, then `./release.sh v0.x.x`.

---

## Requirements

- **Rust 1.70+** (when building from source)
- **git** (for `skillset install`)
- macOS, Linux, or WSL 2

## License

MIT
