use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use vigild_core::{
    peer::HealthReport,
    systemd::{connect_system_bus, query_units, query_watched_units},
};

pub async fn run_collector(
    tx: broadcast::Sender<HealthReport>,
    watch: Vec<String>,
    poll_interval_secs: u64,
) {
    let hostname = hostname();
    let conn = match connect_system_bus().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to D-Bus: {e}");
            return;
        }
    };

    let mut ticker = interval(Duration::from_secs(poll_interval_secs));
    loop {
        ticker.tick().await;
        let query_result = if watch.is_empty() {
            query_units(&conn).await
        } else {
            query_watched_units(&conn, &watch).await
        };
        match query_result {
            Ok(units) => {
                let report = HealthReport {
                    host: hostname.clone(),
                    timestamp_unix: unix_now(),
                    units,
                };
                if tx.send(report).is_err() {
                    info!("No receivers for health report, continuing");
                }
            }
            Err(e) => error!("unit query failed: {e}"),
        }
    }
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn collector_sends_report() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let (tx, mut rx) = broadcast::channel(8);
        let handle = tokio::spawn(run_collector(tx, vec![], 1));

        let report = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            rx.recv(),
        )
        .await
        .expect("timeout waiting for report")
        .expect("recv error");

        assert!(!report.units.is_empty());
        handle.abort();
    }
}
