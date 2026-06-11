use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::Utc;

use crate::error::{IronDbError, Result};
use crate::store::{Entry, Store};

pub fn save_snapshot(path: &str, store: &Store) -> Result<()> {
    let now = Utc::now();
    let live_entries: HashMap<String, Entry> = store
        .entries()
        .iter()
        .filter(|(_, entry)| !is_expired(entry, now))
        .map(|(key, entry)| (key.clone(), entry.clone()))
        .collect();

    let json = serde_json::to_string_pretty(&live_entries)
        .map_err(|err| IronDbError::Persistence(err.to_string()))?;

    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, json)?;
    Ok(())
}

pub fn load_snapshot(path: &str) -> Result<HashMap<String, Entry>> {
    if !Path::new(path).exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(path)?;
    let entries: HashMap<String, Entry> =
        serde_json::from_str(&contents).map_err(|err| IronDbError::Persistence(err.to_string()))?;

    let now = Utc::now();
    Ok(entries
        .into_iter()
        .filter(|(_, entry)| !is_expired(entry, now))
        .collect())
}

fn is_expired(entry: &Entry, now: chrono::DateTime<Utc>) -> bool {
    entry.expires_at.is_some_and(|expires_at| now >= expires_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("irondb-{name}-{nanos}.json"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = temp_path("roundtrip");
        let mut store = Store::new();
        store.set("name", "Junan");
        save_snapshot(&path, &store).unwrap();

        let loaded = load_snapshot(&path).unwrap();
        assert_eq!(loaded.get("name").unwrap().value, "Junan");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn save_excludes_expired_keys() {
        let path = temp_path("expired");
        let mut store = Store::new();
        store.set("keep", "yes");
        store.setex("drop", 1, "no");
        std::thread::sleep(std::time::Duration::from_secs(2));

        save_snapshot(&path, &store).unwrap();
        let loaded = load_snapshot(&path).unwrap();
        assert!(loaded.contains_key("keep"));
        assert!(!loaded.contains_key("drop"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let path = temp_path("missing");
        let loaded = load_snapshot(&path).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn load_filters_expired_entries_from_file() {
        let path = temp_path("filter");
        let mut entries: HashMap<String, Entry> = HashMap::new();
        entries.insert(
            "expired".into(),
            Entry {
                value: "gone".into(),
                expires_at: Some(Utc::now() - Duration::seconds(10)),
            },
        );
        entries.insert(
            "live".into(),
            Entry {
                value: "here".into(),
                expires_at: None,
            },
        );
        fs::write(&path, serde_json::to_string_pretty(&entries).unwrap()).unwrap();

        let loaded = load_snapshot(&path).unwrap();
        assert!(loaded.contains_key("live"));
        assert!(!loaded.contains_key("expired"));

        let _ = fs::remove_file(&path);
    }
}
