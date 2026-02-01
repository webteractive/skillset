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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub label: String,
    pub path: String,
}

fn default_source() -> String {
    ".ai/skills".to_string()
}

fn default_targets() -> Vec<Target> {
    vec![Target {
        label: "Cursor".to_string(),
        path: "~/.cursor/skills".to_string(),
    }]
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
        };
        save(&config)?;
        println!("Config created at: {}", path.display());
        return Ok(config);
    }

    let content = fs::read_to_string(&path).context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&content).context("Failed to parse config file")?;
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
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(&path[2..]).prepending(&home);
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
