use anyhow::Result;
use console::style;

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str, no_hosts: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    if !no_hosts {
        hosts::remove(instance)?;
    }

    println!("{} Stopping VM '{}'...", style("→").cyan(), instance);
    LimaClient::stop(instance)?;
    println!("{} VM '{}' stopped.", style("✓").green(), instance);

    Ok(())
}
