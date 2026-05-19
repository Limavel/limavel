use anyhow::Result;
use console::style;
use dialoguer::Confirm;

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();

    if !LimaClient::instance_exists(instance)? {
        println!("{} No VM instance '{}' found.", style("ℹ").cyan(), instance);
        return Ok(());
    }

    // Confirm destruction
    let confirmed = Confirm::new()
        .with_prompt(format!(
            "{} Are you sure you want to destroy VM '{}'? This cannot be undone",
            style("⚠").yellow(),
            instance
        ))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("Aborted.");
        return Ok(());
    }

    // Clean up /etc/hosts entries
    println!("{} Removing /etc/hosts entries for '{}'...", style("→").cyan(), instance);
    hosts::remove(instance)?;

    // Stop if running
    let status = LimaClient::instance_status(instance)?;
    if status == "Running" {
        println!("{} Stopping VM '{}'...", style("→").cyan(), instance);
        LimaClient::stop(instance)?;
    }

    println!("{} Destroying VM '{}'...", style("→").cyan(), instance);
    LimaClient::delete(instance)?;

    println!("{} VM '{}' destroyed.", style("✓").green(), instance);
    Ok(())
}
