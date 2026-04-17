use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

pub fn execute(path: &str) -> Result<()> {
    let target = Path::new(path);

    if !target.is_dir() {
        anyhow::bail!("Target directory does not exist: {}", target.display());
    }

    let bootstrap_dir = target.join("bootstrap");
    std::fs::create_dir_all(&bootstrap_dir)
        .with_context(|| "Failed to create bootstrap directory")?;

    println!("{} Extracting bootstrap files...", "→".cyan());
    crate::bootstrap::write_all(&bootstrap_dir)
        .with_context(|| "Failed to extract bootstrap files")?;

    let ansible_dir = target.join("ansible");
    std::fs::create_dir_all(&ansible_dir)
        .with_context(|| "Failed to create ansible directory")?;

    println!("{} Extracting ansible files...", "→".cyan());
    crate::ansible::playbooks::write_all(&ansible_dir)
        .with_context(|| "Failed to extract ansible files")?;

    println!("{} Published to {}", "✓".green(), target.display());

    Ok(())
}
