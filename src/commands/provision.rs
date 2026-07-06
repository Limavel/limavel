use anyhow::Result;
use console::style;

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;
use crate::ansible::runner;


pub fn execute(name: &str, no_edit: bool) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    if !no_edit && super::sync_vm_config(instance, &config)? {
        println!("{} Starting VM '{}'...", style("→").cyan(), instance);
        LimaClient::start(instance)?;
    }

    println!("{} Running provisioning...", style("→").cyan());
    runner::provision(instance, &config)?;
    println!("{} Provisioning complete!", style("✓").green());

    // Refresh /etc/hosts in case sites changed
    hosts::update_from_config(instance, &config)?;

    Ok(())
}
