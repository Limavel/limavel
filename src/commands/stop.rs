use anyhow::Result;
use colored::Colorize;

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str, no_hosts: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    if !no_hosts {
        println!("{} Removing /etc/hosts entries for '{}'...", "→".cyan(), instance);
        hosts::remove(instance)?;
    }

    println!("{} Stopping VM '{}'...", "→".cyan(), instance);
    LimaClient::stop(instance)?;
    println!("{} VM '{}' stopped.", "✓".green(), instance);

    Ok(())
}
