use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::LimavelError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimavelConfig {
    pub name: String,
    pub memory: u32,
    pub cpus: u32,
    #[serde(default = "default_disk")]
    pub disk: u32,
    pub image: String,
    pub arch: String,
    pub authorize: String,
    pub keys: Vec<String>,
    pub folders: Vec<FolderMap>,
    pub sites: Vec<SiteMap>,
    pub databases: Vec<String>,
    pub database: DatabaseConfig,
    pub features: Features,
    pub ports: Vec<PortMap>,
    #[serde(default = "default_nodejs")]
    pub nodejs: String,
    #[serde(default)]
    pub bootstrap: Option<String>,
    #[serde(default)]
    pub playbooks: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderMap {
    pub map: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteMap {
    pub map: String,
    pub to: String,
    pub php: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: String,
    pub version: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Features {
    #[serde(default)]
    pub ohmyzsh: bool,
    #[serde(default)]
    pub webdriver: bool,
    #[serde(default)]
    pub mailpit: bool,
    #[serde(default)]
    pub mongodb: bool,
    #[serde(default)]
    pub valkey: bool,
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMap {
    pub send: u16,
    pub to: u16,
}

fn default_disk() -> u32 {
    50
}

fn default_nodejs() -> String {
    "24".to_string()
}

impl LimavelConfig {
    /// Returns the config file path for a given instance name: `{name}.yaml`
    pub fn config_path(name: &str) -> PathBuf {
        PathBuf::from(format!("{}.yaml", name))
    }

    pub fn exists(name: &str) -> bool {
        Self::config_path(name).exists()
    }

    pub fn load(name: &str) -> Result<Self> {
        let path = Self::config_path(name);
        if !path.exists() {
            return Err(LimavelError::ConfigNotFound(path.display().to_string()).into());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: LimavelConfig =
            serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(config)
    }

    pub fn resolve_path(path: &str) -> String {
        shellexpand::tilde(path).to_string()
    }

    pub fn read_ssh_pubkey(&self) -> Result<String> {
        let path = Self::resolve_path(&self.authorize);
        let p = Path::new(&path);
        if !p.exists() {
            return Err(LimavelError::SshKeyNotFound(self.authorize.clone()).into());
        }
        let content = fs::read_to_string(p)
            .with_context(|| format!("Failed to read SSH key: {}", path))?;
        Ok(content.trim().to_string())
    }

    pub fn validate_folders(&self) -> Result<()> {
        let missing: Vec<String> = self
            .folders
            .iter()
            .filter_map(|f| {
                let expanded = shellexpand::tilde(&f.map).to_string();
                let path = Path::new(&expanded);
                if !path.exists() {
                    Some(format!("  - {} ({})", f.map, expanded))
                } else {
                    None
                }
            })
            .collect();

        if !missing.is_empty() {
            return Err(LimavelError::FoldersNotFound(missing.join("\n")).into());
        }

        Ok(())
    }

    /// Returns the Lima VM instance name from the `name` field.
    pub fn instance_name(&self) -> &str {
        &self.name
    }

    pub fn php_versions(&self) -> Vec<String> {
        let mut versions: Vec<String> = self.sites.iter().map(|s| s.php.clone()).collect();
        versions.sort();
        versions.dedup();
        versions
    }
}
