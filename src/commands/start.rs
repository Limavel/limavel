use anyhow::Result;
use console::style;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::ansible::runner;
use crate::config::lima_config::LimaConfig;
use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str, no_hosts: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    config.validate_folders()?;
    let instance = config.instance_name();

    if LimaClient::instance_exists(instance)? {
        let status = LimaClient::instance_status(instance)?;
        if status == "Running" {
            println!("{} VM '{}' is already running.", style("ℹ").cyan(), instance);
            return Ok(());
        }

        super::sync_vm_config(instance, &config)?;

        println!("{} Starting VM '{}'...", style("→").cyan(), instance);
        LimaClient::start(instance)?;
        println!("{} VM '{}' started.", style("✓").green(), instance);
        if !no_hosts {
            hosts::update_from_config(instance, &config)?;
        }
        return Ok(());
    }

    // Create new instance
    println!("{} Creating VM '{}'...", style("→").cyan(), instance);

    let ssh_pubkey = config.read_ssh_pubkey()?;
    let lima_config = LimaConfig::from_config(&config, &ssh_pubkey)?;
    let yaml = lima_config.to_yaml()?;

    let mut tmpfile = NamedTempFile::with_suffix(".yaml")?;
    tmpfile.write_all(yaml.as_bytes())?;
    tmpfile.flush()?;

    LimaClient::create(instance, tmpfile.path().to_str().unwrap())?;
    println!("{} VM '{}' created.", style("✓").green(), instance);

    println!("{} Starting VM '{}'...", style("→").cyan(), instance);
    LimaClient::start(instance)?;
    println!("{} VM '{}' started.", style("✓").green(), instance);

    // Run initial provisioning
    println!("{} Running initial provisioning...", style("→").cyan());
    runner::provision(instance, &config)?;
    println!("{} Provisioning complete!", style("✓").green());

    if !no_hosts {
        hosts::update_from_config(instance, &config)?;
    }

    println!("Type \"{}\" for accessing to your development environment", style("limavel ssh").green());

    Ok(())
}
