use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

mod add;
mod config;
mod doc;
mod install;
mod path;
mod registry;
mod remove;
mod skills;
mod validate;

use config::{config_path, load, supported_tools};
use doc::agents_md_snippet;
use path::resolve_source;
use skills::{discover_skills, sync_skills, OverwritePolicy};

#[derive(Parser)]
#[command(name = "skillset", version)]
#[command(about = "Manage and sync AI agent skills across multiple tools", long_about = None)]
struct Cli {
    /// Target user-level dir ~/.skillset/skills (source and install); without this, use workspace .skillset/skills
    #[arg(long, short = 'G', global = true)]
    user: bool,

    /// Skip all prompts: overwrite existing, accept defaults, no target selection
    #[arg(long, global = true)]
    force: bool,

    /// Alias for --force (hidden, backward compat)
    #[arg(short = 'y', long, global = true, hide = true)]
    yes: bool,

    /// Preview what would happen without making changes
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List skills in source and show status per target (or a specific tool via --tool)
    List {
        /// Show the skills currently loaded for a specific tool (e.g., --tool=codex)
        #[arg(long)]
        tool: Option<String>,
        /// Filter skills by name pattern
        #[arg(long)]
        filter: Option<String>,
        /// Filter by sync status: synced, missing, or all
        #[arg(long, default_value = "all")]
        status: String,
    },
    /// Sync skills from source to configured targets
    Sync {
        /// Show diff of SKILL.md before overwriting
        #[arg(long)]
        diff: bool,
    },
    /// Install skills from a vendor/package (e.g., anthropics/skills)
    Install {
        /// Package spec in owner/repo format or full Git URL
        package: String,
        /// Install only the specified skill
        #[arg(long)]
        skill: Option<String>,
        /// After installing, sync skills from source to configured targets
        #[arg(long)]
        sync: bool,
        /// Comma-separated dirs to look for skills in (e.g., .cursor/skills,.claude/skills,skills)
        #[arg(long)]
        dir: Option<String>,
        /// Pull latest from remote before installing (refreshes cached repo)
        #[arg(long)]
        from_remote: bool,
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
    /// Validate skills in source directory (check SKILL.md frontmatter)
    Validate,
    /// Show where skills are installed (tracked instances)
    Where {
        /// Show instances for a specific skill (omit for all)
        skill: Option<String>,
        /// Scan all targets and register existing skill instances
        #[arg(long)]
        scan: bool,
    },
    /// Manage configuration (show, add/remove targets, reset)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Update skillset to the latest version
    #[command(name = "self-update")]
    SelfUpdate,
    /// Output documentation snippets
    Doc {
        /// Output AGENTS.md snippet
        #[arg(long)]
        agents_md: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Add a sync target
    AddTarget {
        /// Label for the target (e.g., "My Editor")
        label: String,
        /// Path to the skills directory (e.g., ~/.myeditor/skills)
        path: String,
    },
    /// Remove a sync target by label
    RemoveTarget {
        /// Label of the target to remove
        label: String,
    },
    /// Reset configuration to defaults
    Reset,
    /// Validate that configured target paths exist
    ValidatePaths,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let force = cli.force || cli.yes;
    let dry_run = cli.dry_run;

    match cli.command {
        Commands::List {
            tool,
            filter,
            status,
        } => list_skills(cli.user, tool.as_deref(), filter.as_deref(), &status)?,
        Commands::Sync { diff } => sync_skills_cli(cli.user, force, dry_run, diff)?,
        Commands::Install {
            package,
            skill,
            sync,
            dir,
            from_remote,
        } => install_package(
            package,
            skill.as_deref(),
            cli.user,
            sync,
            force,
            dir.as_deref(),
            from_remote,
            dry_run,
        )?,
        Commands::Add {
            name,
            force: cmd_force,
        } => add_skill(name, cli.user, cmd_force || force)?,
        Commands::Remove { name, yes } => remove_skill(name, cli.user, yes || force)?,
        Commands::Validate => validate_skills(cli.user)?,
        Commands::Where { skill, scan } => {
            if scan {
                scan_existing_instances()?;
            }
            where_skills(skill.as_deref())?;
        }
        Commands::Config { action } => config_command(action)?,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "skillset", &mut std::io::stdout());
        }
        Commands::SelfUpdate => self_update()?,
        Commands::Doc { agents_md } => doc_output(agents_md)?,
    }

    Ok(())
}

fn list_skills(
    user_scope: bool,
    tool: Option<&str>,
    filter: Option<&str>,
    status: &str,
) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);
    let scope_label = if user_scope { "user" } else { "workspace" };

    println!("Source: {} ({})", source.display(), scope_label);
    println!("Config: {}\n", config_path()?.display());

    match tool {
        Some(tool_name) => {
            list_skills_for_tool(tool_name, user_scope, &cwd)?;
        }
        None => {
            list_skills_with_status(&config, &source, filter, status)?;
        }
    }

    Ok(())
}

fn list_skills_with_status(
    config: &config::Config,
    source: &Path,
    filter: Option<&str>,
    status_filter: &str,
) -> Result<()> {
    let mut skills = discover_skills(source)?;

    if skills.is_empty() {
        println!("No skills found in source directory.");
        return Ok(());
    }

    // Apply name filter
    if let Some(pattern) = filter {
        let pattern_lower = pattern.to_lowercase();
        skills.retain(|s| s.to_lowercase().contains(&pattern_lower));
        if skills.is_empty() {
            println!("No skills matching '{}' found.", pattern);
            return Ok(());
        }
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
        let mut synced_count = 0;
        let total_targets = targets.len();

        for (label, target_path) in &targets {
            let skill_path = target_path.join(skill);
            if skill_path.exists() {
                statuses.push(format!("{} ✓", label));
                synced_count += 1;
            } else {
                statuses.push(format!("{} —", label));
            }
        }

        // Apply status filter
        let show = match status_filter {
            "synced" => synced_count == total_targets,
            "missing" => synced_count < total_targets,
            _ => true,
        };

        if show {
            println!("  {}  {}", skill, statuses.join("  "));
        }
    }

    Ok(())
}

fn list_skills_for_tool(tool: &str, user_scope: bool, cwd: &Path) -> Result<()> {
    let tool_targets = supported_tools();
    let tool_lower = tool.to_lowercase();
    let mut matches = tool_targets
        .into_iter()
        .filter(|t| t.label.to_lowercase().contains(&tool_lower))
        .collect::<Vec<_>>();

    if matches.is_empty() {
        anyhow::bail!(
            "Unknown tool '{}'. Try one of the supported tool names.\nHint: Run `skillset list` to see all supported tools.",
            tool
        );
    }

    // When multiple tools match the filter, prefer exact case-insensitive match, otherwise first result.
    if matches.len() > 1 {
        if let Some(exact) = matches
            .iter()
            .find(|t| t.label.eq_ignore_ascii_case(tool))
            .cloned()
        {
            matches = vec![exact];
        }
    }

    for target in matches {
        let path = if user_scope || target.path.starts_with("~/") {
            config::expand_home(&target.path)
        } else {
            cwd.join(target.path.strip_prefix("~/").unwrap_or(&target.path))
        };
        println!("Tool: {}", target.label);
        println!("Path: {}", path.display());

        if !path.exists() {
            println!("(directory not found)\nHint: {} may not be installed yet, or the skills directory hasn't been created.", target.label);
            continue;
        }

        let tool_skills = discover_skills(&path)?;
        if tool_skills.is_empty() {
            println!("No skills found for this tool.\n");
            continue;
        }

        println!("Skills:");
        for skill in tool_skills {
            println!("  {}", skill);
        }
        println!();
    }

    Ok(())
}

/// Show a checkbox list of supported targets and return the subset the user selects.
/// When --force is set or stdin is not a TTY, returns all targets without prompting.
fn select_sync_targets(
    targets: &[(String, PathBuf)],
    force: bool,
) -> Result<Vec<(String, PathBuf)>> {
    if targets.is_empty() {
        return Ok(vec![]);
    }
    if force || !std::io::stdin().is_terminal() {
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

fn sync_skills_cli(user_scope: bool, force: bool, dry_run: bool, show_diff: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);

    if !source.exists() {
        anyhow::bail!(
            "Source directory not found: {}\nHint: Run `skillset add <name>` to create your first skill, or `skillset install <package>` to install from a repo.",
            source.display()
        );
    }

    let scope_label = if user_scope {
        "user"
    } else {
        "workspace (cwd)"
    };

    if dry_run {
        println!("[DRY RUN] Source: {} ({})", source.display(), scope_label);
    } else {
        println!("Source: {} ({})", source.display(), scope_label);
    }

    let targets = sync_targets_for_scope(&cwd, user_scope);
    let selected = if dry_run {
        targets.clone()
    } else {
        select_sync_targets(&targets, force)?
    };
    if selected.is_empty() {
        println!("No targets selected. Nothing to sync.");
        return Ok(());
    }

    let mut overwrite_policy = if force || dry_run {
        OverwritePolicy::All
    } else {
        OverwritePolicy::PerSkill
    };
    sync_skills(
        &source,
        &selected,
        &mut overwrite_policy,
        dry_run,
        show_diff,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn install_package(
    package: String,
    skill: Option<&str>,
    user_scope: bool,
    do_sync: bool,
    force: bool,
    dir: Option<&str>,
    from_remote: bool,
    dry_run: bool,
) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;

    if dry_run {
        println!("[DRY RUN] Would install from package: {}", package);
        if let Some(s) = skill {
            println!("[DRY RUN] Filtered to skill: {}", s);
        }
        let scope = if user_scope {
            "user store"
        } else {
            "workspace"
        };
        println!("[DRY RUN] Target scope: {}", scope);
        if do_sync {
            let targets = sync_targets_for_scope(&cwd, user_scope);
            println!("[DRY RUN] Would sync to {} target(s):", targets.len());
            for (label, path) in &targets {
                println!("[DRY RUN]   {} ({})", label, path.display());
            }
        }
        println!("[DRY RUN] No changes made.");
        return Ok(());
    }

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

    let skill_dirs: Vec<String> = if let Some(d) = dir {
        d.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        config.install.skill_dirs.clone()
    };

    install::install_package(
        &package,
        skill,
        source_dir.as_deref(),
        user_store_dir.as_deref(),
        force,
        config.install.use_ssh,
        &skill_dirs,
        from_remote,
    )?;

    if !do_sync {
        if user_scope {
            println!(
                "\nNext: run `skillset sync --user` (or `skillset sync -G`) to copy skills to your configured tools."
            );
        } else {
            println!(
                "\nNext: run `skillset sync` to copy skills to your configured tools (Cursor, Claude Code, etc.)."
            );
        }
    }

    if do_sync {
        let source = resolve_source(user_scope, &cwd, &config.source);
        if !source.exists() {
            anyhow::bail!(
                "Source directory not found: {} (cannot sync)\nHint: The install may have failed or the source path is misconfigured.",
                source.display()
            );
        }
        let scope_label = if user_scope { "user" } else { "workspace" };
        println!("\nSyncing from {} ({})", source.display(), scope_label);
        let selected = select_sync_targets(&targets, force)?;
        if selected.is_empty() {
            println!("No targets selected. Skipping sync.");
        } else {
            let mut overwrite_policy = if force {
                OverwritePolicy::All
            } else {
                OverwritePolicy::PerSkill
            };
            sync_skills(&source, &selected, &mut overwrite_policy, false, false)?;
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

fn validate_skills(user_scope: bool) -> Result<()> {
    let config = load()?;
    let cwd = std::env::current_dir()?;
    let source = resolve_source(user_scope, &cwd, &config.source);

    if !source.exists() {
        anyhow::bail!(
            "Source directory not found: {}\nHint: Run `skillset add <name>` to create your first skill.",
            source.display()
        );
    }

    let skills = discover_skills(&source)?;
    if skills.is_empty() {
        println!("No skills found in source directory.");
        return Ok(());
    }

    validate::validate_skills(&source, &skills)
}

fn scan_existing_instances() -> Result<()> {
    let tools = supported_tools();
    let mut count = 0;

    println!("Scanning targets for existing skill instances...\n");

    for tool in &tools {
        let path = config::expand_home(&tool.path);
        if !path.exists() {
            continue;
        }

        let skills = discover_skills(&path)?;
        for skill_name in &skills {
            let skill_path = path.join(skill_name);
            registry::record(skill_name, &skill_path.to_string_lossy(), &tool.label)?;
            count += 1;
        }
    }

    if count == 0 {
        println!("No existing skill instances found across targets.");
    } else {
        println!("Registered {} skill instance(s).\n", count);
    }

    Ok(())
}

fn where_skills(skill: Option<&str>) -> Result<()> {
    match skill {
        Some(name) => registry::where_skill(name)?,
        None => registry::where_all()?,
    }
    Ok(())
}

fn config_command(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = load()?;
            let path = config_path()?;
            println!("Config file: {}\n", path.display());
            println!("Source: {}\n", config.source);
            println!("Install:");
            println!("  use_ssh: {}", config.install.use_ssh);
            println!("  skill_dirs: {}\n", config.install.skill_dirs.join(", "));
            println!("Targets ({}):", config.targets.len());
            for target in &config.targets {
                let expanded = config::expand_home(&target.path);
                let exists = expanded.exists();
                let status = if exists { "✓" } else { "—" };
                println!("  {} {} ({})", status, target.label, target.path);
            }
        }
        ConfigAction::AddTarget { label, path } => {
            let mut config = load()?;
            // Check if label already exists
            if config.targets.iter().any(|t| t.label == label) {
                anyhow::bail!("Target '{}' already exists. Remove it first with `skillset config remove-target \"{}\"`.", label, label);
            }
            config.targets.push(config::Target {
                label: label.clone(),
                path: path.clone(),
            });
            config::save(&config)?;
            println!("Added target: {} ({})", label, path);
        }
        ConfigAction::RemoveTarget { label } => {
            let mut config = load()?;
            let before = config.targets.len();
            config.targets.retain(|t| t.label != label);
            if config.targets.len() == before {
                anyhow::bail!(
                    "Target '{}' not found.\nHint: Run `skillset config show` to see configured targets.",
                    label
                );
            }
            config::save(&config)?;
            println!("Removed target: {}", label);
        }
        ConfigAction::Reset => {
            let config = config::Config {
                source: ".skillset/skills".to_string(),
                targets: supported_tools()
                    .into_iter()
                    .map(|t| config::Target {
                        label: t.label,
                        path: t.path,
                    })
                    .collect(),
                install: config::InstallConfig::default(),
            };
            config::save(&config)?;
            println!("Configuration reset to defaults.");
        }
        ConfigAction::ValidatePaths => {
            let config = load()?;
            println!("Checking target paths:\n");
            let mut all_ok = true;
            for target in &config.targets {
                let expanded = config::expand_home(&target.path);
                if expanded.exists() {
                    println!("  ✓ {} — {}", target.label, expanded.display());
                } else {
                    println!("  ✗ {} — {} (not found)", target.label, expanded.display());
                    all_ok = false;
                }
            }
            if all_ok {
                println!("\nAll target paths exist.");
            } else {
                println!("\nSome target paths are missing. These tools may not be installed yet.");
            }
        }
    }
    Ok(())
}

fn self_update() -> Result<()> {
    println!("Updating skillset to the latest version...");
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg("curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download")
        .status()
        .context("Failed to run install script.\nHint: Check your internet connection and try again.")?;

    if !status.success() {
        anyhow::bail!("Self-update failed.\nHint: Try running the install script manually: curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh");
    }

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
