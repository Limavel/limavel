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

#[cfg(test)]
mod tests {
    use super::*;

    const PUBKEY: &str = "ssh-ed25519 AAAATESTKEY user@host";

    fn limavel_fixture() -> LimavelConfig {
        let yaml = include_str!("../../templates/default.yaml").replace("{{ name }}", "testbox");
        serde_yml::from_str(&yaml).unwrap()
    }

    #[test]
    fn from_config_maps_resources() {
        let lima = LimaConfig::from_config(&limavel_fixture(), PUBKEY).unwrap();
        assert_eq!(lima.cpus, 2);
        assert_eq!(lima.memory, "2048MiB");
        assert_eq!(lima.disk, "50GiB");
        assert_eq!(lima.arch, "aarch64");
        assert_eq!(lima.vm_type, "vz");
        assert_eq!(lima.mount_type, "virtiofs");
    }

    #[test]
    fn rosetta_is_only_enabled_for_non_arm() {
        let mut config = limavel_fixture();
        assert!(!LimaConfig::from_config(&config, PUBKEY).unwrap().rosetta.enabled);
        config.arch = "x86_64".to_string();
        assert!(LimaConfig::from_config(&config, PUBKEY).unwrap().rosetta.enabled);
    }

    #[test]
    fn mounts_include_folders_and_tmp_lima() {
        let lima = LimaConfig::from_config(&limavel_fixture(), PUBKEY).unwrap();
        assert_eq!(lima.mounts.len(), 2);
        assert!(lima.mounts[0].location.ends_with("/myproject"));
        assert!(!lima.mounts[0].location.starts_with('~'), "tilde should be expanded");
        assert_eq!(lima.mounts[0].mount_point.as_deref(), Some("/home/limavel/myproject"));
        let tmp = lima.mounts.last().unwrap();
        assert_eq!(tmp.location, "/tmp/lima");
        assert!(tmp.mount_point.is_none());
    }

    #[test]
    fn port_forwards_map_send_to_host_and_to_to_guest() {
        let lima = LimaConfig::from_config(&limavel_fixture(), PUBKEY).unwrap();
        assert_eq!(lima.port_forwards[0].host_port, 33060);
        assert_eq!(lima.port_forwards[0].guest_port, 3306);
    }

    #[test]
    fn bootstrap_script_substitutes_ssh_pubkey() {
        let script = generate_bootstrap_script(PUBKEY, &None).unwrap();
        assert!(script.contains(PUBKEY));
        assert!(!script.contains("{ssh_pubkey}"));
    }

    #[test]
    fn to_yaml_uses_lima_field_names() {
        let yaml = LimaConfig::from_config(&limavel_fixture(), PUBKEY)
            .unwrap()
            .to_yaml()
            .unwrap();
        assert!(yaml.contains("vmType: vz"));
        assert!(yaml.contains("mountPoint: /home/limavel/myproject"));
        assert!(yaml.contains("portForwards:"));
        assert!(yaml.contains("vzNAT: true"));
        assert!(yaml.contains("loadDotSSHPubKeys: true"));
    }
}
