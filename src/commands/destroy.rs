use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use crate::config::limavel_config::LimavelConfig;
use crate::hosts;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();

    if !LimaClient::instance_exists(instance)? {
        println!("{} No VM instance '{}' found.", "ℹ".cyan(), instance);
        return Ok(());
    }

    // Confirm destruction
    print!(
        "{} Are you sure you want to destroy VM '{}'? This cannot be undone. [y/N] ",
        "⚠".yellow(),
        instance
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted.");
        return Ok(());
    }

    // Clean up /etc/hosts entries
    println!("{} Removing /etc/hosts entries for '{}'...", "→".cyan(), instance);
    hosts::remove(instance)?;

    // Stop if running
    let status = LimaClient::instance_status(instance)?;
    if status == "Running" {
        println!("{} Stopping VM '{}'...", "→".cyan(), instance);
        LimaClient::stop(instance)?;
    }

    println!("{} Destroying VM '{}'...", "→".cyan(), instance);
    LimaClient::delete(instance)?;

    println!("{} VM '{}' destroyed.", "✓".green(), instance);
    Ok(())
}
