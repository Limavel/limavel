use anyhow::{Context, Result};
use serde::Serialize;

use super::limavel_config::LimavelConfig;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimaConfig {
    pub vm_type: String,
    pub os: String,
    pub arch: String,
    pub images: Vec<LimaImage>,
    pub cpus: u32,
    pub memory: String,
    pub disk: String,
    pub mount_type: String,
    pub mounts: Vec<LimaMount>,
    pub networks: Vec<LimaNetwork>,
    pub port_forwards: Vec<LimaPortForward>,
    pub ssh: LimaSsh,
    pub containerd: LimaContainerd,
    pub provision: Vec<LimaProvision>,
    pub rosetta: LimaRosetta,
}

#[derive(Debug, Serialize)]
pub struct LimaImage {
    pub location: String,
    pub arch: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimaMount {
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_point: Option<String>,
    pub writable: bool,
}

#[derive(Debug, Serialize)]
pub struct LimaNetwork {
    #[serde(rename = "vzNAT")]
    pub vz_nat: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimaPortForward {
    pub guest_port: u16,
    pub host_port: u16,
}

#[derive(Debug, Serialize)]
pub struct LimaSsh {
    #[serde(rename = "loadDotSSHPubKeys")]
    pub load_dot_ssh_pub_keys: bool,
}

#[derive(Debug, Serialize)]
pub struct LimaContainerd {
    pub system: bool,
    pub user: bool,
}

#[derive(Debug, Serialize)]
pub struct LimaProvision {
    pub mode: String,
    pub script: String,
}

#[derive(Debug, Serialize)]
pub struct LimaRosetta {
    pub enabled: bool,
}

impl LimaConfig {
    pub fn from_config(config: &LimavelConfig, ssh_pubkey: &str) -> Result<Self> {
        let mut mounts: Vec<LimaMount> = config
            .folders
            .iter()
            .map(|f| {
                let expanded = shellexpand::tilde(&f.map).to_string();
                LimaMount {
                    location: expanded,
                    mount_point: Some(f.to.clone()),
                    writable: true,
                }
            })
            .collect();

        // Add /tmp/lima mount
        mounts.push(LimaMount {
            location: "/tmp/lima".to_string(),
            mount_point: None,
            writable: true,
        });

        let port_forwards: Vec<LimaPortForward> = config
            .ports
            .iter()
            .map(|p| LimaPortForward {
                guest_port: p.to,
                host_port: p.send,
            })
            .collect();

        let bootstrap_script = generate_bootstrap_script(ssh_pubkey, &config.bootstrap)?;

        Ok(LimaConfig {
            vm_type: "vz".to_string(),
            os: "Linux".to_string(),
            arch: config.arch.clone(),
            images: vec![
                LimaImage {
                    location: config.image.clone(),
                    arch: config.arch.clone(),
                },
            ],
            cpus: config.cpus,
            memory: format!("{}MiB", config.memory),
            disk: format!("{}GiB", config.disk),
            mount_type: "virtiofs".to_string(),
            mounts,
            networks: vec![LimaNetwork { vz_nat: true }],
            port_forwards,
            ssh: LimaSsh {
                load_dot_ssh_pub_keys: true,
            },
            containerd: LimaContainerd {
                system: false,
                user: false,
            },
            provision: vec![LimaProvision {
                mode: "system".to_string(),
                script: bootstrap_script,
            }],
            rosetta: LimaRosetta {
                enabled: config.arch != "aarch64",
            },
        })
    }

    pub fn to_yaml(&self) -> Result<String> {
        let yaml = serde_yml::to_string(self)?;
        Ok(yaml)
    }
}

fn generate_bootstrap_script(ssh_pubkey: &str, custom_path: &Option<String>) -> Result<String> {
    let template = if let Some(path) = custom_path {
        let expanded = shellexpand::tilde(path).to_string();
        std::fs::read_to_string(&expanded)
            .with_context(|| format!("Failed to read custom bootstrap script '{}'", expanded))?
    } else {
        use include_dir::{include_dir, Dir};

        static BOOTSTRAPS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/provision/bootstrap");
        BOOTSTRAPS_DIR
            .get_file("main.sh")
            .expect("bootstrap/main.sh missing from embedded directory")
            .contents_utf8()
            .expect("bootstrap/main.sh is not valid UTF-8")
            .to_string()
    };

    Ok(template.replace("{ssh_pubkey}", ssh_pubkey))
}
