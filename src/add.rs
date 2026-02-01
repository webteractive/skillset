use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Scaffold a new skill with template SKILL.md.
/// source_dir is the resolved source path (e.g. .skillset/skills or ~/.skillset/skills).
pub fn add_skill(name: &str, source_dir: &Path, user_scope: bool, force: bool) -> Result<()> {
    // Validate skill name
    if name.is_empty() {
        anyhow::bail!("Skill name cannot be empty");
    }

    // Validate name format (alphanumeric, hyphens, underscores)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "Skill name must contain only alphanumeric characters, hyphens, or underscores"
        );
    }

    let dest_dir = source_dir.join(name);

    // Check if skill already exists
    if dest_dir.exists() {
        if !force {
            anyhow::bail!(
                "Skill '{}' already exists at {}. Use --force to overwrite.",
                name,
                dest_dir.display()
            );
        }
        fs::remove_dir_all(&dest_dir).context("Failed to remove existing skill directory")?;
    }

    // Create skill directory
    fs::create_dir_all(&dest_dir).context("Failed to create skill directory")?;

    // Write SKILL.md template
    let skill_md = dest_dir.join("SKILL.md");
    let skill_content = generate_skill_template(name);
    fs::write(&skill_md, skill_content).context("Failed to write SKILL.md")?;

    // Write optional README.md
    let readme_md = dest_dir.join("README.md");
    let readme_content = generate_readme_template(name);
    fs::write(&readme_md, readme_content).context("Failed to write README.md")?;

    println!("Skill '{}' created at: {}", name, dest_dir.display());
    println!("Edit {} to add your skill content.", skill_md.display());
    println!(
        "Run 'skillset sync{}' to load to configured tools.",
        if user_scope { " --user" } else { "" }
    );

    Ok(())
}

fn generate_skill_template(name: &str) -> String {
    format!(
        r#"---
name: {}
description: A brief description of what this skill does.
---

# {}

## When to use

Apply this skill when:

- The user asks for help with [specific task/area]
- You need to [perform specific action]
- The context involves [domain or framework]

## Instructions

1. First step or condition
2. Second step
3. Continue as needed

## Paths

| Role   | Path                          |
|--------|-------------------------------|
| Source | [relevant source path if any] |
| Target | [relevant target path if any]  |

## Workflow

### 1. Setup

- Initial setup steps here
- Check for required resources

### 2. Execution

- Step-by-step process
- Handle edge cases

### 3. Verification

- How to verify the result
- Common issues and solutions

## Edge cases

- **Case 1:** Description and solution
- **Case 2:** Description and solution
- **Case 3:** Description and solution
"#,
        name,
        name.replace("-", " ").replace("_", " ").to_uppercase()
    )
}

fn generate_readme_template(name: &str) -> String {
    format!(
        "# {} Skill\n\nA skill for {}\n\n## Usage\n\nRun `skillset sync` to load this skill to your configured tools.\n",
        name,
        name.replace("-", " ").replace("_", " ").to_lowercase()
    )
}
