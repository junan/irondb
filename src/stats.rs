use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ServerStats {
    start_time: Instant,
    connected_clients: AtomicU64,
    total_commands_processed: AtomicU64,
    expired_keys: AtomicU64,
    snapshot_path: Option<String>,
}

impl ServerStats {
    pub fn new(snapshot_path: Option<String>) -> Self {
        Self {
            start_time: Instant::now(),
            connected_clients: AtomicU64::new(0),
            total_commands_processed: AtomicU64::new(0),
            expired_keys: AtomicU64::new(0),
            snapshot_path,
        }
    }

    pub fn client_connected(&self) {
        self.connected_clients.fetch_add(1, Ordering::Relaxed);
    }

    pub fn client_disconnected(&self) {
        self.connected_clients.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn command_processed(&self) {
        self.total_commands_processed
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_lazy_expirations(&self, count: u64) {
        if count > 0 {
            self.expired_keys.fetch_add(count, Ordering::Relaxed);
        }
    }

    pub fn record_cleanup_expirations(&self, count: usize) {
        if count > 0 {
            self.expired_keys.fetch_add(count as u64, Ordering::Relaxed);
        }
    }

    pub fn info_fields(&self, key_count: usize) -> Vec<(String, String)> {
        let snapshot = self
            .snapshot_path
            .clone()
            .unwrap_or_else(|| "(none)".to_string());

        vec![
            ("irondb_version".into(), env!("CARGO_PKG_VERSION").into()),
            (
                "uptime_seconds".into(),
                self.start_time.elapsed().as_secs().to_string(),
            ),
            (
                "connected_clients".into(),
                self.connected_clients.load(Ordering::Relaxed).to_string(),
            ),
            (
                "total_commands_processed".into(),
                self.total_commands_processed
                    .load(Ordering::Relaxed)
                    .to_string(),
            ),
            ("keys".into(), key_count.to_string()),
            (
                "expired_keys".into(),
                self.expired_keys.load(Ordering::Relaxed).to_string(),
            ),
            ("snapshot_path".into(), snapshot),
        ]
    }
}
