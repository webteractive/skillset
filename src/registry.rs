use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInstance {
    pub path: String,
    pub label: String,
    pub synced_at: String,
}

/// Registry maps skill_name -> Vec<SkillInstance>
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Registry {
    pub skills: BTreeMap<String, Vec<SkillInstance>>,
}

fn registry_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "skillset")
        .context("Failed to determine config directory")?;
    Ok(dirs.config_dir().join("registry.json"))
}

fn now_timestamp() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO-ish format from unix timestamp
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let months = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    format!("{:04}-{:02}-{:02}", years, months, day)
}

pub fn load() -> Result<Registry> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(Registry::default());
    }

    let content = fs::read_to_string(&path).context("Failed to read registry")?;
    let mut registry: Registry = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Warning: registry.json is malformed ({}). Starting with empty registry.",
                e
            );
            eprintln!("  The original file is at: {}", path.display());
            return Ok(Registry::default());
        }
    };

    // Clean stale entries (paths that no longer exist)
    let mut changed = false;
    for instances in registry.skills.values_mut() {
        let before = instances.len();
        instances.retain(|inst| {
            let exists = PathBuf::from(&inst.path).exists();
            if !exists {
                changed = true;
            }
            exists
        });
        if instances.len() != before {
            changed = true;
        }
    }
    // Remove skills with no remaining instances
    registry.skills.retain(|_, v| !v.is_empty());

    if changed {
        save(&registry)?;
    }

    Ok(registry)
}

pub fn save(registry: &Registry) -> Result<()> {
    let path = registry_path()?;
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir).context("Failed to create registry directory")?;
    let content = serde_json::to_string_pretty(registry).context("Failed to serialize registry")?;
    fs::write(&path, content).context("Failed to write registry")?;
    Ok(())
}

/// Record a skill instance after a successful sync/copy.
pub fn record(skill_name: &str, path: &str, label: &str) -> Result<()> {
    let mut registry = load()?;
    let instances = registry.skills.entry(skill_name.to_string()).or_default();

    // Update existing entry for same path, or add new
    if let Some(existing) = instances.iter_mut().find(|i| i.path == path) {
        existing.synced_at = now_timestamp();
        existing.label = label.to_string();
    } else {
        instances.push(SkillInstance {
            path: path.to_string(),
            label: label.to_string(),
            synced_at: now_timestamp(),
        });
    }

    save(&registry)
}

/// Remove all instances of a skill at a specific path.
pub fn remove_path(skill_name: &str, path: &str) -> Result<()> {
    let mut registry = load()?;
    if let Some(instances) = registry.skills.get_mut(skill_name) {
        instances.retain(|i| i.path != path);
        if instances.is_empty() {
            registry.skills.remove(skill_name);
        }
    }
    save(&registry)
}

/// Remove all instances of a skill.
pub fn remove_skill(skill_name: &str) -> Result<()> {
    let mut registry = load()?;
    registry.skills.remove(skill_name);
    save(&registry)
}

/// Display all tracked skill instances.
pub fn where_all() -> Result<()> {
    let registry = load()?;

    if registry.skills.is_empty() {
        println!("No tracked skill instances found.");
        println!("Hint: Run `skillset sync` to sync skills and start tracking instances.");
        return Ok(());
    }

    println!(
        "Tracked skill instances ({} skill(s)):\n",
        registry.skills.len()
    );

    for (skill_name, instances) in &registry.skills {
        println!("  {}:", skill_name);
        for inst in instances {
            println!(
                "    {} — {} (synced {})",
                inst.label, inst.path, inst.synced_at
            );
        }
    }

    Ok(())
}

/// Display instances of a specific skill.
pub fn where_skill(skill_name: &str) -> Result<()> {
    let registry = load()?;

    match registry.skills.get(skill_name) {
        None => {
            println!("No tracked instances of '{}'.", skill_name);
            println!("Hint: Run `skillset sync` to sync skills and start tracking instances.");
        }
        Some(instances) => {
            println!("Instances of '{}' ({}):\n", skill_name, instances.len());
            for inst in instances {
                println!(
                    "  {} — {} (synced {})",
                    inst.label, inst.path, inst.synced_at
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_default_is_empty() {
        let reg = Registry::default();
        assert!(reg.skills.is_empty());
    }

    #[test]
    fn test_now_timestamp_format() {
        let ts = now_timestamp();
        assert!(ts.contains('-'));
        assert_eq!(ts.len(), 10); // YYYY-MM-DD
    }
}
