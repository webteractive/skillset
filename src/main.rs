use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::IsTerminal;
use std::path::PathBuf;

mod add;
mod config;
mod doc;
mod install;
mod path;
mod remove;
mod skills;

use config::{config_path, load, supported_tools};
use doc::agents_md_snippet;
use path::resolve_source;
use skills::{discover_skills, sync_skills, OverwritePolicy};

#[derive(Parser)]
#[command(name = "skillset")]
#[command(about = "Manage and sync AI agent skills across multiple tools", long_about = None)]
struct Cli {
    /// Target user-level dir ~/.skillset/skills (source and install); without this, use workspace .skillset/skills
    #[arg(long, global = true)]
    user: bool,

    /// Skip overwrite prompts; always overwrite (sync and install)
    #[arg(short = 'y', long, global = true)]
    yes: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List skills in source and show status per target
    List,
    /// Sync skills from source to configured targets
    Sync,
    /// Install skills from a vendor/package (e.g., anthropics/skills)
    Install {
        /// Package spec in owner/repo format
        package: String,
        /// Install only the specified skill
        #[arg(long)]
        skill: Option<String>,
        /// After installing, sync skills from source to configured targets
        #[arg(long)]
        sync: bool,
    },
    /// Add/scaffold a new skill
    Add {
        /// Name of the skill to create
        name: String,
        /// Overwrite if skill already exists
        #[arg(long)]
        force: bool,
    },
    /// Remove a skill from targets (and optionally from user store)
    Remove {
        /// Name of the skill to remove
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Output documentation snippets
    Doc {
        /// Output AGENTS.md snippet
        #[arg(long)]
        agents_md: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_skills(cli.user)?,
        Commands::Sync => sync_skills_cli(cli.user, cli.yes)?,
        Commands::Install {
            package,
            skill,
            sync,
        } => install_package(package, skill.as_deref(), cli.user, sync, cli.yes)?,
        Commands::Add { name, force } => add_skill(name, cli.user, force)?,
        Commands::Remove { name, yes } => remove_skill(name, cli.user, yes || cli.yes)?,
        Commands::Doc { agents_md } => doc_output(agents_md)?,
    }

    Ok(())
}

fn list_skills(user_scope: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);
    let scope_label = if user_scope { "user" } else { "workspace" };

    println!("Source: {} ({})", source.display(), scope_label);
    println!("Config: {}\n", config_path()?.display());

    let skills = discover_skills(&source)?;

    if skills.is_empty() {
        println!("No skills found in source directory.");
        return Ok(());
    }

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    println!("Skills:");
    for skill in &skills {
        let mut statuses = Vec::new();
        for (label, target_path) in &targets {
            let skill_path = target_path.join(skill);
            if skill_path.exists() {
                statuses.push(format!("{} ✓", label));
            } else {
                statuses.push(format!("{} —", label));
            }
        }
        println!("  {}  {}", skill, statuses.join("  "));
    }

    Ok(())
}

/// Show a checkbox list of supported targets and return the subset the user selects.
/// When stdin is not a TTY, returns all targets (non-interactive).
fn select_sync_targets(targets: &[(String, PathBuf)]) -> Result<Vec<(String, PathBuf)>> {
    if targets.is_empty() {
        return Ok(vec![]);
    }
    if !std::io::stdin().is_terminal() {
        return Ok(targets.to_vec());
    }

    let items: Vec<String> = targets
        .iter()
        .map(|(label, path)| format!("{}  ({})", label, path.display()))
        .collect();
    // Preselect Cursor, Claude Code, Gemini, Codex
    const PRESELECTED: &[&str] = &["Cursor", "Claude Code", "Gemini", "Codex"];
    let default_selected: Vec<bool> = targets
        .iter()
        .map(|(label, _)| PRESELECTED.contains(&label.as_str()))
        .collect();

    let selected_indices =
        dialoguer::MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(
                "Supported tools that will receive the copy (Space to toggle, Enter to confirm)",
            )
            .items(&items)
            .defaults(&default_selected)
            .interact_opt()
            .context("Failed to show selection")?;

    let selected = match selected_indices {
        None => {
            println!("Selection cancelled.");
            return Ok(vec![]);
        }
        Some(indices) if indices.is_empty() => {
            println!("No targets selected. Nothing to sync.");
            return Ok(vec![]);
        }
        Some(indices) => indices.into_iter().map(|i| targets[i].clone()).collect(),
    };
    Ok(selected)
}

/// All supported tools with path chosen by scope so we never write to the wrong level.
/// Without --user (workspace): same tools, paths under cwd (e.g. .cursor/skills, .github/skills).
/// With --user: tools that have ~/ paths, expanded to home (e.g. ~/.cursor/skills).
fn sync_targets_for_scope(cwd: &std::path::Path, user_scope: bool) -> Vec<(String, PathBuf)> {
    supported_tools()
        .into_iter()
        .filter(|t| {
            // User scope: only tools that have a user-level path. Workspace scope: include all.
            if user_scope {
                t.path.starts_with("~/")
            } else {
                true
            }
        })
        .map(|t| {
            let path = if user_scope {
                config::expand_home(&t.path)
            } else if t.path.starts_with("~/") {
                // Workspace: ~/.cursor/skills -> cwd/.cursor/skills
                cwd.join(t.path.strip_prefix("~/").unwrap())
            } else {
                cwd.join(&t.path)
            };
            (t.label, path)
        })
        .collect()
}

fn sync_skills_cli(user_scope: bool, yes: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);

    if !source.exists() {
        anyhow::bail!("Source directory not found: {}", source.display());
    }

    let scope_label = if user_scope {
        "user"
    } else {
        "workspace (cwd)"
    };
    println!("Source: {} ({})", source.display(), scope_label);

    let targets = sync_targets_for_scope(&cwd, user_scope);
    let selected = select_sync_targets(&targets)?;
    if selected.is_empty() {
        println!("No targets selected. Nothing to sync.");
        return Ok(());
    }

    let mut overwrite_policy = if yes {
        OverwritePolicy::All
    } else {
        OverwritePolicy::PerSkill
    };
    sync_skills(&source, &selected, &mut overwrite_policy)?;

    Ok(())
}

fn install_package(
    package: String,
    skill: Option<&str>,
    user_scope: bool,
    do_sync: bool,
    yes: bool,
) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;

    // --sync: targets filtered by scope (workspace vs user)
    let targets = sync_targets_for_scope(&cwd, user_scope);

    // Without --user: install to workspace source (cwd/.skillset/skills). With --user: to ~/.skillset/skills only.
    let source_dir = if user_scope {
        None
    } else {
        Some(resolve_source(false, &cwd, &config.source))
    };

    let user_store_dir = if user_scope {
        Some(resolve_source(true, &cwd, &config.source))
    } else {
        None
    };
    install::install_package(
        &package,
        skill,
        source_dir.as_deref(),
        user_store_dir.as_deref(),
        yes,
        config.install.use_ssh,
    )?;

    if do_sync {
        let source = resolve_source(user_scope, &cwd, &config.source);
        if !source.exists() {
            anyhow::bail!(
                "Source directory not found: {} (cannot sync)",
                source.display()
            );
        }
        let scope_label = if user_scope { "user" } else { "workspace" };
        println!("\nSyncing from {} ({})", source.display(), scope_label);
        let selected = select_sync_targets(&targets)?;
        if selected.is_empty() {
            println!("No targets selected. Skipping sync.");
        } else {
            let mut overwrite_policy = if yes {
                OverwritePolicy::All
            } else {
                OverwritePolicy::PerSkill
            };
            sync_skills(&source, &selected, &mut overwrite_policy)?;
        }
    }

    Ok(())
}

fn add_skill(name: String, user_scope: bool, force: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);
    add::add_skill(&name, &source, user_scope, force)?;
    Ok(())
}

fn remove_skill(name: String, user_scope: bool, yes: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    let user_source = if user_scope {
        Some(resolve_source(true, &cwd, &config.source))
    } else {
        None
    };
    remove::remove_skill(&name, &targets, user_source.as_deref(), yes)?;

    Ok(())
}

fn doc_output(agents_md: bool) -> Result<()> {
    if agents_md {
        println!("{}", agents_md_snippet());
    } else {
        println!("Use --agents-md to output the AGENTS.md snippet.");
    }
    Ok(())
}
