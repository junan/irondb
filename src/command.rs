use crate::error::{IronDbError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Ping,
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Del {
        key: String,
    },
    Exists {
        key: String,
    },
    Expire {
        key: String,
        seconds: u64,
    },
    Ttl {
        key: String,
    },
    SetEx {
        key: String,
        seconds: u64,
        value: String,
    },
    Save,
    Load,
    Info,
}

pub fn parse_command(line: &str) -> Result<Command> {
    let line = line.trim();
    if line.is_empty() {
        return Err(IronDbError::Parse("empty command".into()));
    }

    let mut parts = line.split_whitespace();
    let cmd = parts
        .next()
        .ok_or_else(|| IronDbError::Parse("empty command".into()))?
        .to_ascii_uppercase();

    match cmd.as_str() {
        "PING" => {
            if parts.next().is_some() {
                return Err(IronDbError::Parse("PING takes no arguments".into()));
            }
            Ok(Command::Ping)
        }
        "SET" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("SET requires key and value".into()))?;
            let value = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("SET requires key and value".into()))?;
            let value = if let Some(rest) = parts.next() {
                format!(
                    "{value} {}",
                    parts.fold(rest.to_string(), |acc, p| format!("{acc} {p}"))
                )
            } else {
                value.to_string()
            };
            Ok(Command::Set {
                key: key.to_string(),
                value,
            })
        }
        "GET" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("GET requires a key".into()))?;
            if parts.next().is_some() {
                return Err(IronDbError::Parse("GET takes exactly one argument".into()));
            }
            Ok(Command::Get {
                key: key.to_string(),
            })
        }
        "DEL" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("DEL requires a key".into()))?;
            if parts.next().is_some() {
                return Err(IronDbError::Parse("DEL takes exactly one argument".into()));
            }
            Ok(Command::Del {
                key: key.to_string(),
            })
        }
        "EXISTS" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("EXISTS requires a key".into()))?;
            if parts.next().is_some() {
                return Err(IronDbError::Parse(
                    "EXISTS takes exactly one argument".into(),
                ));
            }
            Ok(Command::Exists {
                key: key.to_string(),
            })
        }
        "EXPIRE" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("EXPIRE requires key and seconds".into()))?;
            let seconds_str = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("EXPIRE requires key and seconds".into()))?;
            if parts.next().is_some() {
                return Err(IronDbError::Parse(
                    "EXPIRE takes exactly two arguments".into(),
                ));
            }
            let seconds = seconds_str
                .parse::<u64>()
                .map_err(|_| IronDbError::Parse("EXPIRE seconds must be a number".into()))?;
            Ok(Command::Expire {
                key: key.to_string(),
                seconds,
            })
        }
        "TTL" => {
            let key = parts
                .next()
                .ok_or_else(|| IronDbError::Parse("TTL requires a key".into()))?;
            if parts.next().is_some() {
                return Err(IronDbError::Parse("TTL takes exactly one argument".into()));
            }
            Ok(Command::Ttl {
                key: key.to_string(),
            })
        }
        "SETEX" => {
            let key = parts.next().ok_or_else(|| {
                IronDbError::Parse("SETEX requires key, seconds, and value".into())
            })?;
            let seconds_str = parts.next().ok_or_else(|| {
                IronDbError::Parse("SETEX requires key, seconds, and value".into())
            })?;
            let value = parts.next().ok_or_else(|| {
                IronDbError::Parse("SETEX requires key, seconds, and value".into())
            })?;
            let value = if let Some(rest) = parts.next() {
                format!(
                    "{value} {}",
                    parts.fold(rest.to_string(), |acc, p| format!("{acc} {p}"))
                )
            } else {
                value.to_string()
            };
            let seconds = seconds_str
                .parse::<u64>()
                .map_err(|_| IronDbError::Parse("SETEX seconds must be a number".into()))?;
            Ok(Command::SetEx {
                key: key.to_string(),
                seconds,
                value,
            })
        }
        "SAVE" => {
            if parts.next().is_some() {
                return Err(IronDbError::Parse("SAVE takes no arguments".into()));
            }
            Ok(Command::Save)
        }
        "LOAD" => {
            if parts.next().is_some() {
                return Err(IronDbError::Parse("LOAD takes no arguments".into()));
            }
            Ok(Command::Load)
        }
        "INFO" => {
            if parts.next().is_some() {
                return Err(IronDbError::Parse("INFO takes no arguments".into()));
            }
            Ok(Command::Info)
        }
        other => Err(IronDbError::Parse(format!("unknown command: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping() {
        assert_eq!(parse_command("PING").unwrap(), Command::Ping);
    }

    #[test]
    fn parses_set_with_spaces_in_value() {
        assert_eq!(
            parse_command("SET name Junan Kim").unwrap(),
            Command::Set {
                key: "name".into(),
                value: "Junan Kim".into(),
            }
        );
    }

    #[test]
    fn rejects_set_without_value() {
        assert!(parse_command("SET onlykey").is_err());
    }

    #[test]
    fn rejects_get_without_key() {
        assert!(parse_command("GET").is_err());
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse_command("UNKNOWN key").is_err());
    }

    #[test]
    fn rejects_expire_with_non_numeric_seconds() {
        assert!(parse_command("EXPIRE key abc").is_err());
    }
}
