use poise::serenity_prelude::RoleParseError;
use std::fmt;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, crate::bot::Data, Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct BotError {
    message: String,
}

impl BotError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BotError {}

pub fn bot_error(message: impl Into<String>) -> Error {
    Box::new(BotError::new(message))
}

#[tracing::instrument]
pub async fn handle_error(error: poise::FrameworkError<'_, crate::bot::Data, Error>) {
    match error {
        poise::FrameworkError::ArgumentParse { error, ctx, .. } => {
            tracing::warn!(
                command = %ctx.command().name,
                user_id = %ctx.author().id,
                error = %error,
                "Argument parse error"
            );

            if let Some(role_error) = error.downcast_ref::<RoleParseError>() {
                tracing::error!("Role parse error: {:?}", role_error);
            } else {
                tracing::error!("Argument parse error: {:?}", error);
            }
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(
                command = %ctx.command().name,
                user_id = %ctx.author().id,
                guild_id = ?ctx.guild_id(),
                channel_id = %ctx.channel_id(),
                error = %error,
                "Command execution error"
            );

            let response = "An error occurred while processing your command.";
            if let Err(e) = ctx.say(response).await {
                tracing::error!(error = %e, "Failed to send error message");
            }
        }
        poise::FrameworkError::Setup { error, .. } => {
            tracing::error!(error = %error, "Framework setup error");
        }
        other => {
            tracing::warn!(error = ?other, "Unhandled framework error");
            if let Err(e) = poise::builtins::on_error(other).await {
                tracing::error!(error = %e, "Error in error handler");
            }
        }
    }
}
