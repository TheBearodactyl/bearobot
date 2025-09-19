use crate::database;
use crate::error::{Context, Result, bot_error};

/// Makes a game request entry for me to play later
#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Games"
)]
pub async fn request_game(
    ctx: Context<'_>,
    #[description = "The name of the game"] game_name: Option<String>,
    #[description = "The game developer"] developer: Option<String>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        user_name = %ctx.author().name,
        guild_id = ?ctx.guild_id(),
        "Game suggestion command invoked"
    );

    let game_name = game_name
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bot_error("Game name cannot be empty"))?;

    let developer = developer
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bot_error("Developer name cannot be empty"))?;

    let suggestion_id = database::save_game_suggestion(
        &ctx.data().database,
        &game_name,
        &developer,
        &ctx.author().id.to_string(),
        &ctx.author().name,
    )
    .await?;

    let response = format!(
        "**Game Suggestion #{suggestion_id}** \n**Game:** {game_name}\n**Developer:** {developer}\n**Suggested by:** {}",
        ctx.author().name
    );

    ctx.say(response).await?;

    tracing::info!(
        suggestion_id = %suggestion_id,
        game_name = %game_name,
        developer = %developer,
        user_id = %ctx.author().id,
        "Game suggestion saved successfully"
    );

    Ok(())
}

/// List all active game requests
#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Games"
)]
pub async fn list_games(
    ctx: Context<'_>,
    #[description = "Number of suggestions to show (max 50)"] limit: Option<i32>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        limit = ?limit,
        "List suggestions command invoked"
    );

    let limit = limit.map(|l| l.clamp(1, 50));
    let suggestions = database::get_game_suggestions(&ctx.data().database, limit).await?;

    if suggestions.is_empty() {
        ctx.say("No game suggestions found! Be the first to suggest a game with `/suggest game`.")
            .await?;
        return Ok(());
    }

    let mut response = format!("**Latest {} Game Suggestions**\n\n", suggestions.len());

    for (index, suggestion) in suggestions.iter().enumerate().take(10) {
        response.push_str(&format!(
            "**{}. {}** developed by {}\n   *Suggested by {} (ID: {})*\n\n",
            index + 1,
            suggestion.game_name,
            suggestion.developer,
            suggestion.suggested_by_name,
            suggestion.id
        ));
    }

    if suggestions.len() > 10 {
        response.push_str(&format!(
            "*... and {} more suggestions*",
            suggestions.len() - 10
        ));
    }

    ctx.say(response).await?;

    tracing::info!(
        user_id = %ctx.author().id,
        suggestions_count = %suggestions.len(),
        "List suggestions completed"
    );

    Ok(())
}

/// List all game requests made by you
#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Games"
)]
pub async fn my_game_requests(
    ctx: Context<'_>,
    #[description = "Number of your suggestions to show (max 20)"] limit: Option<i32>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        limit = ?limit,
        "My suggestions command invoked"
    );

    let limit = limit.map(|l| l.clamp(1, 20));
    let suggestions = database::get_game_suggestions_by_user(
        &ctx.data().database,
        &ctx.author().id.to_string(),
        limit,
    )
    .await?;

    if suggestions.is_empty() {
        ctx.say("You haven't suggested any games yet! Use `/suggest game` to add your first suggestion.").await?;
        return Ok(());
    }

    let mut response = format!("**Your {} Game Suggestions**\n\n", suggestions.len());

    for (index, suggestion) in suggestions.iter().enumerate() {
        response.push_str(&format!(
            "**{}. {}** developed by {}\n   *Suggested on {} (ID: {})*\n\n",
            index + 1,
            suggestion.game_name,
            suggestion.developer,
            suggestion.created_at.format("%Y-%m-%d %H:%M UTC"),
            suggestion.id
        ));
    }

    ctx.say(response).await?;

    tracing::info!(
        user_id = %ctx.author().id,
        suggestions_count = %suggestions.len(),
        "My suggestions completed"
    );

    Ok(())
}

/// Delete a game request based on its ID
#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Games"
)]
pub async fn delete_game_request(
    ctx: Context<'_>,
    #[description = "ID of the suggestion to delete"] suggestion_id: i64,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        suggestion_id = %suggestion_id,
        "Delete suggestion command invoked"
    );

    let deleted = database::delete_game_suggestion(
        &ctx.data().database,
        suggestion_id,
        &ctx.author().id.to_string(),
    )
    .await?;

    if deleted {
        ctx.say(format!(
            "Successfully deleted suggestion #{}",
            suggestion_id
        ))
        .await?;
        tracing::info!(
            user_id = %ctx.author().id,
            suggestion_id = %suggestion_id,
            "Game suggestion deleted successfully"
        );
    } else {
        ctx.say("Suggestion not found or you don't have permission to delete it.")
            .await?;
        tracing::warn!(
            user_id = %ctx.author().id,
            suggestion_id = %suggestion_id,
            "Failed to delete suggestion - not found or unauthorized"
        );
    }

    Ok(())
}
