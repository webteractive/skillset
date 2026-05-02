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

/// Expand a leading ~/ in a local path spec.
fn expand_home_path(spec: &str) -> PathBuf {
    if spec == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }

    if let Some(rest) = spec.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(spec)
}

fn is_path_like(spec: &str) -> bool {
    spec == "~"
        || spec.starts_with("~/")
        || spec.starts_with("./")
        || spec.starts_with("../")
        || spec.starts_with('/')
}

/// Generate a stable cache directory name from a URL hash
fn cache_dir_name_from_url(url: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    format!("url-{:016x}", hasher.finish())
}

/// Resolve a vendor/package spec to a local path.
///
/// Format:
///   - Local path to a repo, skills directory, or single skill directory
///   - owner/repo (e.g., anthropics/skills) - uses GitHub
///   - Full Git URL (e.g., git@github.com:anthropics/skills.git, https://github.com/anthropics/skills.git)
///
/// The use_ssh flag determines whether owner/repo format uses SSH or HTTPS URLs.
pub fn resolve_package(spec: &str, use_ssh: bool, from_remote: bool) -> Result<PathBuf> {
    let local_path = expand_home_path(spec);
    if local_path.exists() {
        if !local_path.is_dir() {
            anyhow::bail!(
                "Local package path is not a directory: {}",
                local_path.display()
            );
        }

        if from_remote {
            eprintln!("Warning: --from-remote is ignored for local path installs.");
        }

        return local_path
            .canonicalize()
            .with_context(|| format!("Failed to resolve local path: {}", local_path.display()));
    }

    if is_path_like(spec) {
        anyhow::bail!(
            "Local package path does not exist: {}",
            local_path.display()
        );
    }

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
                "Invalid package spec '{}'. Expected format: local path, owner/repo, or full Git URL",
                spec
            );
        }

        let owner = parts[0];
        let repo = parts[1];

        let ssh_url = format!("git@github.com:{}/{}.git", owner, repo);
        let https_url = format!("https://github.com/{}/{}.git", owner, repo);

        let url = if use_ssh { &ssh_url } else { &https_url };

        (url.to_string(), format!("{}-{}", owner, repo))
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
            // If the primary URL failed and spec is owner/repo, try the other protocol
            if !is_git_url(spec) {
                let fallback_url = if use_ssh {
                    format!("https://github.com/{}.git", spec)
                } else {
                    format!("git@github.com:{}.git", spec)
                };
                let protocol = if use_ssh { "HTTPS" } else { "SSH" };
                println!("Retrying with {}...", protocol);

                let status = Command::new("git")
                    .arg("clone")
                    .arg("--depth")
                    .arg("1")
                    .arg(&fallback_url)
                    .arg(&repo_dir)
                    .status()
                    .context("Failed to run git clone fallback")?;

                if !status.success() {
                    anyhow::bail!("Failed to clone repository: {}", spec);
                }
            } else {
                anyhow::bail!("Failed to clone repository: {}", spec);
            }
        }

        println!("Cloned to: {}", repo_dir.display());
    }

    Ok(repo_dir)
}

/// Find installable skills within a package root.
///
/// Supports:
/// - A direct single skill directory containing SKILL.md
/// - A direct skills directory containing skill subdirectories
/// - A repository root containing one of the configured skill_dirs
pub fn find_installable_skills(
    package_root: &Path,
    skill_dirs: &[String],
) -> Result<(PathBuf, Vec<String>)> {
    if package_root.join("SKILL.md").is_file() {
        let parent = package_root.parent().with_context(|| {
            format!(
                "Could not determine parent directory for skill: {}",
                package_root.display()
            )
        })?;
        let name = package_root
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| {
                format!(
                    "Could not determine skill name from path: {}",
                    package_root.display()
                )
            })?;

        return Ok((parent.to_path_buf(), vec![name.to_string()]));
    }

    let direct_skills = discover_skills(package_root)?;
    if !direct_skills.is_empty() {
        return Ok((package_root.to_path_buf(), direct_skills));
    }

    for dir in skill_dirs {
        let path = package_root.join(dir);
        if path.exists() {
            let skills = discover_skills(&path)?;
            if skills.is_empty() {
                anyhow::bail!("{} exists but contains no skills", dir);
            }
            return Ok((path, skills));
        }
    }

    let checked = skill_dirs.join(", ");
    anyhow::bail!(
        "No skills found in package (checked package root and {})",
        checked
    );
}

/// Install skills from a package into the source of truth only (workspace or user store).
/// Does not copy to AI tool dirs (Cursor, etc.); use `skillset sync` or `install --sync` for that.
/// - source_dir: if Some, copy to this path (e.g. cwd/.skillset/skills) as workspace source of truth
/// - user_store_dir: if Some, copy to that path (user-level store, e.g. ~/.skillset/skills)
/// - overwrite_all: if true, skip prompts and always overwrite when skill already exists
#[allow(clippy::too_many_arguments)]
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
    let package_dir = resolve_package(spec, use_ssh, from_remote)?;
    let (skills_dir, all_skills) = find_installable_skills(&package_dir, skill_dirs)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("skillset_install_test_{}_{}", name, nonce))
    }

    #[test]
    fn local_package_path_resolves_without_git() {
        let tmp = temp_dir("resolve_local");
        fs::create_dir_all(&tmp).unwrap();

        let resolved = resolve_package(tmp.to_str().unwrap(), true, false).unwrap();

        fs::remove_dir_all(&tmp).ok();
        assert!(resolved.is_absolute());
    }

    #[test]
    fn missing_path_like_package_errors_as_local_path() {
        let missing = temp_dir("missing_path").join("missing");
        let error = resolve_package(missing.to_str().unwrap(), true, false).unwrap_err();

        assert!(error
            .to_string()
            .contains("Local package path does not exist"));
    }

    #[test]
    fn local_single_skill_dir_is_installable() {
        let tmp = temp_dir("single_skill");
        let skill = tmp.join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# My Skill").unwrap();

        let (skills_dir, skills) = find_installable_skills(&skill, &[]).unwrap();

        fs::remove_dir_all(&tmp).ok();
        assert_eq!(skills_dir.file_name().unwrap(), tmp.file_name().unwrap());
        assert_eq!(skills, vec!["my-skill"]);
    }

    #[test]
    fn local_skills_dir_is_installable() {
        let tmp = temp_dir("skills_dir");
        let skill = tmp.join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# My Skill").unwrap();

        let (skills_dir, skills) = find_installable_skills(&tmp, &[]).unwrap();

        fs::remove_dir_all(&tmp).ok();
        assert_eq!(skills_dir.file_name().unwrap(), tmp.file_name().unwrap());
        assert_eq!(skills, vec!["my-skill"]);
    }

    #[test]
    fn local_repo_root_uses_configured_skill_dirs() {
        let tmp = temp_dir("repo_root");
        let skill = tmp.join("skills").join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# My Skill").unwrap();

        let (skills_dir, skills) =
            find_installable_skills(&tmp, &[".claude/skills".to_string(), "skills".to_string()])
                .unwrap();

        fs::remove_dir_all(&tmp).ok();
        assert_eq!(skills_dir.file_name().unwrap(), "skills");
        assert_eq!(skills, vec!["my-skill"]);
    }
}
