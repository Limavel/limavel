use anyhow::Result;
use console::style;

use crate::config::lima_config::LimaConfig;
use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;
use crate::ansible::runner;


pub fn execute(name: &str, no_edit: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    if !no_edit {
        apply_vm_changes(instance, &config)?;
    }

    println!("{} Running provisioning...", style("→").cyan());
    runner::provision(instance, &config)?;
    println!("{} Provisioning complete!", style("✓").green());

    // Refresh /etc/hosts in case sites changed
    hosts::update_from_config(instance, &config)?;

    Ok(())
}

fn folders_changed(instance: &str, config: &LimavelConfig) -> Result<bool> {
    let current_mounts = LimaClient::instance_mounts(instance)?;
    let desired: Vec<(String, Option<String>)> = config
        .folders
        .iter()
        .map(|f| {
            let expanded = shellexpand::tilde(&f.map).to_string();
            (expanded, Some(f.to.clone()))
        })
        .collect();

    // Filter current mounts to only user-defined ones (those with a mount_point)
    let current_user_mounts: Vec<&(String, Option<String>)> = current_mounts
        .iter()
        .filter(|(_, mp)| mp.is_some())
        .collect();

    if desired.len() != current_user_mounts.len() {
        return Ok(true);
    }

    for (loc, mp) in &desired {
        if !current_user_mounts.iter().any(|(cl, cm)| cl == loc && cm == mp) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn apply_vm_changes(instance: &str, config: &LimavelConfig) -> Result<()> {
    let current_cpus = LimaClient::instance_cpus(instance)?;
    let current_memory = LimaClient::instance_memory_mib(instance)?;
    let current_disk = LimaClient::instance_disk_gib(instance)?;

    let cpus_changed = config.cpus != current_cpus;
    let memory_changed = config.memory != current_memory;
    let disk_changed = config.disk > current_disk;
    let mounts_changed = folders_changed(instance, config)?;

    if !cpus_changed && !memory_changed && !disk_changed && !mounts_changed {
        return Ok(());
    }

    let mut changes = Vec::new();
    if cpus_changed {
        changes.push(format!("cpus: {} -> {}", current_cpus, config.cpus));
    }
    if memory_changed {
        changes.push(format!("memory: {}MiB -> {}MiB", current_memory, config.memory));
    }
    if disk_changed {
        changes.push(format!("disk: {}GiB -> {}GiB", current_disk, config.disk));
    }
    if mounts_changed {
        changes.push("shared folders".to_string());
    }
    println!("{} Applying VM changes: {}", style("→").cyan(), changes.join(", "));

    println!("{} Stopping VM '{}' to apply changes...", style("→").cyan(), instance);
    LimaClient::stop(instance)?;

    let ssh_pubkey = config.read_ssh_pubkey()?;
    let lima_config = LimaConfig::from_config(config, &ssh_pubkey)?;
    let yaml = lima_config.to_yaml()?;
    LimaClient::edit(instance, &yaml)?;

    println!("{} Starting VM '{}'...", style("→").cyan(), instance);
    LimaClient::start(instance)?;
    println!("{} VM changes applied.", style("✓").green());

    Ok(())
}
