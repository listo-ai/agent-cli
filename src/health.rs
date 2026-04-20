use crate::config::ServerConfig;
use anyhow::Result;
use std::process::Command;
use std::time::Duration;

pub async fn check_server_health(name: &str, config: &ServerConfig) -> Result<()> {
    println!("testing MCP server health: {name}");

    let mut child = Command::new(&config.command)
        .args(&config.args)
        .envs(&config.env)
        .spawn()?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    if let Ok(Some(status)) = child.try_wait() {
        if !status.success() {
            anyhow::bail!("Server {name} crashed immediately with status: {status}");
        }
    }

    println!("server {name} is healthy");

    let _ = child.kill();
    let _ = child.wait();

    Ok(())
}
