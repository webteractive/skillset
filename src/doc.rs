/// Return the AGENTS.md snippet for AI coding agents.
/// This snippet instructs agents on where to store generated skills and how to use the skillset CLI.
pub fn agents_md_snippet() -> &'static str {
    r#"## Skills (skillset)

- **Where to store generated skills:** Put new skills under **`.skillset/skills/<name>/`** (workspace) or **`~/.skillset/skills/<name>/`** (user-level). Each skill is a directory containing at least **`SKILL.md`**.
- **Scaffold a new skill:** Run **`skillset add <name>`** to create `.skillset/skills/<name>/` with a template `SKILL.md` (use **`--user`** for user-level). Then edit `SKILL.md` with the skill content.
- **After adding or updating skills:** Run **`skillset sync`** (workspace) or **`skillset sync --user`** (user-level) to load skills to the configured tools (e.g. Cursor, Claude)."#
}
