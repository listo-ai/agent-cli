use crate::registry::{build_client, fetch_registry, fetch_skill_markdown, SkillEntry};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const INSTALL_DIR: &str = "docs/agents";
const LOCK_FILE: &str = "docs/agents/.skills.lock";

#[derive(Debug, Default, Deserialize, Serialize)]
struct SkillsLock {
    #[serde(default)]
    skills: BTreeMap<String, InstalledSkill>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InstalledSkill {
    version: String,
    source: String,
    path: String,
}

pub async fn list_available(registry_url: &str) -> Result<()> {
    let client = build_client()?;
    let registry = fetch_registry(&client, registry_url).await?;

    for skill in registry.skills {
        println!("{} {} - {}", skill.name, skill.version, skill.description);
    }

    Ok(())
}

pub async fn add(names: &[String], registry_url: &str) -> Result<()> {
    if names.is_empty() {
        bail!("Provide at least one skill name.");
    }

    let client = build_client()?;
    let registry = fetch_registry(&client, registry_url).await?;
    let mut lock = load_lock()?;

    for name in names {
        let entry = registry
            .skills
            .iter()
            .find(|skill| skill.name == *name)
            .ok_or_else(|| anyhow!("Skill `{name}` is not in the registry"))?;

        install_entry(&client, registry_url, entry, &mut lock).await?;
    }

    write_lock(&lock)?;
    Ok(())
}

pub async fn update(name: Option<&str>, registry_url: &str) -> Result<()> {
    let client = build_client()?;
    let registry = fetch_registry(&client, registry_url).await?;
    let mut lock = load_lock()?;

    if lock.skills.is_empty() {
        bail!("No installed skills found in {INSTALL_DIR}.");
    }

    let targets: Vec<String> = match name {
        Some(name) => {
            if !lock.skills.contains_key(name) {
                bail!("Skill `{name}` is not installed.");
            }
            vec![name.to_string()]
        }
        None => lock.skills.keys().cloned().collect(),
    };

    for target in targets {
        let entry = registry
            .skills
            .iter()
            .find(|skill| skill.name == target)
            .ok_or_else(|| anyhow!("Installed skill `{target}` is missing from the registry"))?;

        install_entry(&client, registry_url, entry, &mut lock).await?;
    }

    write_lock(&lock)?;
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let mut lock = load_lock()?;
    let file_path = lock
        .skills
        .get(name)
        .map(|entry| PathBuf::from(&entry.path))
        .unwrap_or_else(|| install_path(name, None));

    if file_path.exists() {
        fs::remove_file(&file_path)
            .with_context(|| format!("Failed to remove {}", file_path.display()))?;
        println!("removed {}", file_path.display());
        remove_empty_skill_dirs(file_path.parent())?;
    } else if !lock.skills.contains_key(name) {
        bail!("Skill `{name}` is not installed.");
    }

    lock.skills.remove(name);
    write_lock(&lock)?;
    Ok(())
}

pub fn list_installed() -> Result<()> {
    let lock = load_lock()?;
    let install_dir = PathBuf::from(INSTALL_DIR);
    let mut installed_files = BTreeSet::new();

    if install_dir.exists() {
        collect_installed_skill_names(&install_dir, &mut installed_files)?;
    }

    let names: BTreeSet<String> = installed_files
        .into_iter()
        .chain(lock.skills.keys().cloned())
        .collect();

    if names.is_empty() {
        println!("No skills installed in {INSTALL_DIR}.");
        return Ok(());
    }

    for name in names {
        match lock.skills.get(&name) {
            Some(entry) => println!("{} {} -> {}", name, entry.version, entry.path),
            None => println!("{} unknown -> {}/{}.md", name, INSTALL_DIR, name),
        }
    }

    Ok(())
}

async fn install_entry(
    client: &reqwest::Client,
    registry_url: &str,
    entry: &SkillEntry,
    lock: &mut SkillsLock,
) -> Result<()> {
    let markdown = fetch_skill_markdown(client, registry_url, entry).await?;
    ensure_install_dir()?;

    let target = install_path(&entry.name, Some(&entry.path));
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let legacy_target = install_path(&entry.name, None);
    if legacy_target != target && legacy_target.exists() {
        fs::remove_file(&legacy_target)
            .with_context(|| format!("Failed to remove legacy {}", legacy_target.display()))?;
    }

    let status = match fs::read_to_string(&target) {
        Ok(existing) if existing == markdown => "unchanged",
        Ok(_) => "updated",
        Err(_) => "installed",
    };

    fs::write(&target, markdown)
        .with_context(|| format!("Failed to write {}", target.display()))?;

    lock.skills.insert(
        entry.name.clone(),
        InstalledSkill {
            version: entry.version.clone(),
            source: entry.path.clone(),
            path: target.display().to_string(),
        },
    );

    println!("{status} {}", target.display());
    Ok(())
}

fn ensure_install_dir() -> Result<()> {
    fs::create_dir_all(INSTALL_DIR).with_context(|| format!("Failed to create {INSTALL_DIR}"))
}

fn install_path(name: &str, source_path: Option<&str>) -> PathBuf {
    if let Some(source_path) = source_path {
        if let Some(relative_path) = source_path.strip_prefix("skills/") {
            return Path::new(INSTALL_DIR).join(relative_path);
        }
    }

    Path::new(INSTALL_DIR).join(format!("{name}.md"))
}

fn load_lock() -> Result<SkillsLock> {
    let lock_path = PathBuf::from(LOCK_FILE);
    if !lock_path.exists() {
        return Ok(SkillsLock::default());
    }

    let content = fs::read_to_string(&lock_path)
        .with_context(|| format!("Failed to read {}", lock_path.display()))?;
    serde_yml::from_str(&content).context("Failed to parse .skills.lock")
}

fn write_lock(lock: &SkillsLock) -> Result<()> {
    ensure_install_dir()?;
    let lock_path = PathBuf::from(LOCK_FILE);

    if lock.skills.is_empty() {
        if lock_path.exists() {
            fs::remove_file(&lock_path)
                .with_context(|| format!("Failed to remove {}", lock_path.display()))?;
        }
        return Ok(());
    }

    let body = serde_yml::to_string(lock).context("Failed to serialize .skills.lock")?;
    fs::write(&lock_path, body).with_context(|| format!("Failed to write {}", lock_path.display()))
}

fn collect_installed_skill_names(dir: &Path, names: &mut BTreeSet<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();

        if path.file_name().and_then(|name| name.to_str()) == Some(".skills.lock") {
            continue;
        }

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                names.insert(name.to_string());
            }
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                names.insert(stem.to_string());
            }
        }
    }

    Ok(())
}

fn remove_empty_skill_dirs(mut dir: Option<&Path>) -> Result<()> {
    while let Some(path) = dir {
        if path == Path::new(INSTALL_DIR) {
            break;
        }

        match fs::remove_dir(path) {
            Ok(()) => dir = path.parent(),
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                dir = path.parent();
            }
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to remove {}", path.display()));
            }
        }
    }

    Ok(())
}
