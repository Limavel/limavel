use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::Path;
use tempfile::TempDir;

use crate::config::limavel_config::{Features, LimavelConfig};
use crate::lima::client::LimaClient;

use super::playbooks;

#[derive(Serialize)]
struct AnsibleVars {
    instance: String,
    ssh_public_key: String,
    sites: Vec<AnsibleSite>,
    php_versions: Vec<String>,
    db_type: String,
    db_version: String,
    db_password: String,
    databases: Vec<String>,
    features: Features,
    nodejs_version: String,
}

#[derive(Serialize)]
struct AnsibleSite {
    domain: String,
    root: String,
    php_version: String,
}

fn ensure_ansible_installed(name: &str) -> Result<()> {
    let check = LimaClient::shell(name, "which ansible-playbook");
    if check.is_err() {
        println!("Ansible not found in guest, installing...");
        LimaClient::shell(name, "sudo apt-get update -qq && sudo apt-get install -y -qq ansible")
            .context("Failed to install ansible in guest")?;
    }
    Ok(())
}

pub fn provision(name: &str, config: &LimavelConfig) -> Result<()> {
    let ssh_pubkey = config
        .read_ssh_pubkey()
        .context("Failed to read SSH public key")?;

    let vars = AnsibleVars {
        instance: config.name.clone(),
        ssh_public_key: ssh_pubkey,
        sites: config
            .sites
            .iter()
            .map(|s| AnsibleSite {
                domain: s.map.clone(),
                root: s.to.clone(),
                php_version: s.php.clone(),
            })
            .collect(),
        php_versions: config.php_versions(),
        db_type: config.database.db_type.clone(),
        db_version: config.database.version.clone(),
        db_password: config.database.password.clone(),
        databases: config.databases.clone(),
        features: config.features.clone(),
        nodejs_version: config.nodejs.clone(),
    };

    let vars_yaml = serde_yml::to_string(&vars).context("Failed to serialize ansible vars")?;

    // Write playbooks to temp directory
    let tmpdir = TempDir::new().context("Failed to create temp directory")?;
    let ansible_dir = tmpdir.path();

    if let Some(ref playbooks_path) = config.playbooks {
        let expanded = shellexpand::tilde(playbooks_path).to_string();
        let src = Path::new(&expanded);
        if !src.is_dir() {
            bail!(
                "Custom playbooks path '{}' does not exist or is not a directory.",
                expanded
            );
        }
        if !src.join("playbook.yml").exists() {
            bail!(
                "Custom playbooks path '{}' does not contain a playbook.yml file.",
                expanded
            );
        }
        copy_dir_recursive(src, ansible_dir)
            .with_context(|| format!("Failed to copy custom playbooks from '{}'", expanded))?;
    } else {
        playbooks::write_all(ansible_dir)?;
    }

    // Write vars file
    std::fs::write(ansible_dir.join("vars.yml"), &vars_yaml)?;

    // Ensure ansible is installed in the guest
    ensure_ansible_installed(name)?;

    // Copy playbooks to guest via tar pipe (avoids limactl copy directory nesting issues)
    LimaClient::shell(name, "sudo mkdir -p /opt/limavel/ansible")?;
    LimaClient::tar_to_guest(name, ansible_dir, "/opt/limavel/ansible")?;

    // Run ansible-playbook inside guest
    LimaClient::shell_interactive(
        name,
        "sudo ansible-playbook -c local -i 'localhost,' /opt/limavel/ansible/playbook.yml -e @/opt/limavel/ansible/vars.yml",
    )?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
