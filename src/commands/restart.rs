use anyhow::Result;
use console::style;

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    println!("{} Restarting VM '{}'...", style("→").cyan(), instance);
    LimaClient::restart(instance)?;
    println!("{} VM '{}' restarted.", style("✓").green(), instance);

    // Refresh /etc/hosts since IP may change after reboot
    hosts::update_from_config(instance, &config)?;

    Ok(())
}
