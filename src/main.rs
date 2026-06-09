use clap::Parser;
use irondb::config::Config;
use irondb::server;

#[tokio::main]
async fn main() -> irondb::error::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::parse();
    tracing::info!(port = config.port, "starting IronDB");
    server::run(config).await
}
