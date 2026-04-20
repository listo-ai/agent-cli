mod config;
mod formatters;
mod health;
mod init;
mod registry;
mod skills;
mod sync;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "agent-cli",
    about = "Agent-agnostic MCP sync and skills bootstrap CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Drop a bundled mcp-compose.yaml template into the current repo.
    Init {
        /// Stack template to install.
        stack: String,
        /// Output path for the generated template.
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        output: PathBuf,
        /// Overwrite the target file if it already exists.
        #[arg(long)]
        force: bool,
    },
    /// Sync mcp-compose.yaml to all agents.
    Sync {
        /// Path to the mcp-compose.yaml file.
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        file: PathBuf,
    },
    /// Test health of all servers in mcp-compose.yaml.
    Test {
        #[arg(short, long, default_value = "mcp-compose.yaml")]
        file: PathBuf,
    },
    /// Manage stack-specific agent skills.
    Skills {
        #[arg(long, env = "AGENT_CLI_REGISTRY", default_value = registry::DEFAULT_REGISTRY_URL)]
        registry_url: String,
        #[command(subcommand)]
        command: SkillsCommand,
    },
}

#[derive(Subcommand)]
enum SkillsCommand {
    /// List available skills from the registry.
    List,
    /// Add one or more skills into ./docs/agents.
    Add {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Update one installed skill or all installed skills.
    Update { name: Option<String> },
    /// Remove an installed skill.
    Remove { name: String },
    /// Show currently installed skills.
    Installed,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            stack,
            output,
            force,
        } => init::write_template(&stack, &output, force)?,
        Commands::Sync { file } => {
            let compose = read_compose(&file)?;
            sync::sync_all(&compose)?;
        }
        Commands::Test { file } => {
            let compose = read_compose(&file)?;
            for (name, server) in &compose.servers {
                health::check_server_health(name, server).await?;
            }
        }
        Commands::Skills {
            registry_url,
            command,
        } => match command {
            SkillsCommand::List => skills::list_available(&registry_url).await?,
            SkillsCommand::Add { names } => skills::add(&names, &registry_url).await?,
            SkillsCommand::Update { name } => {
                skills::update(name.as_deref(), &registry_url).await?
            }
            SkillsCommand::Remove { name } => skills::remove(&name)?,
            SkillsCommand::Installed => skills::list_installed()?,
        },
    }

    Ok(())
}

fn read_compose(path: &PathBuf) -> Result<config::McpCompose> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Could not read config file {}", path.display()))?;
    serde_yml::from_str(&content).context("Invalid YAML format")
}
