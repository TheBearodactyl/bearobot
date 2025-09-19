use anyhow::{Context, Result};
use serenity::all::GatewayIntents;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub intents: GatewayIntents,
    pub command_prefix: String,
    pub database_path: PathBuf,
}

impl Config {
    #[tracing::instrument]
    pub fn from_env() -> Result<Self> {
        tracing::info!("Loading .env variables...");
        dotenv::dotenv().ok();

        let discord_token = std::env::var("DISCORD_TOKEN")
            .context("DISCORD_TOKEN environment variable is required")?;

        let database_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "./db/bearobot.sqlite".to_string())
            .into();

        Ok(Self {
            discord_token,
            intents: GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
            command_prefix: ")".to_string(),
            database_path,
        })
    }
}
