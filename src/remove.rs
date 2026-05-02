use crate::registry;
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Remove a skill from all configured targets.
/// Only removes target instances that match the active source skill, then removes it from source.
pub fn remove_skill(
    name: &str,
    targets: &[(String, PathBuf)],
    source_dir: &Path,
    yes: bool,
) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Skill name cannot be empty");
    }

    let source_skill_path = source_dir.join(name);
    if !source_skill_path.exists() {
        println!(
            "Skill '{}' not found in source: {}",
            name,
            source_dir.display()
        );
        println!("Nothing removed.");
        return Ok(());
    }

    // Collect targets where the skill exists and appears to be managed by this source.
    let mut targets_with_skill = Vec::new();
    let mut skipped_targets = Vec::new();
    for (label, target_path) in targets {
        let skill_path = target_path.join(name);
        if !skill_path.exists() {
            continue;
        }

        if target_matches_source(&source_skill_path, &skill_path) {
            targets_with_skill.push((label.clone(), skill_path));
        } else {
            skipped_targets.push((label.clone(), skill_path));
        }
    }

    if targets_with_skill.is_empty() {
        println!(
            "Skill '{}' was found in source but not in any matching configured target.",
            name
        );
        for (label, skill_path) in &skipped_targets {
            println!(
                "  Skipped {} at {} (does not match source)",
                label,
                skill_path.display()
            );
        }
        remove_from_source(name, &source_skill_path)?;
        cleanup_registry(name);
        println!("Remove complete.");
        return Ok(());
    }

    // Prompt for confirmation unless --yes is set
    if !yes {
        let target_labels: Vec<&str> = targets_with_skill
            .iter()
            .map(|(label, _)| label.as_str())
            .collect();
        print!(
            "Remove '{}' from {}? [y/n] ",
            name,
            target_labels.join(", ")
        );
        std::io::stdout().flush().context("Flush stdout")?;
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
        remove_path(skill_path)
            .with_context(|| format!("Failed to remove skill '{}' from {}", name, label))?;
        if let Err(e) = registry::remove_path(name, &skill_path.to_string_lossy()) {
            eprintln!("Warning: failed to update registry for '{}': {}", name, e);
        }
        println!("  Removed {} from {}", name, label);
    }

    for (label, skill_path) in &skipped_targets {
        println!(
            "  Skipped {} at {} (does not match source)",
            label,
            skill_path.display()
        );
    }

    remove_from_source(name, &source_skill_path)?;
    cleanup_registry(name);

    println!("Remove complete.");
    Ok(())
}

fn remove_from_source(name: &str, source_skill_path: &Path) -> Result<()> {
    if source_skill_path.exists() {
        remove_path(source_skill_path).context("Failed to remove skill from source")?;
        println!("  Removed {} from source", name);
    }
    Ok(())
}

fn cleanup_registry(name: &str) {
    if let Err(e) = registry::remove_skill(name) {
        eprintln!("Warning: failed to clean up registry for '{}': {}", name, e);
    }
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("Failed to inspect path")?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).context("Failed to remove file or symlink")?;
    } else {
        fs::remove_dir_all(path).context("Failed to remove directory")?;
    }
    Ok(())
}

fn target_matches_source(source: &Path, target: &Path) -> bool {
    if target_symlink_matches_source(source, target) {
        return true;
    }

    skill_md_matches_source(source, target)
}

fn target_symlink_matches_source(source: &Path, target: &Path) -> bool {
    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    if !metadata.file_type().is_symlink() {
        return false;
    }

    let link_target = match fs::read_link(target) {
        Ok(path) => path,
        Err(_) => return false,
    };
    let resolved_target = if link_target.is_absolute() {
        link_target
    } else {
        target
            .parent()
            .map(|parent| parent.join(&link_target))
            .unwrap_or(link_target)
    };

    match (source.canonicalize(), resolved_target.canonicalize()) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

fn skill_md_matches_source(source: &Path, target: &Path) -> bool {
    let source_md = source.join("SKILL.md");
    let target_md = target.join("SKILL.md");

    if !source_md.exists() || !target_md.exists() {
        return false;
    }

    match (fs::read(source_md), fs::read(target_md)) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "skillset_remove_test_{}_{}",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn target_matches_source_for_symlink_to_source() {
        let tmp = unique_tmp("symlink");
        let _ = fs::remove_dir_all(&tmp);
        let source = tmp.join("source").join("my-skill");
        let target = tmp.join("target").join("my-skill");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(source.join("SKILL.md"), "# My Skill").unwrap();
        symlink_dir(&source, &target).unwrap();

        assert!(target_matches_source(&source, &target));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn target_matches_source_for_identical_copy() {
        let tmp = unique_tmp("copy");
        let _ = fs::remove_dir_all(&tmp);
        let source = tmp.join("source").join("my-skill");
        let target = tmp.join("target").join("my-skill");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("SKILL.md"), "# My Skill").unwrap();
        fs::write(target.join("SKILL.md"), "# My Skill").unwrap();

        assert!(target_matches_source(&source, &target));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn target_does_not_match_different_same_named_skill() {
        let tmp = unique_tmp("different");
        let _ = fs::remove_dir_all(&tmp);
        let source = tmp.join("source").join("my-skill");
        let target = tmp.join("target").join("my-skill");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("SKILL.md"), "# My Skill").unwrap();
        fs::write(target.join("SKILL.md"), "# Different Skill").unwrap();

        assert!(!target_matches_source(&source, &target));

        fs::remove_dir_all(&tmp).ok();
    }

    #[cfg(unix)]
    fn symlink_dir(from: &Path, to: &Path) -> Result<()> {
        std::os::unix::fs::symlink(from, to).context("Failed to create symlink")
    }

    #[cfg(windows)]
    fn symlink_dir(from: &Path, to: &Path) -> Result<()> {
        std::os::windows::fs::symlink_dir(from, to).context("Failed to create symlink")
    }
}
