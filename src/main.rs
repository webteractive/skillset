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

use config::{config_path, load};
use doc::agents_md_snippet;
use path::resolve_source;
use skills::{discover_skills, sync_skills, OverwritePolicy};

#[derive(Parser)]
#[command(name = "skillset")]
#[command(about = "Manage and sync AI agent skills across multiple tools", long_about = None)]
struct Cli {
    /// Use user-level scope (~/.skillset/skills instead of cwd/.skillset/skills)
    #[arg(long, global = true)]
    user: bool,

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
        Commands::Sync => sync_skills_cli(cli.user)?,
        Commands::Install {
            package,
            skill,
            sync,
        } => install_package(package, skill.as_deref(), cli.user, sync)?,
        Commands::Add { name, force } => add_skill(name, cli.user, force)?,
        Commands::Remove { name, yes } => remove_skill(name, cli.user, yes)?,
        Commands::Doc { agents_md } => doc_output(agents_md)?,
    }

    Ok(())
}

fn list_skills(user_scope: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);

    println!("Source: {}", source.display());
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

/// Show a checklist of configured targets and return the subset the user selects.
/// When stdin is not a TTY, returns all targets (non-interactive).
fn select_sync_targets(targets: &[(String, PathBuf)]) -> Result<Vec<(String, PathBuf)>> {
    if targets.is_empty() {
        return Ok(vec![]);
    }
    if !std::io::stdin().is_terminal() {
        return Ok(targets.to_vec());
    }

    println!("\nSync skills to (supported tools):");
    for (i, (label, path)) in targets.iter().enumerate() {
        println!("  [{}] {}  ({})", i + 1, label, path.display());
    }
    print!("\nSelect targets (e.g. 1,3,5 or 'all') [all]: ");
    std::io::Write::flush(&mut std::io::stdout()).context("Flush stdout")?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("Read selection")?;
    let input = input.trim();

    if input.is_empty() || input.eq_ignore_ascii_case("all") {
        return Ok(targets.to_vec());
    }

    let selected: Vec<(String, PathBuf)> = input
        .split(',')
        .filter_map(|s| {
            let s = s.trim();
            s.parse::<usize>().ok().and_then(|n| {
                if n >= 1 && n <= targets.len() {
                    Some(targets[n - 1].clone())
                } else {
                    None
                }
            })
        })
        .collect();
    if selected.is_empty() {
        anyhow::bail!(
            "No valid targets selected. Use numbers 1-{} or 'all'.",
            targets.len()
        );
    }
    Ok(selected)
}

fn sync_skills_cli(user_scope: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);

    if !source.exists() {
        anyhow::bail!("Source directory not found: {}", source.display());
    }

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    let selected = select_sync_targets(&targets)?;
    if selected.is_empty() {
        println!("No targets selected. Nothing to sync.");
        return Ok(());
    }

    let mut overwrite_policy = OverwritePolicy::PerSkill;
    sync_skills(&source, &selected, &mut overwrite_policy)?;

    Ok(())
}

fn install_package(
    package: String,
    skill: Option<&str>,
    user_scope: bool,
    do_sync: bool,
) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    // Without --user: also install to workspace source (cwd/.skillset/skills). With --user: only targets + ~/.skillset/skills.
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
        &targets,
        source_dir.as_deref(),
        user_store_dir.as_deref(),
    )?;

    if do_sync {
        let source = resolve_source(user_scope, &cwd, &config.source);
        if !source.exists() {
            anyhow::bail!(
                "Source directory not found: {} (cannot sync)",
                source.display()
            );
        }
        let selected = select_sync_targets(&targets)?;
        if selected.is_empty() {
            println!("No targets selected. Skipping sync.");
        } else {
            let mut overwrite_policy = OverwritePolicy::PerSkill;
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
