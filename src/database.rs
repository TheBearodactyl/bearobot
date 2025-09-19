use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, migrate::MigrateDatabase};

#[derive(Debug, Serialize, Deserialize)]
pub struct SongSuggestion {
    pub id: i64,
    pub song_name: String,
    pub artist: String,
    pub suggested_by_id: String,
    pub suggested_by_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSuggestion {
    pub id: i64,
    pub game_name: String,
    pub developer: String,
    pub suggested_by_id: String,
    pub suggested_by_name: String,
    pub created_at: DateTime<Utc>,
}

#[tracing::instrument]
pub async fn init_database(database_path: &str) -> Result<SqlitePool> {
    tracing::info!("Initializing database at: {}", database_path);

    if !sqlx::Sqlite::database_exists(database_path)
        .await
        .unwrap_or(false)
    {
        tracing::info!("Database doesn't exist, creating: {}", database_path);
        sqlx::Sqlite::create_database(database_path)
            .await
            .with_context(|| format!("Failed to create database at {}", database_path))?;
    }

    let database_url = if database_path.starts_with("sqlite://") {
        database_path.to_string()
    } else {
        format!("sqlite://{}", database_path)
    };

    let pool = SqlitePool::connect(&database_url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", database_path))?;

    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Database initialized successfully with all migrations applied");
    Ok(pool)
}

#[tracing::instrument]
pub async fn save_song_suggestion(
    pool: &SqlitePool,
    song_name: &str,
    artist: &str,
    suggested_by_id: &str,
    suggested_by_name: &str,
) -> Result<i64> {
    tracing::debug!(
        song_name = %song_name,
        artist = %artist,
        user_id = %suggested_by_id,
        user_name = %suggested_by_name,
        "Saving song suggestion to database"
    );

    let result = sqlx::query!(
        "INSERT INTO song_suggestions (song_name, artist, suggested_by_id, suggested_by_name) VALUES (?, ?, ?, ?)",
        song_name,
        artist,
        suggested_by_id,
        suggested_by_name
    )
    .execute(pool)
    .await
    .context("Failed to save song suggestion")?;

    tracing::info!(
        suggestion_id = %result.last_insert_rowid(),
        song_name = %song_name,
        artist = %artist,
        "Song suggestion saved successfully"
    );

    Ok(result.last_insert_rowid())
}

#[tracing::instrument]
pub async fn save_game_suggestion(
    pool: &SqlitePool,
    game_name: &str,
    developer: &str,
    suggested_by_id: &str,
    suggested_by_name: &str,
) -> Result<i64> {
    tracing::debug!(
        game_name = %game_name,
        developer = %developer,
        user_id = %suggested_by_id,
        user_name = %suggested_by_name,
        "Saving game suggestion to database"
    );

    let result = sqlx::query!(
        "INSERT INTO game_suggestions (game_name, developer, suggested_by_id, suggested_by_name) VALUES (?, ?, ?, ?)",
        game_name,
        developer,
        suggested_by_id,
        suggested_by_name
    )
    .execute(pool)
    .await
    .context("Failed to save game suggestion")?;

    tracing::info!(
        suggestion_id = %result.last_insert_rowid(),
        song_name = %game_name,
        artist = %developer,
        "Game suggestion saved successfully"
    );

    Ok(result.last_insert_rowid())
}

#[tracing::instrument]
pub async fn get_song_suggestions(
    pool: &SqlitePool,
    limit: Option<i32>,
) -> Result<Vec<SongSuggestion>> {
    let limit = limit.unwrap_or(50);

    tracing::debug!(limit = %limit, "Fetching song suggestions from database");

    let rows = sqlx::query!(
        "SELECT id, song_name, artist, suggested_by_id, suggested_by_name, created_at 
         FROM song_suggestions ORDER BY created_at DESC LIMIT ?",
        limit
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch song suggestions")?;

    let suggestions: Vec<SongSuggestion> = rows
        .into_iter()
        .map(|row| SongSuggestion {
            id: row.id.unwrap_or(0),
            song_name: row.song_name,
            artist: row.artist,
            suggested_by_id: row.suggested_by_id,
            suggested_by_name: row.suggested_by_name,
            created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0)
                .unwrap_or_else(Utc::now),
        })
        .collect();

    tracing::debug!(count = %suggestions.len(), "Fetched song suggestions successfully");
    Ok(suggestions)
}

#[tracing::instrument]
pub async fn get_game_suggestions(
    pool: &SqlitePool,
    limit: Option<i32>,
) -> Result<Vec<GameSuggestion>> {
    let limit = limit.unwrap_or(50);

    tracing::debug!(limit = %limit, "Fetching game suggestions from database");

    let rows = sqlx::query!(
        "SELECT id, game_name, developer, suggested_by_id, suggested_by_name, created_at 
         FROM game_suggestions ORDER BY created_at DESC LIMIT ?",
        limit
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch game suggestions")?;

    let suggestions: Vec<GameSuggestion> = rows
        .into_iter()
        .map(|row| GameSuggestion {
            id: row.id.unwrap_or(0),
            game_name: row.game_name,
            developer: row.developer,
            suggested_by_id: row.suggested_by_id,
            suggested_by_name: row.suggested_by_name,
            created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0)
                .unwrap_or_else(Utc::now),
        })
        .collect();

    tracing::debug!(count = %suggestions.len(), "Fetched game suggestions successfully");
    Ok(suggestions)
}

#[tracing::instrument]
pub async fn get_song_suggestions_by_user(
    pool: &SqlitePool,
    user_id: &str,
    limit: Option<i32>,
) -> Result<Vec<SongSuggestion>> {
    let limit = limit.unwrap_or(10);

    tracing::debug!(user_id = %user_id, limit = %limit, "Fetching song suggestions by user");

    let rows = sqlx::query!(
        "SELECT id, song_name, artist, suggested_by_id, suggested_by_name, created_at
         FROM song_suggestions
         WHERE suggested_by_id = ?
         ORDER BY created_at DESC
         LIMIT ?",
        user_id,
        limit
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch song suggestions by user")?;

    let suggestions: Vec<SongSuggestion> = rows
        .into_iter()
        .map(|row| SongSuggestion {
            id: row.id.unwrap_or(0),
            song_name: row.song_name,
            artist: row.artist,
            suggested_by_id: row.suggested_by_id,
            suggested_by_name: row.suggested_by_name,
            created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0)
                .unwrap_or_else(Utc::now),
        })
        .collect();

    tracing::info!(
        user_id = %user_id,
        count = %suggestions.len(),
        "Fetched user song suggestions successfully"
    );

    Ok(suggestions)
}

#[tracing::instrument]
pub async fn get_game_suggestions_by_user(
    pool: &SqlitePool,
    user_id: &str,
    limit: Option<i32>,
) -> Result<Vec<GameSuggestion>> {
    let limit = limit.unwrap_or(10);

    tracing::debug!(user_id = %user_id, limit = %limit, "Fetching game suggestions by user");

    let rows = sqlx::query!(
        "SELECT id, game_name, developer, suggested_by_id, suggested_by_name, created_at
         FROM game_suggestions
         WHERE suggested_by_id = ?
         ORDER BY created_at DESC
         LIMIT ?",
        user_id,
        limit
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch game suggestions by user")?;

    let suggestions: Vec<GameSuggestion> = rows
        .into_iter()
        .map(|row| GameSuggestion {
            id: row.id.unwrap_or(0),
            game_name: row.game_name,
            developer: row.developer,
            suggested_by_id: row.suggested_by_id,
            suggested_by_name: row.suggested_by_name,
            created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0)
                .unwrap_or_else(Utc::now),
        })
        .collect();

    tracing::info!(
        user_id = %user_id,
        count = %suggestions.len(),
        "Fetched user game suggestions successfully"
    );

    Ok(suggestions)
}

#[tracing::instrument]
pub async fn delete_song_suggestion(
    pool: &SqlitePool,
    suggestion_id: i64,
    user_id: &str,
) -> Result<bool> {
    tracing::debug!(
        suggestion_id = %suggestion_id,
        user_id = %user_id,
        "Attempting to delete song suggestion"
    );

    let result = sqlx::query!(
        "DELETE FROM song_suggestions WHERE id = ? AND suggested_by_id = ?",
        suggestion_id,
        user_id
    )
    .execute(pool)
    .await
    .context("Failed to delete song suggestion")?;

    let deleted = result.rows_affected() > 0;

    if deleted {
        tracing::info!(
            suggestion_id = %suggestion_id,
            user_id = %user_id,
            "Song suggestion deleted successfully"
        );
    } else {
        tracing::warn!(
            suggestion_id = %suggestion_id,
            user_id = %user_id,
            "Song suggestion not found or user not authorized to delete"
        );
    }

    Ok(deleted)
}

#[tracing::instrument]
pub async fn delete_game_suggestion(
    pool: &SqlitePool,
    suggestion_id: i64,
    user_id: &str,
) -> Result<bool> {
    tracing::debug!(
        suggestion_id = %suggestion_id,
        user_id = %user_id,
        "Attempting to delete game suggestion"
    );

    let result = sqlx::query!(
        "DELETE FROM game_suggestions WHERE id = ? AND suggested_by_id = ?",
        suggestion_id,
        user_id
    )
    .execute(pool)
    .await
    .context("Failed to delete song suggestion")?;

    let deleted = result.rows_affected() > 0;

    if deleted {
        tracing::info!(
            suggestion_id = %suggestion_id,
            user_id = %user_id,
            "Game suggestion deleted successfully"
        );
    } else {
        tracing::warn!(
            suggestion_id = %suggestion_id,
            user_id = %user_id,
            "Game suggestion not found or user not authorized to delete"
        );
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_operations() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let suggestion_id =
            save_song_suggestion(&pool, "Test Song", "Test Artist", "123456789", "TestUser")
                .await?;

        assert!(suggestion_id > 0);

        let suggestions = get_song_suggestions(&pool, None).await?;
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].song_name, "Test Song");

        let user_suggestions = get_song_suggestions_by_user(&pool, "123456789", None).await?;
        assert_eq!(user_suggestions.len(), 1);

        let deleted = delete_song_suggestion(&pool, suggestion_id, "123456789").await?;
        assert!(deleted);

        let suggestions_after_delete = get_song_suggestions(&pool, None).await?;
        assert_eq!(suggestions_after_delete.len(), 0);

        Ok(())
    }
}
