use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

pub const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/NubeDev/agent-cli/master/skills-registry.yaml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillsRegistry {
    pub skills: Vec<SkillEntry>,
}

pub fn build_client() -> Result<Client> {
    Client::builder()
        .user_agent(format!("agent-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to build HTTP client")
}

pub async fn fetch_registry(client: &Client, registry_url: &str) -> Result<SkillsRegistry> {
    let response = client
        .get(registry_url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch registry from {registry_url}"))?
        .error_for_status()
        .with_context(|| format!("Registry request failed for {registry_url}"))?;

    let body = response
        .text()
        .await
        .context("Failed to read registry response body")?;

    serde_yml::from_str(&body).context("Registry YAML is invalid")
}

pub async fn fetch_skill_markdown(
    client: &Client,
    registry_url: &str,
    entry: &SkillEntry,
) -> Result<String> {
    let base_url = Url::parse(registry_url).context("Registry URL is invalid")?;
    let skill_url = base_url
        .join(&entry.path)
        .with_context(|| format!("Could not resolve skill path {}", entry.path))?;

    let response = client
        .get(skill_url.clone())
        .send()
        .await
        .with_context(|| format!("Failed to fetch skill {} from {}", entry.name, skill_url))?
        .error_for_status()
        .with_context(|| format!("Skill request failed for {}", skill_url))?;

    response
        .text()
        .await
        .with_context(|| format!("Failed to read markdown for {}", entry.name))
}
