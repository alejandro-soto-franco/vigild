use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub poll_interval_secs: u64,
    pub socket_path: PathBuf,
    pub network: NetworkConfig,
    pub units: UnitsConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub peers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnitsConfig {
    pub watch: Vec<String>,
}

pub fn load(path: &std::path::Path) -> anyhow::Result<Config> {
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let raw = r#"
            poll_interval_secs = 5
            socket_path = "/run/vigild/status.sock"

            [network]
            listen_addr = "0.0.0.0:7777"
            peers = []

            [units]
            watch = ["sshd.service", "polybius-engine.service"]
        "#;
        let cfg: Config = toml::from_str(raw).expect("parse");
        assert_eq!(cfg.poll_interval_secs, 5);
        assert_eq!(cfg.units.watch, ["sshd.service", "polybius-engine.service"]);
        assert!(cfg.network.peers.is_empty());
    }

    #[test]
    fn parse_with_peers() {
        let raw = r#"
            poll_interval_secs = 10
            socket_path = "/run/vigild/status.sock"

            [network]
            listen_addr = "0.0.0.0:7777"
            peers = ["100.64.0.2:7777", "100.64.0.3:7777"]

            [units]
            watch = []
        "#;
        let cfg: Config = toml::from_str(raw).expect("parse");
        assert_eq!(cfg.network.peers.len(), 2);
        assert_eq!(cfg.network.peers[0], "100.64.0.2:7777");
    }
}
