use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_targets")]
    pub targets: Vec<Target>,
    #[serde(default)]
    pub install: InstallConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub label: String,
    pub path: String,
}

fn default_skill_dirs() -> Vec<String> {
    vec![".claude/skills".to_string(), "skills".to_string()]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallConfig {
    /// When true, use SSH URL (git@github.com:owner/repo.git) for owner/repo specs
    #[serde(default)]
    pub use_ssh: bool,
    /// Directories to look for skills in when installing from a repo (relative to repo root)
    #[serde(default = "default_skill_dirs")]
    pub skill_dirs: Vec<String>,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            use_ssh: false,
            skill_dirs: default_skill_dirs(),
        }
    }
}

fn default_source() -> String {
    ".skillset/skills".to_string()
}

/// Tools/CLIs that support agent skills (SKILL.md). Used for default config and reference.
pub fn supported_tools() -> Vec<Target> {
    vec![
        Target {
            label: "Cursor".to_string(),
            path: "~/.cursor/skills".to_string(),
        },
        Target {
            label: "Claude Code".to_string(),
            path: "~/.claude/skills".to_string(),
        },
        Target {
            label: "Windsurf".to_string(),
            path: "~/.windsurf/skills".to_string(),
        },
        Target {
            label: "Codex".to_string(),
            path: "~/.codex/skills".to_string(),
        },
        Target {
            label: "OpenCode".to_string(),
            path: "~/.opencode/skills".to_string(),
        },
        Target {
            label: "Gemini".to_string(),
            path: "~/.gemini/skills".to_string(),
        },
        Target {
            label: "GitHub Copilot (project)".to_string(),
            path: ".github/skills".to_string(),
        },
        Target {
            label: "GitHub Copilot (personal)".to_string(),
            path: "~/.copilot/skills".to_string(),
        },
    ]
}

fn default_targets() -> Vec<Target> {
    supported_tools()
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "skillset")
        .context("Failed to determine config directory")?;
    Ok(dirs.config_dir().to_path_buf())
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        let config = Config {
            source: default_source(),
            targets: default_targets(),
            install: InstallConfig::default(),
        };
        save(&config)?;
        println!("Config created at: {}", path.display());
        return Ok(config);
    }

    let content = fs::read_to_string(&path).context("Failed to read config file")?;
    let mut config: Config =
        serde_json::from_str(&content).context("Failed to parse config file")?;

    // Migrate legacy .ai/skills to .skillset/skills
    if config.source == ".ai/skills" {
        config.source = default_source();
        save(&config)?;
    }

    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir).context("Failed to create config directory")?;
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content).context("Failed to write config file")?;
    Ok(())
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(stripped).prepending(&home);
        }
    }
    PathBuf::from(path)
}

trait PathPrepend {
    fn prepending(&self, base: &str) -> PathBuf;
}

impl PathPrepend for PathBuf {
    fn prepending(&self, base: &str) -> PathBuf {
        let mut result = PathBuf::from(base);
        result.push(self);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_tools_returns_all_tools() {
        let tools = supported_tools();
        assert_eq!(tools.len(), 8, "supported_tools should return 8 tools");
        let labels: Vec<&str> = tools.iter().map(|t| t.label.as_str()).collect();
        assert!(labels.contains(&"Cursor"));
        assert!(labels.contains(&"Claude Code"));
        assert!(labels.contains(&"Gemini"));
        assert!(labels.contains(&"Codex"));
        assert!(labels.contains(&"GitHub Copilot (project)"));
        assert!(labels.contains(&"GitHub Copilot (personal)"));
    }

    #[test]
    fn test_supported_tools_user_vs_workspace_paths() {
        let tools = supported_tools();
        let user_level: Vec<_> = tools.iter().filter(|t| t.path.starts_with("~/")).collect();
        let workspace_level: Vec<_> = tools.iter().filter(|t| !t.path.starts_with("~/")).collect();
        assert_eq!(user_level.len(), 7, "7 tools use user-level paths (~/...)");
        assert_eq!(
            workspace_level.len(),
            1,
            "1 tool uses workspace-relative path"
        );
        assert_eq!(workspace_level[0].path, ".github/skills");
    }

    #[test]
    fn test_expand_home() {
        std::env::set_var("HOME", "/home/user");
        assert_eq!(
            expand_home("~/.skillset/skills"),
            PathBuf::from("/home/user/.skillset/skills")
        );
        assert_eq!(
            expand_home("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
    }
}
