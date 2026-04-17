use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tracing::{error, info};
use vigild_core::peer::HealthReport;

pub async fn run_socket_server(
    socket_path: PathBuf,
    mut rx: broadcast::Receiver<HealthReport>,
) {
    let _ = std::fs::remove_file(&socket_path);
    if let Some(parent) = socket_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind Unix socket at {socket_path:?}: {e}");
            return;
        }
    };
    info!("Unix socket listening at {socket_path:?}");

    let (client_tx, _) = tokio::sync::broadcast::channel::<String>(64);
    let client_tx2 = client_tx.clone();

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let mut client_rx = client_tx2.subscribe();
                    tokio::spawn(async move {
                        loop {
                            match client_rx.recv().await {
                                Ok(line) => {
                                    if stream.write_all(line.as_bytes()).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
                Err(e) => error!("accept error: {e}"),
            }
        }
    });

    loop {
        match rx.recv().await {
            Ok(report) => {
                if let Ok(mut line) = serde_json::to_string(&report) {
                    line.push('\n');
                    let _ = client_tx.send(line);
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                error!("socket server lagged by {n} messages");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use tokio::net::UnixStream;
    use vigild_core::{
        peer::HealthReport,
        unit::{ActiveState, UnitStatus},
    };

    #[tokio::test]
    async fn socket_streams_json() {
        let path = "/tmp/vigild-test.sock";
        let _ = std::fs::remove_file(path);
        let (tx, _rx) = broadcast::channel(8);
        let tx2 = tx.clone();

        tokio::spawn(run_socket_server(path.into(), tx.subscribe()));

        // Wait for socket to bind
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Connect BEFORE sending so the client_tx subscriber exists
        let mut stream = UnixStream::connect(path).await.expect("connect");

        // Small pause to let the accept task register the subscriber
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let report = HealthReport {
            host: "test-host".into(),
            timestamp_unix: 1_700_000_000,
            units: vec![UnitStatus {
                name: "sshd.service".into(),
                description: "SSH Daemon".into(),
                load_state: "loaded".into(),
                active: ActiveState::Active,
                sub_state: "running".into(),
            }],
        };
        tx2.send(report).unwrap();

        let mut buf = String::new();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let mut byte = [0u8; 1];
            loop {
                stream.read_exact(&mut byte).await.unwrap();
                if byte[0] == b'\n' {
                    break;
                }
                buf.push(byte[0] as char);
            }
        })
        .await
        .expect("timeout reading line");

        let val: serde_json::Value = serde_json::from_str(&buf).expect("parse JSON");
        assert_eq!(val["host"], "test-host");
        let _ = std::fs::remove_file(path);
    }
}
