use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Discover skills in a source directory.
/// A skill is a subdirectory that contains a SKILL.md file.
pub fn discover_skills(source_dir: &Path) -> Result<Vec<String>> {
    if !source_dir.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();

    let entries = fs::read_dir(source_dir).context("Failed to read source directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        skills.push(name_str.to_string());
                    }
                }
            }
        }
    }

    skills.sort();
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_skills_finds_subdir_with_skill_md() {
        let tmp = std::env::temp_dir().join("skillset_test_discover");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();
        let skills = discover_skills(&tmp).unwrap();
        fs::remove_dir_all(&tmp).ok();
        assert_eq!(skills, vec!["my-skill"]);
    }

    #[test]
    fn test_discover_skills_ignores_subdir_without_skill_md() {
        let tmp = std::env::temp_dir().join("skillset_test_discover_no_md");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("not-a-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        // no SKILL.md
        let skills = discover_skills(&tmp).unwrap();
        fs::remove_dir_all(&tmp).ok();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_discover_skills_empty_dir_returns_empty() {
        let tmp = std::env::temp_dir().join("skillset_test_discover_empty");
        fs::create_dir_all(&tmp).unwrap();
        let skills = discover_skills(&tmp).unwrap();
        fs::remove_dir_all(&tmp).ok();
        assert!(skills.is_empty());
    }
}

/// Copy an entire skill directory from source to target.
/// Creates parent directories if needed and overwrites existing files.
pub fn copy_skill(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        anyhow::bail!("Source skill directory does not exist: {}", from.display());
    }

    if to.exists() {
        fs::remove_dir_all(to).context("Failed to remove existing target directory")?;
    }

    fs::create_dir_all(to.parent().unwrap()).context("Failed to create parent directory")?;

    copy_dir_recursive(from, to).context("Failed to copy skill directory")?;

    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        fs::copy(from, to).context("Failed to copy file")?;
        return Ok(());
    }

    fs::create_dir_all(to).context("Failed to create directory")?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dest = to.join(entry.file_name());
        copy_dir_recursive(&src, &dest)?;
    }

    Ok(())
}

/// Overwrite policy for syncing skills.
#[derive(Debug, Clone, Copy)]
pub enum OverwritePolicy {
    PerSkill,
    All,
}

/// Sync skills from source to multiple targets.
/// Creates each target dir if it doesn't exist, then copies skills. Prompts when a skill already exists.
pub fn sync_skills(
    source: &Path,
    targets: &[(String, PathBuf)],
    user_policy: &mut OverwritePolicy,
) -> Result<()> {
    let skills = discover_skills(source)?;

    if skills.is_empty() {
        println!("No skills found in source: {}", source.display());
        return Ok(());
    }

    // Ensure each target base dir exists (e.g. ~/.claude/skills, ~/.cursor/skills)
    for (_, target_path) in targets {
        fs::create_dir_all(target_path).context("Failed to create target directory")?;
    }

    println!("Found {} skill(s) to sync:", skills.len());

    for skill_name in &skills {
        let skill_source = source.join(skill_name);

        for (label, target_path) in targets {
            let skill_target = target_path.join(skill_name);
            let exists = skill_target.exists();

            if exists {
                match *user_policy {
                    OverwritePolicy::All => {
                        copy_skill(&skill_source, &skill_target)?;
                        println!("  Overwrote {} at {}", skill_name, label);
                    }
                    OverwritePolicy::PerSkill => {
                        print!(
                            "  Skill '{}' already exists at {}. Overwrite? [y/n/all] ",
                            skill_name, label
                        );
                        std::io::stdout().flush().context("Flush stdout")?;
                        let mut input = String::new();
                        std::io::stdin()
                            .read_line(&mut input)
                            .context("Failed to read user input")?;
                        let input = input.trim().to_lowercase();

                        match input.as_str() {
                            "y" | "yes" => {
                                copy_skill(&skill_source, &skill_target)?;
                                println!("    Copied to {}", label);
                            }
                            "a" | "all" => {
                                *user_policy = OverwritePolicy::All;
                                copy_skill(&skill_source, &skill_target)?;
                                println!("    Copied to {} (will overwrite rest)", label);
                            }
                            _ => {
                                println!("    Skipped {}", label);
                            }
                        }
                    }
                }
            } else {
                copy_skill(&skill_source, &skill_target)?;
                println!("  Copied {} to {}", skill_name, label);
            }
        }
    }

    println!("Sync complete.");
    Ok(())
}
