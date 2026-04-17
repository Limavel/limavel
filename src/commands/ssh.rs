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
