use std::path::{Path, PathBuf};

/// Resolve the source path based on user scope flag.
/// If user_scope is true, use ~/.ai/skills (user-level).
/// If user_scope is false, use cwd.join(config_source) (workspace).
pub fn resolve_source(user_scope: bool, cwd: &Path, config_source: &str) -> PathBuf {
    if user_scope {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".ai/skills");
        }
    }
    cwd.join(config_source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_source_user_scope() {
        std::env::set_var("HOME", "/tmp/test_home");
        let result = resolve_source(true, Path::new("/workspace"), ".ai/skills");
        assert_eq!(result, PathBuf::from("/tmp/test_home/.ai/skills"));
    }

    #[test]
    fn test_resolve_source_workspace() {
        let result = resolve_source(false, Path::new("/workspace"), ".ai/skills");
        assert_eq!(result, PathBuf::from("/workspace/.ai/skills"));
    }
}
