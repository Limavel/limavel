use thiserror::Error;

#[derive(Error, Debug)]
pub enum LimavelError {
    #[error("{0} already exists in current directory")]
    ConfigAlreadyExists(String),

    #[error("{0} not found in current directory. Run 'limavel init' first.")]
    ConfigNotFound(String),

    #[error("lima-vm (limactl) not found. Install it with: brew install lima")]
    LimaNotFound,

    #[error("VM instance '{0}' does not exist. Run 'limavel start' first.")]
    InstanceNotFound(String),

    #[error("VM instance '{0}' is not running")]
    InstanceNotRunning(String),

    #[error("Failed to execute limactl: {0}")]
    LimactlExec(String),

    #[error("SSH key not found: {0}")]
    SshKeyNotFound(String),

    #[error("The following host directories do not exist:\n{0}")]
    FoldersNotFound(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yml::Error),
}
