use anyhow::{Context, Result};
use console::style;
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

fn read_hosts() -> Result<String> {
    std::fs::read_to_string(HOSTS_FILE).with_context(|| format!("Failed to read {}", HOSTS_FILE))
}

/// Strip the block for the given instance name, returning the cleaned content.
/// Errors on a BEGIN marker without a matching END marker rather than
/// discarding everything up to the end of the file.
fn strip_block(content: &str, name: &str) -> Result<String> {
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

    if inside_block {
        anyhow::bail!(
            "{} contains '{}' without a matching '{}'. Fix the file manually and retry.",
            HOSTS_FILE,
            begin,
            end
        );
    }

    // Preserve a missing trailing newline so unchanged content compares equal.
    if !content.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}

/// Write content to /etc/hosts via sudo: stream it to a temp file on the same
/// filesystem, then rename into place so the file is never left truncated if
/// the write is interrupted.
fn sudo_write(content: &str) -> Result<()> {
    let script = format!(
        r#"tmp=$(mktemp {hosts}.XXXXXX) && cat > "$tmp" && chmod 644 "$tmp" && mv "$tmp" {hosts}"#,
        hosts = HOSTS_FILE
    );
    let mut child = Command::new("sudo")
        .args(["sh", "-c", &script])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .context("Failed to run sudo")?;

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

/// Add site domains pointing to the given IP in /etc/hosts for the named
/// instance. No-ops (and skips the sudo prompt) if the entries are already
/// up to date.
pub fn update(name: &str, ip: &str, domains: &[String]) -> Result<()> {
    if domains.is_empty() {
        return Ok(());
    }

    let content = read_hosts()?;
    let mut updated = strip_block(&content, name)?;

    // Ensure a newline before our block
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&build_hosts_block(name, ip, domains));
    updated.push('\n');

    if updated == content {
        return Ok(());
    }

    println!("{} Updating /etc/hosts ({})...", style("→").cyan(), ip);
    sudo_write(&updated)?;
    println!("{} /etc/hosts updated.", style("✓").green());
    Ok(())
}

/// Remove entries for the named instance from /etc/hosts. No-ops (and skips
/// the sudo prompt) if there is nothing to remove.
pub fn remove(name: &str) -> Result<()> {
    let content = read_hosts()?;
    let cleaned = strip_block(&content, name)?;

    if cleaned == content {
        return Ok(());
    }

    println!("{} Removing /etc/hosts entries for '{}'...", style("→").cyan(), name);
    sudo_write(&cleaned)?;
    println!("{} /etc/hosts entries removed.", style("✓").green());
    Ok(())
}

/// Resolve the guest IP and update /etc/hosts for the given instance's sites.
/// Prints progress messages. No-ops if there are no site domains.
pub fn update_from_config(
    instance: &str,
    config: &crate::config::limavel_config::LimavelConfig,
) -> Result<()> {
    let domains: Vec<String> = config.sites.iter().map(|s| s.map.clone()).collect();
    if domains.is_empty() {
        return Ok(());
    }

    let ip = crate::lima::client::LimaClient::guest_ip(instance)?;
    update(instance, &ip, &domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OTHER_ENTRIES: &str = "127.0.0.1 localhost\n255.255.255.255 broadcasthost\n";

    #[test]
    fn build_block_wraps_entries_in_markers() {
        let block = build_hosts_block("dev", "192.168.5.2", &["a.test".into(), "b.test".into()]);
        assert_eq!(
            block,
            "# BEGIN limavel[dev]\n192.168.5.2 a.test\n192.168.5.2 b.test\n# END limavel[dev]"
        );
    }

    #[test]
    fn strip_removes_only_the_named_block() {
        let content = format!(
            "{}{}\n{}\n",
            OTHER_ENTRIES,
            build_hosts_block("dev", "1.2.3.4", &["a.test".into()]),
            build_hosts_block("other", "1.2.3.5", &["b.test".into()])
        );
        let cleaned = strip_block(&content, "dev").unwrap();
        assert!(!cleaned.contains("a.test"));
        assert!(!cleaned.contains("limavel[dev]"));
        assert!(cleaned.contains("b.test"));
        assert!(cleaned.contains("localhost"));
    }

    #[test]
    fn strip_is_identity_when_no_block_present() {
        assert_eq!(strip_block(OTHER_ENTRIES, "dev").unwrap(), OTHER_ENTRIES);
    }

    #[test]
    fn strip_preserves_missing_trailing_newline() {
        let content = "127.0.0.1 localhost";
        assert_eq!(strip_block(content, "dev").unwrap(), content);
    }

    #[test]
    fn strip_handles_empty_content() {
        assert_eq!(strip_block("", "dev").unwrap(), "");
    }

    #[test]
    fn strip_errors_on_begin_without_end() {
        let content = format!("{}# BEGIN limavel[dev]\n1.2.3.4 a.test\n", OTHER_ENTRIES);
        let err = strip_block(&content, "dev").unwrap_err();
        assert!(err.to_string().contains("without a matching"));
    }

    #[test]
    fn strip_then_append_round_trips() {
        // Mirrors update(): unchanged entries must compare equal so the
        // sudo write is skipped.
        let block = build_hosts_block("dev", "1.2.3.4", &["a.test".into()]);
        let content = format!("{}{}\n", OTHER_ENTRIES, block);
        let mut rebuilt = strip_block(&content, "dev").unwrap();
        rebuilt.push_str(&block);
        rebuilt.push('\n');
        assert_eq!(rebuilt, content);
    }
}