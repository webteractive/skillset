use crate::registry;
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
                    match name.to_str() {
                        Some(name_str) => skills.push(name_str.to_string()),
                        None => eprintln!(
                            "Warning: skipping skill with non-UTF-8 directory name: {:?}",
                            name
                        ),
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

    fn unique_tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("skillset_test_{}_{}", name, std::process::id()))
    }

    #[test]
    fn test_discover_skills_finds_subdir_with_skill_md() {
        let tmp = unique_tmp("discover");
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
        let tmp = unique_tmp("discover_no_md");
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
        let tmp = unique_tmp("discover_empty");
        fs::create_dir_all(&tmp).unwrap();
        let skills = discover_skills(&tmp).unwrap();
        fs::remove_dir_all(&tmp).ok();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_symlink_skill_creates_directory_symlink() {
        let tmp = unique_tmp("symlink_skill");
        let _ = fs::remove_dir_all(&tmp);
        let source = tmp.join("source").join("my-skill");
        let target = tmp.join("target").join("my-skill");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "# My Skill").unwrap();

        symlink_skill(&source, &target).unwrap();

        assert!(target.join("SKILL.md").exists());
        assert!(fs::symlink_metadata(&target)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(skill_target_unchanged(
            &source,
            &target,
            SyncMethod::Symlink
        ));
        assert!(!skill_target_unchanged(&source, &target, SyncMethod::Copy));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_copy_skill_replaces_directory_symlink_with_copy() {
        let tmp = unique_tmp("copy_replaces_symlink");
        let _ = fs::remove_dir_all(&tmp);
        let source = tmp.join("source").join("my-skill");
        let target = tmp.join("target").join("my-skill");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "# My Skill").unwrap();
        symlink_skill(&source, &target).unwrap();

        copy_skill(&source, &target).unwrap();

        assert!(target.join("SKILL.md").exists());
        assert!(!fs::symlink_metadata(&target)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(skill_target_unchanged(&source, &target, SyncMethod::Copy));

        fs::remove_dir_all(&tmp).ok();
    }
}

/// Copy an entire skill directory from source to target.
/// Creates parent directories if needed and overwrites existing files.
pub fn copy_skill(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        anyhow::bail!("Source skill directory does not exist: {}", from.display());
    }

    remove_existing_path(to).context("Failed to remove existing target directory")?;

    fs::create_dir_all(to.parent().unwrap()).context("Failed to create parent directory")?;

    copy_dir_recursive(from, to).context("Failed to copy skill directory")?;

    Ok(())
}

/// Symlink an entire skill directory from source to target.
/// Creates parent directories if needed and overwrites existing files.
pub fn symlink_skill(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        anyhow::bail!("Source skill directory does not exist: {}", from.display());
    }

    remove_existing_path(to).context("Failed to remove existing target directory")?;
    fs::create_dir_all(to.parent().unwrap()).context("Failed to create parent directory")?;
    create_dir_symlink(from, to).context("Failed to symlink skill directory")?;

    Ok(())
}

fn remove_existing_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("Failed to inspect existing target"),
    };

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).context("Failed to remove existing file or symlink")?;
    } else {
        fs::remove_dir_all(path).context("Failed to remove existing directory")?;
    }

    Ok(())
}

#[cfg(unix)]
fn create_dir_symlink(from: &Path, to: &Path) -> Result<()> {
    std::os::unix::fs::symlink(from, to).context("Failed to create directory symlink")
}

#[cfg(windows)]
fn create_dir_symlink(from: &Path, to: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(from, to).context("Failed to create directory symlink")
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

/// How skills should be written to target directories.
#[derive(Debug, Clone, Copy)]
pub enum SyncMethod {
    Copy,
    Symlink,
}

impl SyncMethod {
    fn action(self) -> &'static str {
        match self {
            SyncMethod::Copy => "copy",
            SyncMethod::Symlink => "symlink",
        }
    }

    fn past_tense(self) -> &'static str {
        match self {
            SyncMethod::Copy => "Copied",
            SyncMethod::Symlink => "Symlinked",
        }
    }

    fn overwrite_tense(self) -> &'static str {
        match self {
            SyncMethod::Copy => "Overwrote",
            SyncMethod::Symlink => "Re-symlinked",
        }
    }
}

/// Check if a skill's content is identical between source and target by comparing SKILL.md.
fn skill_unchanged(source: &Path, target: &Path) -> bool {
    if fs::symlink_metadata(target)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return false;
    }

    let source_md = source.join("SKILL.md");
    let target_md = target.join("SKILL.md");

    if !source_md.exists() || !target_md.exists() {
        return false;
    }

    match (fs::read(&source_md), fs::read(&target_md)) {
        (Ok(src), Ok(tgt)) => src == tgt,
        _ => false,
    }
}

fn skill_symlink_unchanged(source: &Path, target: &Path) -> bool {
    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    if !metadata.file_type().is_symlink() {
        return false;
    }

    let target_link = match fs::read_link(target) {
        Ok(path) => path,
        Err(_) => return false,
    };

    let resolved_target = if target_link.is_absolute() {
        target_link
    } else {
        target
            .parent()
            .map(|parent| parent.join(&target_link))
            .unwrap_or(target_link)
    };

    match (source.canonicalize(), resolved_target.canonicalize()) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

fn skill_target_unchanged(source: &Path, target: &Path, method: SyncMethod) -> bool {
    match method {
        SyncMethod::Copy => skill_unchanged(source, target),
        SyncMethod::Symlink => skill_symlink_unchanged(source, target),
    }
}

fn write_skill(source: &Path, target: &Path, method: SyncMethod) -> Result<()> {
    match method {
        SyncMethod::Copy => copy_skill(source, target),
        SyncMethod::Symlink => symlink_skill(source, target),
    }
}

/// Show a unified diff of two files using the `similar` crate.
fn show_diff(source_path: &Path, target_path: &Path, skill_name: &str, label: &str) {
    let source_content = match fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not read {}: {}", source_path.display(), e);
            return;
        }
    };
    let target_content = match fs::read_to_string(target_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not read {}: {}", target_path.display(), e);
            return;
        }
    };

    if source_content == target_content {
        println!(
            "    (no changes in SKILL.md for {} at {})",
            skill_name, label
        );
        return;
    }

    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(&target_content, &source_content);

    println!(
        "    --- {}/{}/SKILL.md (target: {})",
        label, skill_name, label
    );
    println!("    +++ {}/{}/SKILL.md (source)", label, skill_name);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("    {}{}", sign, change);
    }
    println!();
}

/// Sync skills from source to multiple targets.
/// Creates each target dir if it doesn't exist, then copies skills. Prompts when a skill already exists.
pub fn sync_skills(
    source: &Path,
    targets: &[(String, PathBuf)],
    user_policy: &mut OverwritePolicy,
    dry_run: bool,
    show_diffs: bool,
    method: SyncMethod,
) -> Result<()> {
    let skills = discover_skills(source)?;

    if skills.is_empty() {
        println!("No skills found in source: {}", source.display());
        return Ok(());
    }

    if !dry_run {
        // Ensure each target base dir exists (e.g. ~/.claude/skills, ~/.cursor/skills)
        for (_, target_path) in targets {
            fs::create_dir_all(target_path).context("Failed to create target directory")?;
        }
    }

    println!("Found {} skill(s) to sync:", skills.len());

    for skill_name in &skills {
        let skill_source = source.join(skill_name);

        for (label, target_path) in targets {
            let skill_target = target_path.join(skill_name);
            let exists = skill_target.exists();

            if dry_run {
                if exists {
                    println!(
                        "[DRY RUN]   Would {} {} at {}",
                        method.action(),
                        skill_name,
                        label
                    );
                    if show_diffs {
                        let source_md = skill_source.join("SKILL.md");
                        let target_md = skill_target.join("SKILL.md");
                        if source_md.exists() && target_md.exists() {
                            show_diff(&source_md, &target_md, skill_name, label);
                        }
                    }
                } else {
                    println!(
                        "[DRY RUN]   Would {} {} to {}",
                        method.action(),
                        skill_name,
                        label
                    );
                }
                continue;
            }

            if exists {
                // Skip if target already matches the selected sync method.
                if skill_target_unchanged(&skill_source, &skill_target, method) {
                    continue;
                }

                if show_diffs {
                    let source_md = skill_source.join("SKILL.md");
                    let target_md = skill_target.join("SKILL.md");
                    if source_md.exists() && target_md.exists() {
                        show_diff(&source_md, &target_md, skill_name, label);
                    }
                }

                match *user_policy {
                    OverwritePolicy::All => {
                        write_skill(&skill_source, &skill_target, method)?;
                        if let Err(e) =
                            registry::record(skill_name, &skill_target.to_string_lossy(), label)
                        {
                            eprintln!(
                                "Warning: failed to update registry for '{}': {}",
                                skill_name, e
                            );
                        }
                        println!("  {} {} at {}", method.overwrite_tense(), skill_name, label);
                    }
                    OverwritePolicy::PerSkill => {
                        print!(
                            "  Skill '{}' already exists at {}. {}? [y/n/all] ",
                            skill_name,
                            label,
                            method.overwrite_tense()
                        );
                        std::io::stdout().flush().context("Flush stdout")?;
                        let mut input = String::new();
                        std::io::stdin()
                            .read_line(&mut input)
                            .context("Failed to read user input")?;
                        let input = input.trim().to_lowercase();

                        match input.as_str() {
                            "y" | "yes" => {
                                write_skill(&skill_source, &skill_target, method)?;
                                if let Err(e) = registry::record(
                                    skill_name,
                                    &skill_target.to_string_lossy(),
                                    label,
                                ) {
                                    eprintln!(
                                        "Warning: failed to update registry for '{}': {}",
                                        skill_name, e
                                    );
                                }
                                println!("    {} to {}", method.past_tense(), label);
                            }
                            "a" | "all" => {
                                *user_policy = OverwritePolicy::All;
                                write_skill(&skill_source, &skill_target, method)?;
                                if let Err(e) = registry::record(
                                    skill_name,
                                    &skill_target.to_string_lossy(),
                                    label,
                                ) {
                                    eprintln!(
                                        "Warning: failed to update registry for '{}': {}",
                                        skill_name, e
                                    );
                                }
                                println!(
                                    "    {} to {} (will overwrite rest)",
                                    method.past_tense(),
                                    label
                                );
                            }
                            _ => {
                                println!("    Skipped {}", label);
                            }
                        }
                    }
                }
            } else {
                write_skill(&skill_source, &skill_target, method)?;
                if let Err(e) = registry::record(skill_name, &skill_target.to_string_lossy(), label)
                {
                    eprintln!(
                        "Warning: failed to update registry for '{}': {}",
                        skill_name, e
                    );
                }
                println!("  {} {} to {}", method.past_tense(), skill_name, label);
            }
        }
    }

    if dry_run {
        println!("[DRY RUN] Sync complete. No changes were made.");
    } else {
        println!("Sync complete.");
    }
    Ok(())
}
