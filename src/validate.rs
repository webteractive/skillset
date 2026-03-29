use anyhow::Result;
use std::fs;
use std::path::Path;

/// Parsed frontmatter from a SKILL.md file.
#[derive(Debug, Default)]
pub struct SkillMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
}

/// Parse YAML-like frontmatter from a SKILL.md file.
/// Expects content between --- delimiters at the start of the file.
pub fn parse_frontmatter(content: &str) -> Option<SkillMeta> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let yaml_block = &after_first[..end];

    let mut meta = SkillMeta::default();

    for line in yaml_block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "name" => meta.name = Some(value.to_string()),
                "description" => meta.description = Some(value.to_string()),
                "author" => meta.author = Some(value.to_string()),
                "version" => meta.version = Some(value.to_string()),
                "tags" => {
                    // Parse [tag1, tag2, tag3] format
                    let stripped = value.trim_start_matches('[').trim_end_matches(']');
                    meta.tags = stripped
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    Some(meta)
}

/// Validate all skills in the source directory.
pub fn validate_skills(source: &Path, skills: &[String]) -> Result<()> {
    let mut errors = 0;
    let mut warnings = 0;

    println!("Validating {} skill(s):\n", skills.len());

    for skill_name in skills {
        let skill_md = source.join(skill_name).join("SKILL.md");

        if !skill_md.exists() {
            println!("  ✗ {} — SKILL.md not found", skill_name);
            errors += 1;
            continue;
        }

        let content = fs::read_to_string(&skill_md)?;

        if content.trim().is_empty() {
            println!("  ✗ {} — SKILL.md is empty", skill_name);
            errors += 1;
            continue;
        }

        match parse_frontmatter(&content) {
            None => {
                println!(
                    "  ✗ {} — missing frontmatter (expected --- delimiters at start)",
                    skill_name
                );
                errors += 1;
            }
            Some(meta) => {
                let mut issues = Vec::new();

                if meta.name.is_none() || meta.name.as_deref() == Some("") {
                    issues.push("missing 'name'");
                }
                if meta.description.is_none() || meta.description.as_deref() == Some("") {
                    issues.push("missing 'description'");
                }
                if meta
                    .description
                    .as_deref()
                    .map(|d| d.starts_with("A brief description"))
                    .unwrap_or(false)
                {
                    issues.push("description is still the template placeholder");
                }

                if issues.is_empty() {
                    let mut extras = Vec::new();
                    if meta.author.is_some() {
                        extras.push("author");
                    }
                    if meta.version.is_some() {
                        extras.push("version");
                    }
                    if !meta.tags.is_empty() {
                        extras.push("tags");
                    }
                    let extra_str = if extras.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", extras.join(", "))
                    };
                    println!("  ✓ {}{}", skill_name, extra_str);
                } else {
                    let severity = if issues.iter().any(|i| i.starts_with("missing")) {
                        errors += 1;
                        "✗"
                    } else {
                        warnings += 1;
                        "⚠"
                    };
                    println!("  {} {} — {}", severity, skill_name, issues.join(", "));
                }
            }
        }
    }

    println!();
    if errors == 0 && warnings == 0 {
        println!("All {} skill(s) are valid.", skills.len());
    } else {
        if errors > 0 {
            println!("{} error(s) found.", errors);
        }
        if warnings > 0 {
            println!("{} warning(s) found.", warnings);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: my-skill
description: Does something cool
author: Glen
version: 1.0.0
tags: [testing, automation]
---

# My Skill
"#;
        let meta = parse_frontmatter(content).unwrap();
        assert_eq!(meta.name.as_deref(), Some("my-skill"));
        assert_eq!(meta.description.as_deref(), Some("Does something cool"));
        assert_eq!(meta.author.as_deref(), Some("Glen"));
        assert_eq!(meta.version.as_deref(), Some("1.0.0"));
        assert_eq!(meta.tags, vec!["testing", "automation"]);
    }

    #[test]
    fn test_parse_frontmatter_missing_fields() {
        let content = "---\nname: test\n---\n# Test";
        let meta = parse_frontmatter(content).unwrap();
        assert_eq!(meta.name.as_deref(), Some("test"));
        assert!(meta.description.is_none());
    }

    #[test]
    fn test_parse_frontmatter_no_delimiters() {
        let content = "# No frontmatter here";
        assert!(parse_frontmatter(content).is_none());
    }

    #[test]
    fn test_parse_frontmatter_empty_tags() {
        let content = "---\nname: test\ndescription: test\ntags: []\n---\n";
        let meta = parse_frontmatter(content).unwrap();
        assert!(meta.tags.is_empty());
    }
}
