use anyhow::Result;
use colored::Colorize;
use tempfile::NamedTempFile;
use std::io::Write;

use crate::config::limavel_config::LimavelConfig;
use crate::config::lima_config::LimaConfig;
use crate::hosts;
use crate::lima::client::LimaClient;
use crate::ansible::runner;

fn apply_resource_changes(instance: &str, config: &LimavelConfig) -> Result<()> {
    let current_cpus = LimaClient::instance_cpus(instance)?;
    let current_memory = LimaClient::instance_memory_mib(instance)?;
    let current_disk = LimaClient::instance_disk_gib(instance)?;

    let cpus_changed = config.cpus != current_cpus;
    let memory_changed = config.memory != current_memory;
    let disk_changed = config.disk > current_disk;

    if cpus_changed || memory_changed || disk_changed {
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
        println!("{} Applying resource changes: {}", "→".cyan(), changes.join(", "));

        let new_disk = if disk_changed { Some(config.disk) } else { None };
        LimaClient::edit(instance, config.cpus, config.memory, new_disk)?;

        println!("{} Resource changes applied.", "✓".green());
    }

    Ok(())
}

pub fn execute(name: &str, no_hosts: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    config.validate_folders()?;
    let instance = config.instance_name();

    if LimaClient::instance_exists(instance)? {
        let status = LimaClient::instance_status(instance)?;
        if status == "Running" {
            println!("{} VM '{}' is already running.", "ℹ".cyan(), instance);
            return Ok(());
        }

        apply_resource_changes(instance, &config)?;

        println!("{} Starting VM '{}'...", "→".cyan(), instance);
        LimaClient::start(instance)?;
        println!("{} VM '{}' started.", "✓".green(), instance);
        if !no_hosts {
            hosts::update_from_config(instance, &config)?;
        }
        return Ok(());
    }

    // Create new instance
    println!("{} Creating VM '{}'...", "→".cyan(), instance);

    let ssh_pubkey = config.read_ssh_pubkey()?;
    let lima_config = LimaConfig::from_config(&config, &ssh_pubkey)?;
    let yaml = lima_config.to_yaml()?;

    let mut tmpfile = NamedTempFile::with_suffix(".yaml")?;
    tmpfile.write_all(yaml.as_bytes())?;
    tmpfile.flush()?;

    LimaClient::create(instance, tmpfile.path().to_str().unwrap())?;
    println!("{} VM '{}' created.", "✓".green(), instance);

    println!("{} Starting VM '{}'...", "→".cyan(), instance);
    LimaClient::start(instance)?;
    println!("{} VM '{}' started.", "✓".green(), instance);

    // Run initial provisioning
    println!("{} Running initial provisioning...", "→".cyan());
    runner::provision(instance, &config)?;
    println!("{} Provisioning complete!", "✓".green());

    if !no_hosts {
        hosts::update_from_config(instance, &config)?;
    }

    println!("Type \"{}\" for accessing to your development environment", "limavel ssh".green());

    Ok(())
}
