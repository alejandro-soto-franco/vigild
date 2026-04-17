use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use vigild_core::peer::{decode_frame, encode_frame, HealthReport};

pub async fn run_network(
    listen_addr: String,
    peers: Vec<String>,
    inbound_tx: broadcast::Sender<HealthReport>,
    mut outbound_rx: broadcast::Receiver<HealthReport>,
) {
    let inbound_tx2 = inbound_tx.clone();
    let listen = listen_addr.clone();

    tokio::spawn(async move {
        let listener = match TcpListener::bind(&listen).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind TCP listener on {listen}: {e}");
                return;
            }
        };
        info!("Peer listener on {listen}");
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Peer connected from {addr}");
                    let tx = inbound_tx2.clone();
                    tokio::spawn(receive_loop(stream, tx));
                }
                Err(e) => error!("accept error: {e}"),
            }
        }
    });

    tokio::spawn(async move {
        loop {
            match outbound_rx.recv().await {
                Ok(report) => {
                    let frame = match encode_frame(&report) {
                        Ok(f) => f,
                        Err(e) => {
                            error!("encode_frame failed: {e}");
                            continue;
                        }
                    };
                    for peer in &peers {
                        if let Err(e) = send_frame_to(peer, &frame).await {
                            warn!("Failed to send to peer {peer}: {e}");
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("network outbound lagged {n}");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

async fn receive_loop(mut stream: TcpStream, tx: broadcast::Sender<HealthReport>) {
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut len_buf).await {
            if e.kind() != ErrorKind::UnexpectedEof {
                error!("peer read error: {e}");
            }
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        if stream.read_exact(&mut payload).await.is_err() {
            break;
        }
        match decode_frame(&payload) {
            Ok(report) => {
                let _ = tx.send(report);
            }
            Err(e) => error!("decode_frame error: {e}"),
        }
    }
}

async fn send_frame_to(addr: &str, frame: &[u8]) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    let len = (frame.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(frame).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vigild_core::{
        peer::HealthReport,
        unit::{ActiveState, UnitStatus},
    };

    #[tokio::test]
    async fn two_peers_exchange_reports() {
        let (tx_a, _) = broadcast::channel::<HealthReport>(8);
        let (tx_b, _) = broadcast::channel::<HealthReport>(8);

        tokio::spawn(run_network(
            "0.0.0.0:7781".into(),
            vec![],
            tx_a.clone(),
            tx_a.subscribe(),
        ));
        tokio::spawn(run_network(
            "0.0.0.0:7782".into(),
            vec!["127.0.0.1:7781".into()],
            tx_b.clone(),
            tx_b.subscribe(),
        ));

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut rx_a = tx_a.subscribe();

        let report = HealthReport {
            host: "peer-b".into(),
            timestamp_unix: 1_700_000_000,
            units: vec![UnitStatus {
                name: "test.service".into(),
                description: "test".into(),
                load_state: "loaded".into(),
                active: ActiveState::Active,
                sub_state: "running".into(),
            }],
        };
        tx_b.send(report).unwrap();

        let received = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx_a.recv(),
        )
        .await;

        assert!(received.is_ok(), "peer A did not receive report from peer B");
    }
}
