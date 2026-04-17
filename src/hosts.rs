use anyhow::{Context, Result};
use std::process::Command;

const HOSTS_FILE: &str = "/etc/hosts";

fn marker_begin(name: &str) -> String {
    format!("# BEGIN limavel[{}]", name)
}

fn marker_end(name: &str) -> String {
    format!("# END limavel[{}]", name)
}

/// Build the block of host entries to insert into /etc/hosts.
fn build_hosts_block(name: &str, ip: &str, domains: &[String]) -> String {
    let mut block = marker_begin(name);
    block.push('\n');
    for domain in domains {
        block.push_str(&format!("{} {}\n", ip, domain));
    }
    block.push_str(&marker_end(name));
    block
}

/// Read /etc/hosts, strip the block for the given instance name, return the cleaned content.
fn read_and_strip(name: &str) -> Result<String> {
    let content = std::fs::read_to_string(HOSTS_FILE)
        .with_context(|| format!("Failed to read {}", HOSTS_FILE))?;

    let begin = marker_begin(name);
    let end = marker_end(name);

    let mut result = String::new();
    let mut inside_block = false;

    for line in content.lines() {
        if line.trim() == begin {
            inside_block = true;
            continue;
        }
        if line.trim() == end {
            inside_block = false;
            continue;
        }
        if !inside_block {
            result.push_str(line);
            result.push('\n');
        }
    }

    Ok(result)
}

/// Write content to /etc/hosts via sudo tee.
fn sudo_write(content: &str) -> Result<()> {
    let mut child = Command::new("sudo")
        .args(["tee", HOSTS_FILE])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .context("Failed to run sudo tee")?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(content.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Failed to write {}", HOSTS_FILE);
    }
    Ok(())
}

/// Add site domains pointing to the given IP in /etc/hosts for the named instance.
pub fn update(name: &str, ip: &str, domains: &[String]) -> Result<()> {
    if domains.is_empty() {
        return Ok(());
    }

    let mut cleaned = read_and_strip(name)?;
    let block = build_hosts_block(name, ip, domains);

    // Ensure a newline before our block
    if !cleaned.ends_with('\n') {
        cleaned.push('\n');
    }
    cleaned.push_str(&block);
    cleaned.push('\n');

    sudo_write(&cleaned)
}

/// Remove entries for the named instance from /etc/hosts.
pub fn remove(name: &str) -> Result<()> {
    let cleaned = read_and_strip(name)?;
    sudo_write(&cleaned)
}

/// Resolve the guest IP and update /etc/hosts for the given instance's sites.
/// Prints progress messages. No-ops if there are no site domains.
pub fn update_from_config(
    instance: &str,
    config: &crate::config::limavel_config::LimavelConfig,
) -> Result<()> {
    use colored::Colorize;

    let domains: Vec<String> = config.sites.iter().map(|s| s.map.clone()).collect();
    if domains.is_empty() {
        return Ok(());
    }

    let ip = crate::lima::client::LimaClient::guest_ip(instance)?;
    println!("{} Updating /etc/hosts ({})...", "→".cyan(), ip);
    update(instance, &ip, &domains)?;
    println!("{} /etc/hosts updated.", "✓".green());
    Ok(())
}
