use crate::command::Command;
use crate::error::IronDbError;
use crate::persistence;
use crate::stats::ServerStats;
use crate::store::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Simple(String),
    Bulk(Option<String>),
    Integer(i64),
    Error(String),
    Info(Vec<(String, String)>),
}

pub fn format_response(response: &Response) -> String {
    match response {
        Response::Simple(s) => format!("{s}\n"),
        Response::Bulk(Some(s)) => format!("{s}\n"),
        Response::Bulk(None) => "(nil)\n".to_string(),
        Response::Integer(n) => format!("{n}\n"),
        Response::Error(msg) => format!("ERR {msg}\n"),
        Response::Info(fields) => {
            fields
                .iter()
                .map(|(k, v)| format!("{k}:{v}"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
    }
}

pub fn execute(
    command: Command,
    store: &mut Store,
    stats: &ServerStats,
    snapshot_path: Option<&str>,
) -> Response {
    match command {
        Command::Ping => Response::Simple("PONG".into()),
        Command::Set { key, value } => {
            store.set(&key, &value);
            Response::Simple("OK".into())
        }
        Command::Get { key } => {
            let value = store.get(&key);
            stats.record_lazy_expirations(store.take_lazy_expired_count());
            Response::Bulk(value)
        }
        Command::Del { key } => {
            let removed = store.del(&key);
            stats.record_lazy_expirations(store.take_lazy_expired_count());
            Response::Integer(i64::from(removed))
        }
        Command::Exists { key } => {
            let exists = store.exists(&key);
            stats.record_lazy_expirations(store.take_lazy_expired_count());
            Response::Integer(i64::from(exists))
        }
        Command::Expire { key, seconds } => {
            if store.expire(&key, seconds) {
                stats.record_lazy_expirations(store.take_lazy_expired_count());
                Response::Simple("OK".into())
            } else {
                stats.record_lazy_expirations(store.take_lazy_expired_count());
                Response::Integer(0)
            }
        }
        Command::Ttl { key } => {
            let ttl = store.ttl(&key);
            stats.record_lazy_expirations(store.take_lazy_expired_count());
            Response::Integer(ttl)
        }
        Command::SetEx {
            key,
            seconds,
            value,
        } => {
            store.setex(&key, seconds, &value);
            Response::Simple("OK".into())
        }
        Command::Save => match snapshot_path {
            Some(path) => match persistence::save_snapshot(path, store) {
                Ok(()) => Response::Simple("OK".into()),
                Err(err) => Response::Error(err.to_string()),
            },
            None => Response::Error("snapshot path not configured".into()),
        },
        Command::Load => match snapshot_path {
            Some(path) => match persistence::load_snapshot(path) {
                Ok(entries) => {
                    store.load_entries(entries);
                    Response::Simple("OK".into())
                }
                Err(err) => Response::Error(err.to_string()),
            },
            None => Response::Error("snapshot path not configured".into()),
        },
        Command::Info => Response::Info(stats.info_fields(store.len())),
    }
}

pub fn parse_and_execute(
    line: &str,
    store: &mut Store,
    stats: &ServerStats,
    snapshot_path: Option<&str>,
) -> Response {
    match crate::command::parse_command(line) {
        Ok(command) => execute(command, store, stats, snapshot_path),
        Err(IronDbError::Parse(msg)) => Response::Error(msg),
        Err(err) => Response::Error(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::ServerStats;
    use std::sync::Arc;

    #[test]
    fn formats_simple_response() {
        assert_eq!(format_response(&Response::Simple("OK".into())), "OK\n");
    }

    #[test]
    fn formats_nil_bulk() {
        assert_eq!(format_response(&Response::Bulk(None)), "(nil)\n");
    }

    #[test]
    fn formats_error() {
        assert_eq!(format_response(&Response::Error("bad".into())), "ERR bad\n");
    }

    #[test]
    fn ping_returns_pong() {
        let mut store = Store::new();
        let stats = Arc::new(ServerStats::new(None));
        let response = execute(Command::Ping, &mut store, &stats, None);
        assert_eq!(response, Response::Simple("PONG".into()));
    }
}
