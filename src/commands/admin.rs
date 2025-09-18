use crate::error::{Context, Result, bot_error};
use chrono::{Duration, Utc};
use poise::serenity_prelude::{ChannelId, Permissions};
use regex::Regex;
use serenity::all::GetMessages;

#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("purge"),
    subcommand_required,
    category = "Admin",
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES"
)]
pub async fn admin(_: Context<'_>) -> Result<()> {
    Ok(())
}

#[tracing::instrument]
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES"
)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Regex pattern to match messages"] pattern: String,
    #[description = "Channel to purge (defaults to current)"] channel: Option<ChannelId>,
    #[description = "Duration in minutes (e.g., 60 for 1 hour)"] duration_minutes: Option<i64>,
) -> Result<()> {
    tracing::info!(
        user_id = %ctx.author().id,
        user_name = %ctx.author().name,
        guild_id = ?ctx.guild_id(),
        pattern = %pattern,
        channel_id = ?channel,
        duration_minutes = ?duration_minutes,
        "Admin purge command invoked"
    );

    #[allow(deprecated)]
    if let Some(guild_id) = ctx.guild_id() {
        let member = guild_id.member(&ctx.http(), ctx.author().id).await?;
        let permissions = member.permissions(ctx.cache())?;

        if !permissions.contains(Permissions::MANAGE_MESSAGES) {
            tracing::warn!(
                user_id = %ctx.author().id,
                "User attempted purge without MANAGE_MESSAGES permission"
            );
            return Err(bot_error(
                "You need the 'Manage Messages' permission to use this command",
            ));
        }
    }

    let target_channel = channel.unwrap_or_else(|| ctx.channel_id());
    let duration_minutes = duration_minutes.unwrap_or(60);

    if duration_minutes <= 0 || duration_minutes > 10080 {
        return Err(bot_error(
            "Duration must be between 1 minute and 1 week (10080 minutes)",
        ));
    }

    let regex =
        Regex::new(&pattern).map_err(|e| bot_error(format!("Invalid regex pattern: {}", e)))?;

    tracing::info!(
        pattern = %pattern,
        channel_id = %target_channel,
        duration_minutes = %duration_minutes,
        "Starting message purge operation"
    );

    let response = ctx.say("Starting the purge...").await?;

    let time_threshold = Utc::now() - Duration::minutes(duration_minutes);
    let mut messages_to_delete = Vec::new();
    let mut total_checked = 0u32;
    let mut last_message_id = None;

    tracing::debug!("Collecting victims for the purge");

    loop {
        let mut builder = GetMessages::new().limit(100);

        if let Some(before_id) = last_message_id {
            builder = builder.before(before_id);
        }

        let messages = target_channel.messages(&ctx.http(), builder).await?;

        if messages.is_empty() {
            tracing::debug!("No more messages to check");
            break;
        }

        let mut found_old_message = false;
        for message in messages {
            total_checked += 1;

            if message.timestamp.timestamp() < time_threshold.timestamp() {
                found_old_message = true;
                tracing::debug!(
                    message_id = %message.id,
                    "Message is older than threshold, stopping collection"
                );
                break;
            }

            if regex.is_match(&message.content) {
                messages_to_delete.push(message.id);
                tracing::debug!(
                    message_id = %message.id,
                    content_preview = %message.content.chars().take(50).collect::<String>(),
                    "Message matched pattern and will be deleted"
                );
            }

            last_message_id = Some(message.id);
        }

        if found_old_message {
            break;
        }

        if total_checked.is_multiple_of(500) {
            response
                .edit(
                    ctx,
                    poise::CreateReply::default().content(format!(
                        "Scanning messages... Checked: {}, Found: {}",
                        total_checked,
                        messages_to_delete.len()
                    )),
                )
                .await?;
        }
    }

    tracing::info!(
        total_checked = %total_checked,
        messages_to_delete = %messages_to_delete.len(),
        "Message collection completed"
    );

    if messages_to_delete.is_empty() {
        response.edit(ctx, poise::CreateReply::default().content(format!(
            "Purge completed! No messages matched the pattern.\n**Checked:** {} messages\n**Pattern:** `{}`",
            total_checked, pattern
        ))).await?;
        return Ok(());
    }

    response
        .edit(
            ctx,
            poise::CreateReply::default()
                .content(format!("Deleting {} messages...", messages_to_delete.len())),
        )
        .await?;

    let mut deleted_count = 0;
    let mut failed_deletes = 0;

    let two_weeks_ago = Utc::now() - Duration::days(14);
    let mut bulk_delete_ids = Vec::new();
    let mut individual_delete_ids = Vec::new();

    for &message_id in &messages_to_delete {
        let message_timestamp = message_id.created_at();
        if message_timestamp.timestamp() > two_weeks_ago.timestamp() {
            bulk_delete_ids.push(message_id);
        } else {
            individual_delete_ids.push(message_id);
        }
    }

    tracing::info!(
        bulk_delete_count = %bulk_delete_ids.len(),
        individual_delete_count = %individual_delete_ids.len(),
        "Starting message deletion"
    );

    for chunk in bulk_delete_ids.chunks(100) {
        match target_channel.delete_messages(&ctx.http(), chunk).await {
            Ok(_) => {
                deleted_count += chunk.len();
                tracing::debug!(count = %chunk.len(), "Bulk deleted messages");
            }
            Err(e) => {
                failed_deletes += chunk.len();
                tracing::warn!(error = %e, count = %chunk.len(), "Failed to bulk delete messages");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    for &message_id in &individual_delete_ids {
        match target_channel
            .delete_message(&ctx.http(), message_id)
            .await
        {
            Ok(_) => {
                deleted_count += 1;
                tracing::debug!(message_id = %message_id, "Individually deleted message");
            }
            Err(e) => {
                failed_deletes += 1;
                tracing::warn!(error = %e, message_id = %message_id, "Failed to delete message");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        if (deleted_count + failed_deletes) % 10 == 0 {
            response
                .edit(
                    ctx,
                    poise::CreateReply::default().content(format!(
                        "Deleting messages... Progress: {}/{}",
                        deleted_count + failed_deletes,
                        messages_to_delete.len()
                    )),
                )
                .await?;
        }
    }

    let final_message = if failed_deletes > 0 {
        format!(
            "Purge completed with some failures!\n**Deleted:** {}\n**Failed:** {}\n**Total checked:** {}\n**Pattern:** `{}`\n**Duration:** {} minutes",
            deleted_count, failed_deletes, total_checked, pattern, duration_minutes
        )
    } else {
        format!(
            "Purge completed successfully!\n**Deleted:** {}\n**Total checked:** {}\n**Pattern:** `{}`\n**Duration:** {} minutes",
            deleted_count, total_checked, pattern, duration_minutes
        )
    };

    response
        .edit(ctx, poise::CreateReply::default().content(final_message))
        .await?;

    tracing::info!(
        deleted_count = %deleted_count,
        failed_deletes = %failed_deletes,
        total_checked = %total_checked,
        pattern = %pattern,
        duration_minutes = %duration_minutes,
        "Purge operation completed"
    );

    Ok(())
}
