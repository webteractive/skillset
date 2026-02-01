# Product Requirements Document: Skillset

**Working title:** Skillset  
**Version:** 0.2  
**Last updated:** 2025-02-01  

---

## 1. Overview

A small CLI that lets you **manage** AI agent skills in one place and **load them** on the tools you use (Cursor, Claude, Codex, Gemini, OpenCode, etc.). The **source of truth** for skills is **`.skillset/`** (e.g. `.skillset/skills` in a project or `~/.skillset/skills` globally). The app syncs from there into each tool’s skills directory when you ask. No TUI required—just commands like `skillset list` and `skillset sync`.

---

## 2. Problem

- **Many tools, many skills dirs.** Each AI tool has its own skills directory (e.g. `~/.cursor/skills`, `~/.claude/skills`). Keeping the same skills available everywhere is manual and easy to forget.
- **No single place to manage and load.** You have to remember paths and copy (or script) yourself. You want: one place to manage skills, and a simple way to load them onto the tools you actually use.

---

## 3. Goals

| Goal | Description |
|------|-------------|
| **Single source of truth** | Skills live under **`.skillset/`** (e.g. `.skillset/skills` in a project or `~/.skillset/skills`). You edit there; the app syncs from that directory to your tools. |
| **Load on the tools you use** | Configure which tool dirs you use (Cursor, Claude, etc.). One command copies your skills into those dirs so each tool sees them. |
| **Manage, don’t overwhelm** | List what you have, see where it’s loaded, sync when you want. Simple CLI; no full-screen UI required. |
| **Optional: create skills** | Later, maybe: create a new skill from a template (name, description, triggers) so you don’t start from a blank file. |

---

## 4. Users

- **Primary:** You (and others) who use more than one AI tool and want to manage skills once and load them on the tools you use.
- **Secondary:** Anyone who wants a single, scriptable way to “install” or “sync” skills to multiple tools without memorizing paths.

---

## 5. Scope

### 5.1 In scope (MVP)

- **Config:** One config file (e.g. `~/.config/skillset/config.json`) with:
  - **Source:** path to your canonical skills directory (can be overridden by scope; see **`--user`** below). User can override in config.
  - **Targets:** list of tool dirs (e.g. Cursor, Claude) with label + path. `~` expanded at runtime.
- **Local vs user scope:** **`--user`** selects **user-level** scope: source is **`~/.skillset/skills`** (user’s home). Without `--user`, scope is **workspace/local**: source is the **current working directory (cwd)**, e.g. **`./.skillset/skills`**. So: `skillset list` and `skillset sync` use cwd’s `.skillset/skills` by default; `skillset list --user` and `skillset sync --user` use `~/.skillset/skills`.
- **List:** Command to list skills found in the source and, per target, whether each skill is present or missing (e.g. `skillset list` or `skillset status`). Respects `--user` for source.
- **Sync:** Command to copy skills from source to selected targets (e.g. `skillset sync`). Respects `--user` for source. For skills that already exist at a target, prompt overwrite or skip (per skill or “all”).
- **Install:** Command to install skills from a **vendor/package** into configured targets (and optionally into user-level store; see `--user`).
  - **Invocation:** `skillset install <vendor/package>` (e.g. `skillset install anthropics/skills`). The vendor/package argument is **required** (e.g. GitHub-style `owner/repo` such as `anthropics/skills`). Skillset resolves it to a skills directory (e.g. from that repo’s `.cursor/skills` or `skills` dir).
  - **What to install:** If **`--skill=NAME`** is provided (e.g. `skillset install anthropics/skills --skill=frontend-design`), install only that skill. If **`--skill=` is not provided**, install **all** available skills from that package. Multiple skills can be supported later (e.g. `--skill=frontend-design,backend-api`).
  - **`--user`:** If set, also copy installed skills into **`~/.skillset/skills`** (user-level store) so they persist and can be synced later with `skillset sync --user`. Without `--user`, install only copies to configured tool targets (e.g. `~/.cursor/skills`).
  - Same overwrite/skip behavior as sync when a skill already exists at a target.
- **Add:** Command to scaffold a new skill in the appropriate store so the AI (or user) knows where to put generated skills and can use the skillset CLI.
  - **Invocation:** `skillset add <name>` (e.g. `skillset add my-skill`). Creates **`.skillset/skills/<name>/`** (workspace, default) or **`~/.skillset/skills/<name>/`** with `--user`, and writes a template **`SKILL.md`** (and optionally `README.md`). The AI or user then edits the file. After adding or editing skills, run **`skillset sync`** (or `skillset sync --user`) to load them to configured tools.
  - **`--user`:** If set, create the skill under **`~/.skillset/skills/<name>/`** (user-level store). Without `--user`, create under **`./.skillset/skills/<name>/`** (workspace). Fails if the skill already exists unless overwrite is confirmed or a flag is used.
- **AGENTS.md AI instruction:** Skillset provides a **standard snippet** that can be added or appended to **AGENTS.md** so that AI coding agents (Cursor, Claude, Codex, etc.) know: (1) **Where to store generated skills:** under **`.skillset/skills/<name>/`** (workspace) or **`~/.skillset/skills/<name>/`** (user-level). (2) **How to use the skillset CLI:** run **`skillset add <name>`** to scaffold a new skill in that location, then edit `SKILL.md`; after creating or updating skills, run **`skillset sync`** (or `skillset sync --user`) to load them to the configured tools. The snippet is intended to be copied into project or user AGENTS.md so the agent consistently stores and syncs skills. Optionally, a command (e.g. **`skillset doc agents-md`**) outputs this snippet for the user to append to AGENTS.md.
- **Remove:** Command to remove a skill by name from configured targets (and optionally from user-level store).
  - **Invocation:** `skillset remove <name>` (e.g. `skillset remove documan`). Removes the skill directory from **all configured targets** (e.g. deletes `~/.cursor/skills/documan`, `~/.claude/skills/documan`). Does not delete from the source (workspace or user store) unless `--user` is used.
  - **`--user`:** If set, also remove the skill from **`~/.skillset/skills/<name>`** (user-level store). Without `--user`, only targets are affected; the skill remains in cwd’s `.skillset/skills` or `~/.skillset/skills` and can be re-synced later.
  - Prompt for confirmation before deleting (e.g. “Remove documan from cursor, claude? [y/n]”) or support `--yes` to skip prompt.
- **CLI only:** Subcommands + prompts where needed (e.g. “Overwrite X? [y/n/all]”, “Remove X? [y/n]”). No full-screen TUI required for MVP.
- **Distribution:** Single binary (`skillset`) on macOS and Linux. Windows is not supported.

### 5.2 Later (post-MVP)

- **Create from template (rich):** `skillset new` — interactive or flags for name, description, trigger phrases; generate a fuller skill dir + `SKILL.md` from a template (add builds on this).
- **LLM-assisted generation:** Optional “describe what you want” → draft `SKILL.md` via API; edit in your editor.
- **Outdated / diff:** Show when a target’s copy is older than source; optional “sync only outdated.”
- **Validation:** Basic check of `SKILL.md` (e.g. frontmatter) and report errors.
- **TUI (optional):** If you ever want a full-screen UI, it can be added on top of the same “manage + load” logic.

### 5.3 Out of scope

- **Windows:** Skillset is not supported on Windows; target platforms are macOS and Linux only.
- Editing skill content inside the app (use your editor).
- Cloud sync, user accounts, or a marketplace.
- Replacing or competing with existing CLIs (e.g. Skills CLI); this can be a minimal “manage + load” tool for your own workflow (and a Rust learning project).

---

## 6. User flows (CLI)

### 6.1 First run / config

1. User runs `skillset list` (or `skillset status`). If no config exists, app creates a default config with common targets (e.g. `~/.cursor/skills`). App tells user where the config is. Source is determined by scope: cwd’s `./.skillset/skills` by default, or `~/.skillset/skills` with `--user`.
2. User edits config if they want different targets or a custom source.

### 6.2 See what you have and where it’s loaded

1. User runs `skillset list` (or `skillset status`). Without `--user`, source = **cwd** (e.g. `./.skillset/skills`). With `--user`, source = **`~/.skillset/skills`**.
2. App reads source, lists each skill (dir with `SKILL.md`). For each target, shows whether that skill is present or missing (e.g. table or lines like `documan  cursor ✓  claude —`).

### 6.3 Load skills onto your tools

1. User runs `skillset sync` (workspace source) or `skillset sync --user` (user-level source `~/.skillset/skills`).
2. App copies each skill from source to every configured target (or to targets selected via flags, if supported). If a skill already exists at a target, prompt: overwrite, skip, or “all” for the rest.
3. When done, user can run `skillset list` (or `skillset list --user`) again to confirm.

### 6.4 Install skills from a vendor/package

1. **Install all skills from a package (to tool targets only):**
   - `skillset install anthropics/skills` — resolve `anthropics/skills` (e.g. to that repo’s skills dir), copy every skill to all configured targets (e.g. `~/.cursor/skills`). Prompt overwrite/skip for existing. Source scope (cwd vs user) does not change where the package is read from; it can affect config. Install always writes to configured targets.
2. **Install only one skill:**
   - `skillset install anthropics/skills --skill=frontend-design` — same package, copy only the `frontend-design` skill to targets.
3. **Install and add to user-level store:**
   - `skillset install anthropics/skills --user` — copy all skills to configured targets **and** into **`~/.skillset/skills`** so they are in the user-level store and can be synced later with `skillset sync --user`.

### 6.5 Add a new skill (scaffold)

1. **Workspace:** `skillset add my-skill` — create `./.skillset/skills/my-skill/` with template `SKILL.md` (and optionally `README.md`). User or AI edits the file. Then `skillset sync` to load to tools.
2. **User-level:** `skillset add my-skill --user` — create `~/.skillset/skills/my-skill/` with template. Then `skillset sync --user` to load to tools.
3. **With AGENTS.md:** If the project (or user) has added the skillset snippet to AGENTS.md, the AI agent knows to store generated skills under `.skillset/skills/<name>/` and to use `skillset add <name>` to scaffold and `skillset sync` after adding or updating skills.

### 6.6 Remove a skill

1. **Remove from targets only:** `skillset remove documan` — delete the `documan` skill directory from every configured target (e.g. `~/.cursor/skills/documan`, `~/.claude/skills/documan`). Source (cwd or user store) is unchanged; skill can be re-synced later.
2. **Remove from targets and user-level store:** `skillset remove documan --user` — remove from all targets **and** from **`~/.skillset/skills/documan`**. Prompt for confirmation (or use `--yes` to skip).

### 6.7 Change where source/targets are

1. User edits config (e.g. `~/.config/skillset/config.json`) or runs something like `skillset config set source /path` / `skillset config set target cursor ~/.cursor/skills` if we add that.
2. Next `skillset list`, `skillset sync`, `skillset install`, and `skillset remove` use the new paths.

---

## 7. Technical approach

### 7.1 Stack

| Layer | Choice | Notes |
|-------|--------|--------|
| Language | Rust | Single binary, no runtime; good for learning and for file I/O. |
| CLI | clap | Subcommands (`list`, `sync`, `install`, `add`, `remove`, maybe `config`, `doc` later). |
| Config | serde + serde_json | One config file (source + targets). |
| Paths | directories | OS-appropriate config dir (e.g. `~/.config/skillset`). |
| Errors | anyhow | Simple `?` and clear messages. |
| TUI | — | Not in MVP. Can add ratatui + crossterm later if desired. |
| Async/HTTP | — | Only if we add LLM generation later (e.g. tokio + reqwest). |

### 7.2 Project layout (target)

```
skillset/
├── Cargo.toml
├── config.example.json
├── src/
│   ├── main.rs           # CLI entry, dispatch to commands
│   ├── config.rs         # Load/save config (source + targets)
│   ├── skills.rs         # List skills from source, copy to targets, overwrite/skip
│   └── (later: tpl.rs, commands/new.rs)
└── README.md
```

No `ui/` folder for MVP; all interaction via CLI and prompts.

### 7.3 Config schema (example)

```json
{
  "source": ".skillset/skills",
  "targets": [
    { "label": "Cursor", "path": "~/.cursor/skills" },
    { "label": "Claude", "path": "~/.claude/skills" }
  ]
}
```

- **Source of truth:** Skills live under **`.skillset/`**. The effective source depends on **scope**: without `--user`, use **workspace (cwd)** e.g. `./.skillset/skills`; with `--user`, use **user-level** `~/.skillset/skills`. Config’s `source` can override if set. Subdirs containing `SKILL.md` are skills.
- `targets`: list of `{ label, path }`; `~` expanded when reading.

### 7.4 Skill format

- **Discovery:** A skill is a directory under the source that contains `SKILL.md`. Other files (e.g. `README.md`) are copied as-is when syncing.
- **Compatibility:** Copy as-is so synced skills work in Cursor (and other tools that accept the same structure). No transformation in MVP.
- **Install package resolution:** For `skillset install <vendor/package>` (e.g. `anthropics/skills`), resolve the package to a skills directory (e.g. GitHub `owner/repo` → fetch or use cached clone; look for `/.cursor/skills` or `/skills` in that repo). Fail clearly if the package cannot be resolved or has no skills. Optional `--skill=NAME` limits installation to that skill. The example `anthropics/skills` is a real package (Anthropic’s skills repo) that we use as well.

### 7.5 AGENTS.md snippet (AI instruction)

Suggested text to add or append to **AGENTS.md** so AI coding agents know where to store generated skills and how to use the skillset CLI. Skillset may provide this via `skillset doc agents-md` or in docs.

```markdown
## Skills (skillset)

- **Where to store generated skills:** Put new skills under **`.skillset/skills/<name>/`** (workspace) or **`~/.skillset/skills/<name>/`** (user-level). Each skill is a directory containing at least **`SKILL.md`**.
- **Scaffold a new skill:** Run **`skillset add <name>`** to create `.skillset/skills/<name>/` with a template `SKILL.md` (use **`--user`** for user-level). Then edit `SKILL.md` with the skill content.
- **After adding or updating skills:** Run **`skillset sync`** (workspace) or **`skillset sync --user`** (user-level) to load skills to the configured tools (e.g. Cursor, Claude).
```

### 7.6 Build and distribution

- `cargo build --release` → single binary.
- Supported platforms: macOS and Linux only. Windows is not supported.
- Optional: CI to build for macOS and Linux; attach to releases.
- No runtime beyond the binary; config created on first run or by user.

---

## 8. Success criteria

| Criterion | How to check |
|-----------|----------------|
| **One source, many targets** | Config has one source and multiple targets; `skillset sync` and `skillset install` copy skills to all (or selected) targets with overwrite/skip. |
| **Local vs user scope** | Without `--user`, source is **cwd** (e.g. `./.skillset/skills`). With `--user`, source is **`~/.skillset/skills`**. `skillset sync --user` syncs from user-level; `skillset install ... --user` also writes into `~/.skillset/skills`. |
| **Install from vendor/package** | `skillset install anthropics/skills` installs all skills from that package; `skillset install anthropics/skills --skill=frontend-design` installs only that skill. |
| **Visibility** | `skillset list` shows skills from source and per-target status (present / missing). Respects `--user` for source. |
| **Remove** | `skillset remove <name>` removes the skill from all configured targets; `skillset remove <name> --user` also removes from `~/.skillset/skills`. Confirmation prompt or `--yes`. |
| **Manage + load** | User can manage skills in the source (edit in their editor) and load them onto their tools with one command. |
| **Add + AGENTS.md** | `skillset add <name>` scaffolds `.skillset/skills/<name>/` with template SKILL.md; the AGENTS.md snippet tells the AI where to store skills and to use `skillset add` and `skillset sync`. Optional `skillset doc agents-md` outputs the snippet. |
| **Portable** | Single binary runs on macOS and Linux without extra install steps. Windows is out of scope. |

---

## 9. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Tool paths change | Config is user-editable; document how to find each tool’s skills dir. |
| Different tools expect different formats | MVP copies as-is; document that format is Cursor-style; adapters only if needed later. |
| Duplication of “Skills CLI” | Position as minimal “manage + load” for your own workflow (and Rust learning); optional TUI/generation later. |

---

## 10. Open questions

- **Binary name:** skillset (decided).
- **Windows:** Not supported; macOS and Linux only.
- **Build vs use Skills CLI:** If Skills CLI already does what you need, you could use it and skip building. This PRD is for “I want to manage and load from `.skillset/` with a minimal tool (and maybe learn Rust).”

---

## 11. References

- This repo’s `.cursor/skills/` and install-skills / update-skills behavior (overwrite/skip) can inform sync behavior. Source of truth is `.skillset/` (e.g. `.skillset/skills`).
- **anthropics/skills** — Real package used as the install example in this PRD; Anthropic’s skills repo.
- Existing option: [Skills CLI](https://dhruvwill.github.io/skills-cli/) (Bun, multi-source/target sync; no TUI, no “create skill”).
