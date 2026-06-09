use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "irondb", about = "Redis-inspired in-memory database server")]
pub struct Config {
    #[arg(long, default_value_t = 6379)]
    pub port: u16,

    #[arg(long)]
    pub snapshot: Option<String>,
}
