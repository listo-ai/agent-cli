use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Built-in registries shipped with the binary.
const BUNDLED_REGISTRIES: &str = include_str!("../registries.yaml");

// ── Registry types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryEntry {
    pub name: String,
    pub url: String,
    #[serde(default = "default_skill_file")]
    pub skill_file: String,
    #[serde(default)]
    pub description: String,
}

fn default_skill_file() -> String {
    "SKILL.md".to_string()
}

#[derive(Debug, Deserialize)]
struct BundledFile {
    registries: Vec<RegistryEntry>,
}

// ── User config (~/.agent-cli/config.yaml) ────────────────────────────────────

#[derive(Debug, Default, Deserialize, Serialize)]
struct UserConfig {
    #[serde(default)]
    registries: Vec<RegistryEntry>,
}

// ── Store ─────────────────────────────────────────────────────────────────────

pub struct Store {
    root: PathBuf,
    user_config: UserConfig,
}

impl Store {
    pub fn open() -> Result<Self> {
        let root = dirs::home_dir()
            .context("Cannot determine home directory")?
            .join(".agent-cli");
        fs::create_dir_all(&root)?;

        let cfg_path = root.join("config.yaml");
        let user_config = if cfg_path.exists() {
            let body = fs::read_to_string(&cfg_path)?;
            serde_yml::from_str(&body).context("Invalid ~/.agent-cli/config.yaml")?
        } else {
            UserConfig::default()
        };

        Ok(Self { root, user_config })
    }

    fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    fn skill_dir(&self, name: &str) -> PathBuf {
        self.skills_dir().join(name)
    }

    fn config_path(&self) -> PathBuf {
        self.root.join("config.yaml")
    }

    fn save_config(&self) -> Result<()> {
        let body = serde_yml::to_string(&self.user_config)?;
        fs::write(self.config_path(), body)?;
        Ok(())
    }

    // ── all known registries (bundled + user-added) ───────────────────────────

    fn all_registries(&self) -> Result<Vec<RegistryEntry>> {
        let bundled: BundledFile = serde_yml::from_str(BUNDLED_REGISTRIES)
            .context("Built-in registries.yaml is invalid")?;

        let mut map: BTreeMap<String, RegistryEntry> = BTreeMap::new();
        for r in bundled.registries {
            map.insert(r.name.clone(), r);
        }
        // User entries override / extend bundled ones
        for r in &self.user_config.registries {
            map.insert(r.name.clone(), r.clone());
        }
        Ok(map.into_values().collect())
    }

    fn find_registry(&self, name: &str) -> Result<RegistryEntry> {
        self.all_registries()?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| anyhow!("Unknown skill '{name}'. Run `registry list` to see available skills."))
    }

    // ── ls ────────────────────────────────────────────────────────────────────

    pub fn ls(&self) -> Result<()> {
        let skills_dir = self.skills_dir();
        if !skills_dir.exists() {
            println!("No skills installed.");
            return Ok(());
        }
        let mut found = false;
        for entry in fs::read_dir(&skills_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let version = git_current_sha(&entry.path()).unwrap_or_else(|_| "unknown".into());
                println!("{name}  ({version})");
                found = true;
            }
        }
        if !found {
            println!("No skills installed.");
        }
        Ok(())
    }

    pub fn ls_remote(&self) -> Result<()> {
        let regs = self.all_registries()?;
        if regs.is_empty() {
            println!("No registries configured.");
            return Ok(());
        }
        let skills_dir = self.skills_dir();
        for r in &regs {
            let installed = if skills_dir.join(&r.name).exists() { "✓" } else { " " };
            println!("[{installed}] {}  {}", r.name, r.description);
            println!("      {}", r.url);
        }
        Ok(())
    }

    // ── install ───────────────────────────────────────────────────────────────

    pub fn install(&self, name: &str) -> Result<()> {
        let reg = self.find_registry(name)?;
        let dest = self.skill_dir(name);

        if dest.exists() {
            println!("'{name}' is already installed. Use `update {name}` to pull latest.");
            return Ok(());
        }

        fs::create_dir_all(&self.skills_dir())?;
        println!("Cloning {} …", reg.url);

        let status = Command::new("git")
            .args(["clone", "--depth=1", &reg.url, dest.to_str().unwrap()])
            .status()
            .context("git not found — please install git")?;

        if !status.success() {
            bail!("git clone failed for '{}'", reg.url);
        }

        let sha = git_current_sha(&dest).unwrap_or_else(|_| "unknown".into());
        println!("installed '{name}' @ {sha}");
        Ok(())
    }

    // ── update ────────────────────────────────────────────────────────────────

    pub fn update(&self, name: Option<&str>) -> Result<()> {
        let skills_dir = self.skills_dir();

        let targets: Vec<String> = if let Some(n) = name {
            let d = self.skill_dir(n);
            if !d.exists() {
                bail!("'{n}' is not installed.");
            }
            vec![n.to_string()]
        } else {
            if !skills_dir.exists() {
                println!("No skills installed.");
                return Ok(());
            }
            fs::read_dir(&skills_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        };

        if targets.is_empty() {
            println!("No skills installed.");
            return Ok(());
        }

        for target in targets {
            let dir = self.skill_dir(&target);
            print!("updating '{target}' … ");
            let status = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(&dir)
                .status()
                .context("git not found")?;
            if status.success() {
                let sha = git_current_sha(&dir).unwrap_or_else(|_| "unknown".into());
                println!("@ {sha}");
            } else {
                println!("FAILED");
            }
        }
        Ok(())
    }

    // ── remove ────────────────────────────────────────────────────────────────

    pub fn remove(&self, name: &str) -> Result<()> {
        let dir = self.skill_dir(name);
        if !dir.exists() {
            bail!("'{name}' is not installed.");
        }
        fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to remove {}", dir.display()))?;
        println!("removed '{name}'");
        Ok(())
    }

    // ── show / path ───────────────────────────────────────────────────────────

    pub fn show(&self, name: &str) -> Result<()> {
        let reg = self.find_registry(name)?;
        let skill_path = self.skill_dir(name).join(&reg.skill_file);
        if !skill_path.exists() {
            bail!("'{name}' is not installed. Run `install {name}` first.");
        }
        let content = fs::read_to_string(&skill_path)?;
        print!("{content}");
        Ok(())
    }

    pub fn path(&self, name: &str) -> Result<()> {
        let reg = self.find_registry(name)?;
        let skill_path = self.skill_dir(name).join(&reg.skill_file);
        if !skill_path.exists() {
            bail!("'{name}' is not installed. Run `install {name}` first.");
        }
        println!("{}", skill_path.display());
        Ok(())
    }

    // ── registry add / list / remove ──────────────────────────────────────────

    pub fn registry_list(&self) -> Result<()> {
        let regs = self.all_registries()?;
        for r in &regs {
            let tag = if self.user_config.registries.iter().any(|u| u.name == r.name) {
                " [custom]"
            } else {
                " [built-in]"
            };
            println!("{}{tag}  {}  {}", r.name, r.url, r.description);
        }
        Ok(())
    }

    pub fn registry_add(&mut self, name: String, url: String, skill_file: Option<String>, description: Option<String>) -> Result<()> {
        if self.user_config.registries.iter().any(|r| r.name == name) {
            bail!("Registry '{name}' already exists in your config. Remove it first.");
        }
        self.user_config.registries.push(RegistryEntry {
            name: name.clone(),
            url: url.clone(),
            skill_file: skill_file.unwrap_or_else(default_skill_file),
            description: description.unwrap_or_default(),
        });
        self.save_config()?;
        println!("added '{name}' → {url}");
        Ok(())
    }

    pub fn registry_remove(&mut self, name: &str) -> Result<()> {
        let before = self.user_config.registries.len();
        self.user_config.registries.retain(|r| r.name != name);
        if self.user_config.registries.len() == before {
            // Check if it was bundled
            let bundled: BundledFile = serde_yml::from_str(BUNDLED_REGISTRIES).unwrap();
            if bundled.registries.iter().any(|r| r.name == name) {
                bail!("'{name}' is a built-in registry and cannot be removed.\nTo override it, use `registry add {name} <new-url>`.");
            }
            bail!("'{name}' not found in your config.");
        }
        self.save_config()?;
        println!("removed '{name}' from config");
        Ok(())
    }
}

// ── git helpers ───────────────────────────────────────────────────────────────

fn git_current_sha(dir: &PathBuf) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(dir)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
