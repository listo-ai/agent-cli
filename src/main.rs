mod config;
mod formatters;
mod health;
mod init;
mod store;
mod sync;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "agent-cli",
    version,
    about = "Agent-agnostic CLI for managing AI coding agent skills and MCP servers."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed skills.
    Ls,

    /// List all available skills from built-in and custom registries.
    #[command(name = "ls-remote")]
    LsRemote,

    /// Clone and install a skill by name.
    Install {
        /// Skill name (e.g. rust, flutter). See `ls-remote` for available skills.
        name: String,
    },

    /// Pull latest changes for one skill or all installed skills.
    Update {
        /// Skill name to update. Omit to update all.
        name: Option<String>,
    },

    /// Remove an installed skill.
    Remove {
        /// Skill name to remove.
        name: String,
    },

    /// Print the content of an installed skill's main file.
    Show {
        /// Skill name.
        name: String,
    },

    /// Print the filesystem path of an installed skill's main file.
    Path {
        /// Skill name.
        name: String,
    },

    /// Manage skill registries.
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },

    // ── MCP commands (Phase 0, unchanged) ─────────────────────────────────────

    /// Drop a bundled mcp-compose.yaml template into the current repo.
    Init {
        stack: String,
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        output: PathBuf,
        #[arg(long)]
        force: bool,
    },

    /// Sync mcp-compose.yaml to all agents.
    Sync {
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        file: PathBuf,
    },

    /// Test health of all servers in mcp-compose.yaml.
    Health {
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum RegistryCommand {
    /// List all registries (built-in + custom).
    List,
    /// Add a custom skill registry.
    Add {
        /// Short name for the registry.
        name: String,
        /// Git URL of the skills repo.
        url: String,
        /// Path to the skill file inside the repo.
        #[arg(long, default_value = "SKILL.md")]
        skill_file: String,
        /// Short description.
        #[arg(long, default_value = "")]
        description: String,
    },
    /// Remove a custom registry (built-in registries cannot be removed).
    Remove {
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ls => store::Store::open()?.ls()?,
        Commands::LsRemote => store::Store::open()?.ls_remote()?,
        Commands::Install { name } => store::Store::open()?.install(&name)?,
        Commands::Update { name } => store::Store::open()?.update(name.as_deref())?,
        Commands::Remove { name } => store::Store::open()?.remove(&name)?,
        Commands::Show { name } => store::Store::open()?.show(&name)?,
        Commands::Path { name } => store::Store::open()?.path(&name)?,

        Commands::Registry { command } => {
            let mut store = store::Store::open()?;
            match command {
                RegistryCommand::List => store.registry_list()?,
                RegistryCommand::Add { name, url, skill_file, description } => {
                    store.registry_add(
                        name, url,
                        Some(skill_file),
                        Some(description),
                    )?
                }
                RegistryCommand::Remove { name } => store.registry_remove(&name)?,
            }
        }

        Commands::Init { stack, output, force } => init::write_template(&stack, &output, force)?,
        Commands::Sync { file } => sync::sync_all(&read_compose(&file)?)?,
        Commands::Health { file } => {
            let compose = read_compose(&file)?;
            for (name, server) in &compose.servers {
                health::check_server_health(name, server).await?;
            }
        }
    }

    Ok(())
}

fn read_compose(path: &PathBuf) -> Result<config::McpCompose> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Could not read {}", path.display()))?;
    serde_yml::from_str(&content).context("Invalid YAML format")
}
