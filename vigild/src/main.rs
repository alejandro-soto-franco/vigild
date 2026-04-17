mod aggregator;
mod collector;
mod config;
mod error;
mod network;
mod socket;

use std::{path::PathBuf, sync::Arc};
use tokio::{
    signal,
    sync::{broadcast, Mutex},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/vigild/config.toml"));

    let cfg = config::load(&config_path)?;

    info!(
        "vigild starting, poll={}s, peers={}",
        cfg.poll_interval_secs,
        cfg.network.peers.len()
    );

    let (tx, _) = broadcast::channel::<vigild_core::peer::HealthReport>(256);

    let state = Arc::new(Mutex::new(aggregator::SharedState::new()));
    let state2 = state.clone();

    let mut agg_rx = tx.subscribe();
    tokio::spawn(async move {
        loop {
            if let Ok(report) = agg_rx.recv().await {
                state2.lock().await.update(report);
            }
        }
    });

    tokio::spawn(collector::run_collector(
        tx.clone(),
        cfg.units.watch.clone(),
        cfg.poll_interval_secs,
    ));

    tokio::spawn(socket::run_socket_server(
        cfg.socket_path.clone(),
        tx.subscribe(),
    ));

    tokio::spawn(network::run_network(
        cfg.network.listen_addr.clone(),
        cfg.network.peers.clone(),
        tx.clone(),
        tx.subscribe(),
    ));

    signal::ctrl_c().await?;
    info!("vigild shutting down");
    Ok(())
}
