use anyhow::Result;
use std::process::{Command, Stdio};

use crate::error::LimavelError;

pub struct LimaClient;

impl LimaClient {
    pub fn check_installed() -> Result<()> {
        which::which("limactl").map_err(|_| LimavelError::LimaNotFound)?;
        Ok(())
    }

    pub fn instance_exists(name: &str) -> Result<bool> {
        let output = Command::new("limactl")
            .args(["list", "--quiet"])
            .output()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().any(|line| line.trim() == name))
    }

    pub fn instance_status(name: &str) -> Result<String> {
        let obj = match Self::instance_json(name) {
            Ok(obj) => obj,
            Err(_) => return Ok("Unknown".to_string()),
        };
        let status = obj
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        Ok(status.to_string())
    }

    pub fn create(name: &str, template_path: &str) -> Result<()> {
        let status = Command::new("limactl")
            .args(["create", "--name", name, "--tty=false", template_path])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Failed to create instance".to_string()).into());
        }
        Ok(())
    }

    pub fn start(name: &str) -> Result<()> {
        let status = Command::new("limactl")
            .args(["start", name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Failed to start instance".to_string()).into());
        }
        Ok(())
    }

    pub fn stop(name: &str) -> Result<()> {
        let status = Command::new("limactl")
            .args(["stop", name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Failed to stop instance".to_string()).into());
        }
        Ok(())
    }

    pub fn restart(name: &str) -> Result<()> {
        Self::stop(name)?;
        Self::start(name)?;
        Ok(())
    }

    /// Parse the JSON object for a given instance from `limactl list --json`.
    fn instance_json(name: &str) -> Result<serde_json::Value> {
        let output = Command::new("limactl")
            .args(["list", "--json"])
            .output()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                if obj.get("name").and_then(|v| v.as_str()) == Some(name) {
                    return Ok(obj);
                }
            }
        }
        anyhow::bail!("Could not find instance '{}'", name)
    }

    pub fn instance_cpus(name: &str) -> Result<u32> {
        let obj = Self::instance_json(name)?;
        obj.get("cpus")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| anyhow::anyhow!("Could not determine cpus for instance '{}'", name))
    }

    pub fn instance_memory_mib(name: &str) -> Result<u32> {
        let obj = Self::instance_json(name)?;
        obj.get("memory")
            .and_then(|v| v.as_u64())
            .map(|bytes| (bytes / (1024 * 1024)) as u32)
            .ok_or_else(|| anyhow::anyhow!("Could not determine memory for instance '{}'", name))
    }

    pub fn instance_disk_gib(name: &str) -> Result<u32> {
        let obj = Self::instance_json(name)?;
        obj.get("disk")
            .and_then(|v| v.as_u64())
            .map(|bytes| (bytes / (1024 * 1024 * 1024)) as u32)
            .ok_or_else(|| anyhow::anyhow!("Could not determine disk size for instance '{}'", name))
    }

    pub fn edit(name: &str, cpus: u32, memory_mib: u32, disk_gib: Option<u32>) -> Result<()> {
        let mut args = vec![
            "edit".to_string(),
            name.to_string(),
            format!("--cpus={}", cpus),
            format!("--memory={}MiB", memory_mib),
        ];
        if let Some(disk) = disk_gib {
            args.push(format!("--disk={}GiB", disk));
        }
        args.push("--tty=false".to_string());

        let status = Command::new("limactl")
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Failed to edit instance".to_string()).into());
        }
        Ok(())
    }

    pub fn delete(name: &str) -> Result<()> {
        let status = Command::new("limactl")
            .args(["delete", name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Failed to delete instance".to_string()).into());
        }
        Ok(())
    }

    pub fn shell(name: &str, cmd: &str) -> Result<String> {
        let output = Command::new("limactl")
            .args(["shell", "--workdir", "/", name, "--", "bash", "-c", cmd])
            .output()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LimavelError::LimactlExec(format!(
                "Command failed: {}",
                stderr
            ))
            .into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn shell_interactive(name: &str, cmd: &str) -> Result<()> {
        let status = Command::new("limactl")
            .args(["shell", "--workdir", "/", name, "--", "bash", "-c", cmd])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(LimavelError::LimactlExec("Interactive command failed".to_string()).into());
        }
        Ok(())
    }

    /// Tar a local directory and extract it directly into a guest path.
    /// Uses the `tar` crate to build the archive, avoiding macOS-specific
    /// extended attributes that cause warnings on the Linux guest.
    pub fn tar_to_guest(name: &str, local_dir: &std::path::Path, remote_dir: &str) -> Result<()> {
        let mut child = Command::new("limactl")
            .args([
                "shell",
                "--workdir",
                "/",
                name,
                "--",
                "sudo",
                "tar",
                "-xf",
                "-",
                "-C",
                remote_dir,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LimavelError::LimactlExec("Failed to open stdin pipe".to_string()))?;

        let mut builder = tar::Builder::new(stdin);
        builder.follow_symlinks(false);
        builder
            .append_dir_all(".", local_dir)
            .map_err(|e| LimavelError::LimactlExec(format!("Failed to build tar archive: {}", e)))?;
        builder
            .into_inner()
            .map_err(|e| LimavelError::LimactlExec(format!("Failed to finish tar archive: {}", e)))?;

        let status = child
            .wait()
            .map_err(|e| LimavelError::LimactlExec(e.to_string()))?;

        if !status.success() {
            return Err(
                LimavelError::LimactlExec("Failed to copy files to guest via tar".to_string()).into(),
            );
        }
        Ok(())
    }

    pub fn guest_ip(name: &str) -> Result<String> {
        let output = Self::shell(
            name,
            "ip -4 addr show lima0 | grep -oP 'inet \\K[0-9.]+'"
        )?;
        let ip = output.trim().to_string();
        if ip.is_empty() {
            anyhow::bail!("Could not determine guest IP from lima0 interface");
        }
        Ok(ip)
    }

    pub fn ensure_running(name: &str) -> Result<()> {
        if !Self::instance_exists(name)? {
            return Err(LimavelError::InstanceNotFound(name.to_string()).into());
        }
        let status = Self::instance_status(name)?;
        if status != "Running" {
            return Err(LimavelError::InstanceNotRunning(name.to_string()).into());
        }
        Ok(())
    }
}
