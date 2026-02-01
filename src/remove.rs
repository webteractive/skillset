use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Remove a skill from all configured targets.
/// If user_scope is true, also remove from ~/.ai/skills.
pub fn remove_skill(name: &str, targets: &[(String, PathBuf)], user_scope: bool, yes: bool) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Skill name cannot be empty");
    }

    // Collect targets where the skill exists
    let mut targets_with_skill = Vec::new();
    for (label, target_path) in targets {
        let skill_path = target_path.join(name);
        if skill_path.exists() {
            targets_with_skill.push((label.clone(), skill_path));
        }
    }

    if targets_with_skill.is_empty() {
        println!("Skill '{}' not found in any configured target.", name);
        return Ok(());
    }

    // Prompt for confirmation unless --yes is set
    if !yes {
        let target_labels: Vec<&str> = targets_with_skill.iter().map(|(label, _)| label.as_str()).collect();
        print!(
            "Remove '{}' from {}? [y/n] ",
            name,
            target_labels.join(", ")
        );
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;
        let input = input.trim().to_lowercase();

        if !matches!(input.as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Remove from targets
    for (label, skill_path) in &targets_with_skill {
        fs::remove_dir_all(skill_path).with_context(|| {
            format!("Failed to remove skill '{}' from {}", name, label)
        })?;
        println!("  Removed {} from {}", name, label);
    }

    // Remove from user store if --user is set
    if user_scope {
        if let Ok(home) = std::env::var("HOME") {
            let user_skill_path = PathBuf::from(home).join(".ai/skills").join(name);
            if user_skill_path.exists() {
                fs::remove_dir_all(&user_skill_path)
                    .context("Failed to remove skill from user store")?;
                println!("  Removed {} from user store", name);
            }
        } else {
            anyhow::bail!("HOME environment variable not set");
        }
    }

    println!("Remove complete.");
    Ok(())
}
