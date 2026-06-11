use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::protocol::{format_response, parse_and_execute};
use crate::stats::ServerStats;
use crate::store::SharedStore;

pub async fn handle_connection(
    stream: TcpStream,
    store: SharedStore,
    stats: Arc<ServerStats>,
    snapshot_path: Option<String>,
) {
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown".into());

    stats.client_connected();
    tracing::info!(peer = %peer, "client connected");

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.is_empty() {
            continue;
        }

        let response = {
            let snapshot = snapshot_path.as_deref();
            let mut store = store.write().await;
            parse_and_execute(&line, &mut store, &stats, snapshot)
        };

        stats.command_processed();

        let output = format_response(&response);
        if writer.write_all(output.as_bytes()).await.is_err() {
            break;
        }
    }

    stats.client_disconnected();
    tracing::info!(peer = %peer, "client disconnected");
}
