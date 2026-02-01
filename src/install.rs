use crate::skills::{copy_skill, discover_skills};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Resolve a vendor/package spec to a local path via git clone.
/// Format: owner/repo (e.g., anthropics/skills)
pub fn resolve_package(spec: &str) -> Result<PathBuf> {
    // Parse owner/repo
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid package spec '{}'. Expected format: owner/repo",
            spec
        );
    }

    let owner = parts[0];
    let repo = parts[1];

    // Determine cache directory
    let cache_dir = dirs::cache_dir()
        .context("Failed to determine cache directory")?
        .join("skillset")
        .join("repos");

    std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

    let repo_dir = cache_dir.join(format!("{}-{}", owner, repo));
    let repo_url = format!("https://github.com/{}/{}.git", owner, repo);

    if repo_dir.exists() {
        println!("Package already cached at: {}", repo_dir.display());
        // Optionally git pull in the future; for MVP, use as-is
    } else {
        println!("Cloning {} from {}...", spec, repo_url);

        let status = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(&repo_url)
            .arg(&repo_dir)
            .status()
            .context("Failed to run git clone. Is git installed?")?;

        if !status.success() {
            anyhow::bail!("Failed to clone repository: {}", spec);
        }

        println!("Cloned to: {}", repo_dir.display());
    }

    Ok(repo_dir)
}

/// Find the skills directory within a repository root.
/// Checks for .cursor/skills first, then skills.
pub fn find_skills_dir(repo_root: &Path) -> Result<PathBuf> {
    let cursor_skills = repo_root.join(".cursor/skills");
    let skills = repo_root.join("skills");

    if cursor_skills.exists() {
        if discover_skills(&cursor_skills)?.is_empty() {
            anyhow::bail!(".cursor/skills exists but contains no skills");
        }
        return Ok(cursor_skills);
    }

    if skills.exists() {
        if discover_skills(&skills)?.is_empty() {
            anyhow::bail!("skills exists but contains no skills");
        }
        return Ok(skills);
    }

    anyhow::bail!("No skills directory found in repository (checked .cursor/skills and skills)");
}

/// Install skills from a package into the source of truth only (workspace or user store).
/// Does not copy to AI tool dirs (Cursor, etc.); use `skillset sync` or `install --sync` for that.
/// - source_dir: if Some, copy to this path (e.g. cwd/.skillset/skills) as workspace source of truth
/// - user_store_dir: if Some, copy to that path (user-level store, e.g. ~/.skillset/skills)
/// - overwrite_all: if true, skip prompts and always overwrite when skill already exists
pub fn install_package(
    spec: &str,
    skill_filter: Option<&str>,
    source_dir: Option<&Path>,
    user_store_dir: Option<&Path>,
    overwrite_all: bool,
) -> Result<()> {
    // Resolve package
    let repo_dir = resolve_package(spec)?;
    let skills_dir = find_skills_dir(&repo_dir)?;
    let all_skills = discover_skills(&skills_dir)?;

    if all_skills.is_empty() {
        anyhow::bail!("No skills found in package");
    }

    // Filter by skill name if specified
    let skills_to_install: Vec<String> = if let Some(filter) = skill_filter {
        if !all_skills.contains(&filter.to_string()) {
            anyhow::bail!(
                "Skill '{}' not found in package. Available skills: {}",
                filter,
                all_skills.join(", ")
            );
        }
        vec![filter.to_string()]
    } else {
        all_skills
    };

    println!(
        "Installing {} skill(s) from {}:",
        skills_to_install.len(),
        spec
    );
    for skill in &skills_to_install {
        println!("  - {}", skill);
    }

    // Install to workspace source first (e.g. cwd/.skillset/skills when no --user), so .skillset is populated before targets
    if let Some(workspace_source) = source_dir {
        std::fs::create_dir_all(workspace_source)
            .context("Failed to create workspace source directory")?;

        for skill_name in &skills_to_install {
            let skill_source = skills_dir.join(skill_name);
            let skill_target = workspace_source.join(skill_name);

            if skill_target.exists() {
                if overwrite_all {
                    copy_skill(&skill_source, &skill_target)?;
                    println!("  Overwrote {} in workspace source", skill_name);
                } else {
                    print!(
                        "  Skill '{}' already exists in {}. Overwrite? [y/n] ",
                        skill_name,
                        workspace_source.display()
                    );
                    std::io::stdout().flush().context("Flush stdout")?;
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .context("Failed to read user input")?;
                    let input = input.trim().to_lowercase();

                    if matches!(input.as_str(), "y" | "yes") {
                        copy_skill(&skill_source, &skill_target)?;
                        println!("    Copied to workspace source");
                    } else {
                        println!("    Skipped workspace source");
                    }
                }
            } else {
                copy_skill(&skill_source, &skill_target)?;
                println!("  Copied {} to {}", skill_name, workspace_source.display());
            }
        }
    }

    // Install to user store if path provided
    if let Some(user_skills_dir) = user_store_dir {
        std::fs::create_dir_all(user_skills_dir)
            .context("Failed to create user skills directory")?;

        for skill_name in &skills_to_install {
            let skill_source = skills_dir.join(skill_name);
            let skill_target = user_skills_dir.join(skill_name);

            if skill_target.exists() {
                if overwrite_all {
                    copy_skill(&skill_source, &skill_target)?;
                    println!("  Overwrote {} in user store", skill_name);
                } else {
                    print!(
                        "  Skill '{}' already exists in user store. Overwrite? [y/n] ",
                        skill_name
                    );
                    std::io::stdout().flush().context("Flush stdout")?;
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .context("Failed to read user input")?;
                    let input = input.trim().to_lowercase();

                    if matches!(input.as_str(), "y" | "yes") {
                        copy_skill(&skill_source, &skill_target)?;
                        println!("    Copied to user store");
                    } else {
                        println!("    Skipped user store");
                    }
                }
            } else {
                copy_skill(&skill_source, &skill_target)?;
                println!("  Copied {} to user store", skill_name);
            }
        }
    }

    println!("Install complete.");
    Ok(())
}
