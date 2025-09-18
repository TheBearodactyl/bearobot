mod bot;
mod commands;
mod config;
mod database;
mod error;

use anyhow::Result;
use bot::create_bot;
use config::Config;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .init();

    let config = Config::from_env()?;
    let mut client = create_bot(config).await?;

    client.start().await?;

    Ok(())
}
