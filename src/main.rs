use anyhow::Result;
use clap::{Parser, Subcommand};
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
use skills::{discover_skills, OverwritePolicy, sync_skills};

#[derive(Parser)]
#[command(name = "skillset")]
#[command(about = "Manage and sync AI agent skills across multiple tools", long_about = None)]
struct Cli {
    /// Use user-level scope (~/.ai/skills instead of cwd/.ai/skills)
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
        Commands::Install { package, skill } => install_package(package, skill.as_deref(), cli.user)?,
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

    let mut overwrite_policy = OverwritePolicy::PerSkill;
    sync_skills(&source, &targets, &mut overwrite_policy)?;

    Ok(())
}

fn install_package(package: String, skill: Option<&str>, install_to_user: bool) -> Result<()> {
    let config = load()?;

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    install::install_package(&package, skill, &targets, install_to_user)?;

    Ok(())
}

fn add_skill(name: String, user_scope: bool, force: bool) -> Result<()> {
    add::add_skill(&name, user_scope, force)?;
    Ok(())
}

fn remove_skill(name: String, user_scope: bool, yes: bool) -> Result<()> {
    let config = load()?;

    // Expand target paths
    let targets: Vec<(String, PathBuf)> = config
        .targets
        .iter()
        .map(|t| (t.label.clone(), config::expand_home(&t.path)))
        .collect();

    remove::remove_skill(&name, &targets, user_scope, yes)?;

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
