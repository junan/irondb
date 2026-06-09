use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub value: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct Store {
    data: HashMap<String, Entry>,
}

pub type SharedStore = Arc<RwLock<Store>>;

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(
            key.to_string(),
            Entry {
                value: value.to_string(),
                expires_at: None,
            },
        );
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        let now = Utc::now();
        self.remove_if_expired(key, now);
        self.data.get(key).map(|entry| entry.value.clone())
    }

    pub fn del(&mut self, key: &str) -> bool {
        let now = Utc::now();
        self.remove_if_expired(key, now);
        self.data.remove(key).is_some()
    }

    pub fn exists(&mut self, key: &str) -> bool {
        let now = Utc::now();
        self.remove_if_expired(key, now);
        self.data.contains_key(key)
    }

    pub fn expire(&mut self, key: &str, seconds: u64) -> bool {
        let now = Utc::now();
        self.remove_if_expired(key, now);
        if let Some(entry) = self.data.get_mut(key) {
            entry.expires_at = Some(now + Duration::seconds(seconds as i64));
            true
        } else {
            false
        }
    }

    pub fn ttl(&mut self, key: &str) -> i64 {
        let now = Utc::now();
        self.remove_if_expired(key, now);

        match self.data.get(key) {
            None => -2,
            Some(entry) => match entry.expires_at {
                None => -1,
                Some(expires_at) => (expires_at - now).num_seconds().max(0),
            },
        }
    }

    pub fn setex(&mut self, key: &str, seconds: u64, value: &str) {
        let now = Utc::now();
        self.data.insert(
            key.to_string(),
            Entry {
                value: value.to_string(),
                expires_at: Some(now + Duration::seconds(seconds as i64)),
            },
        );
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let now = Utc::now();
        let expired_keys: Vec<String> = self
            .data
            .iter()
            .filter(|(_, entry)| is_expired(entry, now))
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.data.remove(&key);
        }
        count
    }

    pub fn len(&self) -> usize {
        let now = Utc::now();
        self.data
            .iter()
            .filter(|(_, entry)| !is_expired(entry, now))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn entries(&self) -> &HashMap<String, Entry> {
        &self.data
    }

    pub fn load_entries(&mut self, entries: HashMap<String, Entry>) {
        self.data = entries;
    }

    fn remove_if_expired(&mut self, key: &str, now: DateTime<Utc>) -> bool {
        let expired = self
            .data
            .get(key)
            .is_some_and(|entry| is_expired(entry, now));

        if expired {
            self.data.remove(key);
        }

        expired
    }
}

fn is_expired(entry: &Entry, now: DateTime<Utc>) -> bool {
    entry.expires_at.is_some_and(|expires_at| now >= expires_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn set_and_get_round_trip() {
        let mut store = Store::new();
        store.set("name", "Junan");
        assert_eq!(store.get("name"), Some("Junan".to_string()));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let mut store = Store::new();
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn del_removes_existing_key() {
        let mut store = Store::new();
        store.set("name", "Junan");
        assert!(store.del("name"));
        assert!(!store.exists("name"));
    }

    #[test]
    fn del_missing_key_returns_false() {
        let mut store = Store::new();
        assert!(!store.del("missing"));
    }

    #[test]
    fn exists_reflects_presence() {
        let mut store = Store::new();
        store.set("name", "Junan");
        assert!(store.exists("name"));
        assert!(!store.exists("missing"));
    }

    #[test]
    fn expire_and_ttl_semantics() {
        let mut store = Store::new();
        assert_eq!(store.ttl("missing"), -2);

        store.set("session", "abc");
        assert_eq!(store.ttl("session"), -1);

        assert!(store.expire("session", 60));
        let ttl = store.ttl("session");
        assert!((58..=60).contains(&ttl));
    }

    #[test]
    fn setex_sets_value_with_ttl() {
        let mut store = Store::new();
        store.setex("session", 30, "token");
        assert_eq!(store.get("session"), Some("token".to_string()));
        let ttl = store.ttl("session");
        assert!((28..=30).contains(&ttl));
    }

    #[test]
    fn expired_key_is_removed_on_access() {
        let mut store = Store::new();
        store.setex("temp", 1, "gone");
        thread::sleep(StdDuration::from_secs(2));

        assert_eq!(store.get("temp"), None);
        assert!(!store.exists("temp"));
        assert_eq!(store.ttl("temp"), -2);
    }

    #[test]
    fn cleanup_expired_removes_stale_keys() {
        let mut store = Store::new();
        store.set("persistent", "keep");
        store.setex("temp", 1, "gone");
        thread::sleep(StdDuration::from_secs(2));

        let removed = store.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(store.len(), 1);
        assert!(store.exists("persistent"));
    }
}
