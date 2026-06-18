use anyhow::Result;
use std::process::{Command, Stdio};

use crate::config::limavel_config::LimavelConfig;
use crate::lima::client::LimaClient;

pub fn execute(name: &str, cmd: &str) -> Result<()> {
    LimaClient::check_installed()?;

    let config = LimavelConfig::load(name)?;
    let instance = config.instance_name();
    LimaClient::ensure_running(instance)?;

    let status = Command::new("limactl")
        .args([
            "shell",
            "--workdir",
            "/home/limavel",
            instance,
            "--",
            "sudo",
            "-iu",
            "limavel",
            "bash",
            "-c",
            cmd,
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
