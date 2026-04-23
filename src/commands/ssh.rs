use anyhow::Result;
use std::process::{Command, Stdio};

use crate::config::limavel_config::LimavelConfig;
use crate::lima::client::LimaClient;

pub fn execute(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    let status = Command::new("limactl")
        .args(["shell", "--workdir", "/tmp", instance, "--", "sudo", "-iu", "limavel"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to execute limactl shell: {}", e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("SSH session exited with error"));
    }

    Ok(())
}

pub fn details(name: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    let ssh_config_path = format!("{}/.lima/{}/ssh.config", home, instance);

    let content = std::fs::read_to_string(&ssh_config_path)
        .map_err(|_| anyhow::anyhow!("SSH config not found at {}", ssh_config_path))?;

    let mut user = None;
    let mut hostname = None;
    let mut port = None;
    let mut identity_file = None;

    for line in content.lines() {
        let trimmed = line.trim();
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let value = match parts.next() {
            Some(v) => v.trim().trim_matches('"'),
            None => continue,
        };

        match key {
            "User" if user.is_none() => user = Some(value.to_string()),
            "Hostname" if hostname.is_none() => hostname = Some(value.to_string()),
            "Port" if port.is_none() => port = Some(value.to_string()),
            "IdentityFile" if identity_file.is_none() => identity_file = Some(value.to_string()),
            _ => {}
        }
    }

    let user = user.ok_or_else(|| anyhow::anyhow!("User not found in SSH config"))?;
    let hostname = hostname.ok_or_else(|| anyhow::anyhow!("Hostname not found in SSH config"))?;
    let port = port.ok_or_else(|| anyhow::anyhow!("Port not found in SSH config"))?;
    let identity_file =
        identity_file.ok_or_else(|| anyhow::anyhow!("IdentityFile not found in SSH config"))?;

    println!();
    println!("SSH connection details for \"{}\":", instance);
    println!();
    println!("  Host:         {}", hostname);
    println!("  Port:         {}", port);
    println!("  User:         {}", user);
    println!("  Private key:  {}", identity_file);
    println!();
    println!(
        "  ssh -i {} -p {} {}@{}",
        identity_file, port, user, hostname
    );
    println!("    or");
    println!(
        "  ssh -F {} {}",
        ssh_config_path, instance
    );
    println!();
    println!("No password is required (certificate-based authentication).");
    println!();

    Ok(())
}
