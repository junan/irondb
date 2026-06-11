use std::sync::Arc;
use std::time::Duration;

use irondb::config::Config;
use irondb::server;
use irondb::stats::ServerStats;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

async fn send_command(port: u16, command: &str) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .expect("connect to server");

    stream
        .write_all(format!("{command}\n").as_bytes())
        .await
        .expect("write command");

    let mut reader = tokio::io::BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .await
        .expect("read response");
    response
}

async fn send_info(port: u16) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .expect("connect to server");

    stream.write_all(b"INFO\n").await.expect("write INFO");

    let mut reader = tokio::io::BufReader::new(stream);
    let mut response = String::new();
    for _ in 0..7 {
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read INFO line");
        response.push_str(&line);
    }
    response
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn mvp_commands_over_tcp() {
    let port = free_port();
    let config = Config {
        port,
        snapshot: None,
    };

    let server = tokio::spawn(async move {
        server::run(config).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    assert_eq!(send_command(port, "PING").await.trim(), "PONG");
    assert_eq!(send_command(port, "SET name Junan").await.trim(), "OK");
    assert_eq!(send_command(port, "GET name").await.trim(), "Junan");
    assert_eq!(send_command(port, "EXISTS name").await.trim(), "1");
    assert_eq!(send_command(port, "DEL name").await.trim(), "1");
    assert_eq!(send_command(port, "GET name").await.trim(), "(nil)");

    server.abort();
}

#[tokio::test]
async fn ttl_commands_over_tcp() {
    let port = free_port();
    let config = Config {
        port,
        snapshot: None,
    };

    let server = tokio::spawn(async move {
        server::run(config).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    assert_eq!(send_command(port, "SETEX temp 2 value").await.trim(), "OK");
    let ttl = send_command(port, "TTL temp")
        .await
        .trim()
        .parse::<i64>()
        .unwrap();
    assert!((1..=2).contains(&ttl));

    sleep(Duration::from_secs(3)).await;
    assert_eq!(send_command(port, "GET temp").await.trim(), "(nil)");
    assert_eq!(send_command(port, "TTL temp").await.trim(), "-2");

    server.abort();
}

#[tokio::test]
async fn info_command_over_tcp() {
    let port = free_port();
    let config = Config {
        port,
        snapshot: None,
    };

    let server = tokio::spawn(async move {
        server::run(config).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let _ = send_command(port, "SET a 1").await;
    let info = send_info(port).await;
    assert!(info.contains("irondb_version:0.1.0"));
    assert!(info.contains("keys:"));
    assert!(info.contains("connected_clients:"));

    server.abort();
}

#[tokio::test]
async fn snapshot_save_and_load_over_tcp() {
    let port = free_port();
    let snapshot = std::env::temp_dir().join(format!(
        "irondb-integration-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let snapshot = snapshot.to_string_lossy().into_owned();

    let config = Config {
        port,
        snapshot: Some(snapshot.clone()),
    };

    let server = tokio::spawn(async move {
        server::run(config).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    assert_eq!(send_command(port, "SET name Junan").await.trim(), "OK");
    assert_eq!(send_command(port, "SAVE").await.trim(), "OK");
    assert_eq!(send_command(port, "DEL name").await.trim(), "1");
    assert_eq!(send_command(port, "GET name").await.trim(), "(nil)");
    assert_eq!(send_command(port, "LOAD").await.trim(), "OK");
    assert_eq!(send_command(port, "GET name").await.trim(), "Junan");

    server.abort();
    let _ = std::fs::remove_file(&snapshot);
}

#[test]
fn stats_track_commands() {
    let stats = Arc::new(ServerStats::new(Some("./test.snapshot.json".into())));
    stats.client_connected();
    stats.command_processed();
    stats.command_processed();
    stats.record_cleanup_expirations(3);

    let fields = stats.info_fields(5);
    let map: std::collections::HashMap<_, _> = fields.into_iter().collect();
    assert_eq!(map.get("keys").unwrap(), "5");
    assert_eq!(map.get("expired_keys").unwrap(), "3");
    assert_eq!(map.get("total_commands_processed").unwrap(), "2");
}
