# use-skillset Skill

Teaches AI agents how to use the [Skillset](https://github.com/webteractive/skillset) CLI: install skills from GitHub, sync to Cursor/Claude/Codex/etc., and manage skills in `.skillset/skills` or `~/.skillset/skills`.

## Usage

Install this skill (and others from this repo) with Skillset:

```bash
skillset install webteractive/skillset --user --sync
```

Or install only this skill:

```bash
skillset install webteractive/skillset --skill=use-skillset --user --sync
```

Then run `skillset sync` (or `skillset sync --user`) after editing skills to load them to your configured tools.
