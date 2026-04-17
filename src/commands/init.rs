use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::limavel_config::LimavelConfig;
use crate::error::LimavelError;

const TEMPLATE: &str = include_str!("../../templates/default.yaml");

pub fn execute(name: &str) -> Result<()> {
    let file = LimavelConfig::config_path(name);

    if LimavelConfig::exists(name) {
        return Err(LimavelError::ConfigAlreadyExists(file.display().to_string()).into());
    }

    // Write a template file first
    std::fs::write(&file, TEMPLATE)
        .with_context(|| format!("Failed to write config file: {}", file.display()))?;

    println!("{} {} created successfully!", "✓".green(), file.display());
    println!("Edit {} to customize your development environment.", file.display());
    println!("Then run {} to start the VM.", format!("limavel start {}", name).cyan());

    Ok(())
}
