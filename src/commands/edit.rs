use anyhow::Result;
use colored::Colorize;

use crate::config::lima_config::LimaConfig;
use crate::config::limavel_config::LimavelConfig;
use crate::error::LimavelError;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;
    let config = LimavelConfig::load(name)?;
    config.validate_folders()?;
    let instance = config.instance_name();

    if !LimaClient::instance_exists(instance)? {
        return Err(LimavelError::InstanceNotFound(instance.to_string()).into());
    }

    let was_running = LimaClient::instance_status(instance)? == "Running";
    if was_running {
        println!("{} Stopping VM '{}' to apply changes...", "→".cyan(), instance);
        LimaClient::stop(instance)?;
    }

    let ssh_pubkey = config.read_ssh_pubkey()?;
    let lima_config = LimaConfig::from_config(&config, &ssh_pubkey)?;
    let yaml = lima_config.to_yaml()?;

    println!("{} Applying changes...", "→".cyan());
    LimaClient::edit(instance, &yaml)?;
    println!("{} Changes applied.", "✓".green());

    if was_running {
        println!("{} Starting VM '{}'...", "→".cyan(), instance);
        LimaClient::start(instance)?;
        println!("{} VM '{}' started.", "✓".green(), instance);
    }

    Ok(())
}