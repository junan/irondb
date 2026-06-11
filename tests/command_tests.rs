use irondb::command::{parse_command, Command};
use irondb::protocol::{format_response, Response};

#[test]
fn parses_mvp_commands() {
    assert_eq!(parse_command("PING").unwrap(), Command::Ping);
    assert_eq!(
        parse_command("SET name Junan").unwrap(),
        Command::Set {
            key: "name".into(),
            value: "Junan".into(),
        }
    );
    assert_eq!(
        parse_command("GET name").unwrap(),
        Command::Get { key: "name".into() }
    );
    assert_eq!(
        parse_command("DEL name").unwrap(),
        Command::Del { key: "name".into() }
    );
    assert_eq!(
        parse_command("EXISTS name").unwrap(),
        Command::Exists { key: "name".into() }
    );
}

#[test]
fn parses_ttl_commands() {
    assert_eq!(
        parse_command("EXPIRE session 60").unwrap(),
        Command::Expire {
            key: "session".into(),
            seconds: 60,
        }
    );
    assert_eq!(
        parse_command("TTL session").unwrap(),
        Command::Ttl {
            key: "session".into()
        }
    );
    assert_eq!(
        parse_command("SETEX session 60 abc123").unwrap(),
        Command::SetEx {
            key: "session".into(),
            seconds: 60,
            value: "abc123".into(),
        }
    );
}

#[test]
fn parses_admin_commands() {
    assert_eq!(parse_command("SAVE").unwrap(), Command::Save);
    assert_eq!(parse_command("LOAD").unwrap(), Command::Load);
    assert_eq!(parse_command("INFO").unwrap(), Command::Info);
}

#[test]
fn rejects_malformed_commands() {
    assert!(parse_command("SET onlykey").is_err());
    assert!(parse_command("GET").is_err());
    assert!(parse_command("EXPIRE key abc").is_err());
    assert!(parse_command("UNKNOWN key").is_err());
}

#[test]
fn set_value_can_contain_spaces() {
    assert_eq!(
        parse_command("SET msg hello world").unwrap(),
        Command::Set {
            key: "msg".into(),
            value: "hello world".into(),
        }
    );
}

#[test]
fn formats_responses() {
    assert_eq!(format_response(&Response::Simple("PONG".into())), "PONG\n");
    assert_eq!(format_response(&Response::Bulk(None)), "(nil)\n");
    assert_eq!(
        format_response(&Response::Bulk(Some("Junan".into()))),
        "Junan\n"
    );
    assert_eq!(format_response(&Response::Integer(1)), "1\n");
    assert_eq!(
        format_response(&Response::Error("bad command".into())),
        "ERR bad command\n"
    );
}
