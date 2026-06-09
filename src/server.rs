use crate::config::Config;
use crate::error::Result;

pub async fn run(_config: Config) -> Result<()> {
    tracing::info!("IronDB server is not implemented yet (Phase 3)");
    Ok(())
}
