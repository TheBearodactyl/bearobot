use crate::{commands, config::Config, database, error::handle_error};
use anyhow::Result;
use poise::serenity_prelude::{Client, ClientBuilder};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Data {
    pub database: SqlitePool,
}

impl Data {
    pub fn new(database: SqlitePool) -> Self {
        tracing::debug!("Creating new bot data instance");
        Self { database }
    }
}

#[tracing::instrument]
pub async fn create_bot(config: Config) -> Result<Client> {
    tracing::info!("Creating bot with configuration");
    tracing::debug!(prefix = %config.command_prefix, "Bot command prefix configured");

    let database = database::init_database(
        config
            .database_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid database path"))?,
    )
    .await?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config.command_prefix.clone()),
                ..Default::default()
            },
            on_error: |error| Box::pin(handle_error(error)),
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                tracing::info!(
                    bot_name = %ready.user.name,
                    bot_id = %ready.user.id,
                    guild_count = %ready.guilds.len(),
                    "Bot successfully logged in"
                );

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tracing::info!("Global commands registered successfully");

                Ok(Data::new(database))
            })
        })
        .build();

    let client = ClientBuilder::new(config.discord_token, config.intents)
        .framework(framework)
        .await?;

    tracing::info!("Bot client created successfully");
    Ok(client)
}
