use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

use crate::config::Config;
use crate::connection::handle_connection;
use crate::error::Result;
use crate::persistence;
use crate::stats::ServerStats;
use crate::store::{SharedStore, Store};

pub async fn run(config: Config) -> Result<()> {
    let mut store = Store::new();

    if let Some(ref path) = config.snapshot {
        let entries = persistence::load_snapshot(path)?;
        if !entries.is_empty() {
            tracing::info!(path = %path, keys = entries.len(), "loaded snapshot");
            store.load_entries(entries);
        }
    }

    let store: SharedStore = Arc::new(RwLock::new(store));
    let stats = Arc::new(ServerStats::new(config.snapshot.clone()));
    let snapshot_path = config.snapshot.clone();

    spawn_cleanup_task(Arc::clone(&store), Arc::clone(&stats));

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "IronDB listening");

    loop {
        let (stream, peer) = listener.accept().await?;
        tracing::debug!(peer = %peer, "accepted connection");

        let store = Arc::clone(&store);
        let stats = Arc::clone(&stats);
        let snapshot = snapshot_path.clone();

        tokio::spawn(async move {
            handle_connection(stream, store, stats, snapshot).await;
        });
    }
}

fn spawn_cleanup_task(store: SharedStore, stats: Arc<ServerStats>) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let mut guard = store.write().await;
            let removed = guard.cleanup_expired();
            drop(guard);
            if removed > 0 {
                stats.record_cleanup_expirations(removed);
                tracing::debug!(removed, "cleaned up expired keys");
            }
        }
    });
}
