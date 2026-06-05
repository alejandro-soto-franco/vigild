use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
