use irondb::store::Store;
use std::thread;
use std::time::Duration;

#[test]
fn set_stores_value_and_get_returns_it() {
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
fn del_removes_key() {
    let mut store = Store::new();
    store.set("name", "Junan");
    assert!(store.del("name"));
    assert_eq!(store.get("name"), None);
}

#[test]
fn del_missing_key_returns_false() {
    let mut store = Store::new();
    assert!(!store.del("missing"));
}

#[test]
fn exists_returns_true_for_present_key() {
    let mut store = Store::new();
    store.set("name", "Junan");
    assert!(store.exists("name"));
}

#[test]
fn exists_returns_false_for_missing_key() {
    let mut store = Store::new();
    assert!(!store.exists("missing"));
}

#[test]
fn ttl_returns_negative_two_for_missing_key() {
    let mut store = Store::new();
    assert_eq!(store.ttl("missing"), -2);
}

#[test]
fn ttl_returns_negative_one_without_expiry() {
    let mut store = Store::new();
    store.set("name", "Junan");
    assert_eq!(store.ttl("name"), -1);
}

#[test]
fn expire_sets_ttl_and_ttl_returns_remaining_seconds() {
    let mut store = Store::new();
    store.set("session", "abc123");
    assert!(store.expire("session", 60));

    let ttl = store.ttl("session");
    assert!((58..=60).contains(&ttl));
}

#[test]
fn setex_stores_value_with_expiry() {
    let mut store = Store::new();
    store.setex("session:1", 60, "abc123");
    assert_eq!(store.get("session:1"), Some("abc123".to_string()));
}

#[test]
fn expired_key_disappears_on_get() {
    let mut store = Store::new();
    store.setex("temp", 1, "value");
    thread::sleep(Duration::from_secs(2));

    assert_eq!(store.get("temp"), None);
    assert!(!store.exists("temp"));
    assert_eq!(store.ttl("temp"), -2);
}

#[test]
fn cleanup_expired_removes_only_stale_keys() {
    let mut store = Store::new();
    store.set("keep", "yes");
    store.setex("drop", 1, "no");
    thread::sleep(Duration::from_secs(2));

    let removed = store.cleanup_expired();
    assert_eq!(removed, 1);
    assert_eq!(store.len(), 1);
    assert!(store.exists("keep"));
    assert!(!store.exists("drop"));
}
