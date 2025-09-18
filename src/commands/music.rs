use crate::database;
use crate::error::{Context, Result, bot_error};

#[tracing::instrument]
#[poise::command(prefix_command, slash_command)]
pub async fn song(
    ctx: Context<'_>,
    #[description = "The name of the song"] song_name: Option<String>,
    #[description = "The song artist/band"] artist: Option<String>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        user_name = %ctx.author().name,
        guild_id = ?ctx.guild_id(),
        "Song suggestion command invoked"
    );

    let song_name = song_name
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bot_error("Song name cannot be empty"))?;

    let artist = artist
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| bot_error("Artist name cannot be empty"))?;

    let suggestion_id = database::save_song_suggestion(
        &ctx.data().database,
        &song_name,
        &artist,
        &ctx.author().id.to_string(),
        &ctx.author().name,
    )
    .await?;

    let response = format!(
        "**Song Suggestion #{suggestion_id}** \n**Song:** {song_name}\n**Artist:** {artist}\n**Suggested by:** {}",
        ctx.author().name
    );

    ctx.say(response).await?;

    tracing::info!(
        suggestion_id = %suggestion_id,
        song_name = %song_name,
        artist = %artist,
        user_id = %ctx.author().id,
        "Song suggestion saved successfully"
    );

    Ok(())
}

#[tracing::instrument]
#[poise::command(prefix_command, slash_command)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Number of suggestions to show (max 50)"] limit: Option<i32>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        limit = ?limit,
        "List suggestions command invoked"
    );

    let limit = limit.map(|l| l.clamp(1, 50));
    let suggestions = database::get_song_suggestions(&ctx.data().database, limit).await?;

    if suggestions.is_empty() {
        ctx.say(
            "No song suggestions found! Be the first to suggest a song with `/suggest song`.",
        )
        .await?;
        return Ok(());
    }

    let mut response = format!(
        "**Latest {} Song Suggestions** 🎵\n\n",
        suggestions.len()
    );

    for (index, suggestion) in suggestions.iter().enumerate().take(10) {
        response.push_str(&format!(
            "**{}. {}** by {}\n   *Suggested by {} (ID: {})*\n\n",
            index + 1,
            suggestion.song_name,
            suggestion.artist,
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

#[tracing::instrument]
#[poise::command(prefix_command, slash_command)]
pub async fn my_suggestions(
    ctx: Context<'_>,
    #[description = "Number of your suggestions to show (max 20)"] limit: Option<i32>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        limit = ?limit,
        "My suggestions command invoked"
    );

    let limit = limit.map(|l| l.clamp(1, 20));
    let suggestions = database::get_suggestions_by_user(
        &ctx.data().database,
        &ctx.author().id.to_string(),
        limit,
    )
    .await?;

    if suggestions.is_empty() {
        ctx.say("You haven't suggested any songs yet! Use `/suggest song` to add your first suggestion.").await?;
        return Ok(());
    }

    let mut response = format!("**Your {} Song Suggestions** 🎵\n\n", suggestions.len());

    for (index, suggestion) in suggestions.iter().enumerate() {
        response.push_str(&format!(
            "**{}. {}** by {}\n   *Suggested on {} (ID: {})*\n\n",
            index + 1,
            suggestion.song_name,
            suggestion.artist,
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

#[tracing::instrument]
#[poise::command(prefix_command, slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "ID of the suggestion to delete"] suggestion_id: i64,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        suggestion_id = %suggestion_id,
        "Delete suggestion command invoked"
    );

    let deleted = database::delete_song_suggestion(
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
            "Song suggestion deleted successfully"
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
