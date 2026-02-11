use crate::skills::{copy_skill, discover_skills};
use anyhow::{Context, Result};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Check if spec looks like a full Git URL (HTTPS, HTTP, SSH, or git@)
fn is_git_url(spec: &str) -> bool {
    spec.starts_with("https://")
        || spec.starts_with("http://")
        || spec.starts_with("git@")
        || spec.starts_with("ssh://")
}

/// Generate a stable cache directory name from a URL hash
fn cache_dir_name_from_url(url: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    format!("url-{:016x}", hasher.finish())
}

/// Resolve a vendor/package spec to a local path via git clone.
///
/// Format:
///   - owner/repo (e.g., anthropics/skills) - uses GitHub
///   - Full Git URL (e.g., git@github.com:anthropics/skills.git, https://github.com/anthropics/skills.git)
///
/// The use_ssh flag determines whether owner/repo format uses SSH or HTTPS URLs.
pub fn resolve_package(spec: &str, use_ssh: bool, from_remote: bool) -> Result<PathBuf> {
    // Determine cache directory
    let cache_dir = dirs::cache_dir()
        .context("Failed to determine cache directory")?
        .join("skillset")
        .join("repos");

    std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

    let (repo_url, repo_dir_name) = if is_git_url(spec) {
        // Use the URL as-is and derive cache dir name from hash
        (spec.to_string(), cache_dir_name_from_url(spec))
    } else {
        // Parse owner/repo
        let parts: Vec<&str> = spec.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid package spec '{}'. Expected format: owner/repo or full Git URL",
                spec
            );
        }

        let owner = parts[0];
        let repo = parts[1];

        let url = if use_ssh {
            format!("git@github.com:{}:{}.git", owner, repo)
        } else {
            format!("https://github.com/{}/{}.git", owner, repo)
        };

        (url, format!("{}-{}", owner, repo))
    };

    let repo_dir = cache_dir.join(&repo_dir_name);

    if repo_dir.exists() {
        if from_remote {
            println!("Pulling latest from remote for {}...", spec);
            let status = Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .arg("pull")
                .status()
                .context("Failed to run git pull. Is git installed?")?;

            if !status.success() {
                anyhow::bail!("Failed to pull latest for: {}", spec);
            }
            println!("Updated cached package at: {}", repo_dir.display());
        } else {
            println!("Package already cached at: {}", repo_dir.display());
        }
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
/// Checks each dir in skill_dirs in order; uses first that exists and contains skills.
pub fn find_skills_dir(repo_root: &Path, skill_dirs: &[String]) -> Result<PathBuf> {
    for dir in skill_dirs {
        let path = repo_root.join(dir);
        if path.exists() {
            if discover_skills(&path)?.is_empty() {
                anyhow::bail!("{} exists but contains no skills", dir);
            }
            return Ok(path);
        }
    }

    let checked = skill_dirs.join(", ");
    anyhow::bail!(
        "No skills directory found in repository (checked {})",
        checked
    );
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
    use_ssh: bool,
    skill_dirs: &[String],
    from_remote: bool,
) -> Result<()> {
    // Resolve package
    let repo_dir = resolve_package(spec, use_ssh, from_remote)?;
    let skills_dir = find_skills_dir(&repo_dir, skill_dirs)?;
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
