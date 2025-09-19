mod admin;
mod games;
mod music;

use crate::error::{Context, Error, Result};

pub use admin::*;
pub use games::*;
pub use music::*;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "request_song",
        "list_songs",
        "my_song_requests",
        "delete_song_request",
        "request_game",
        "list_games",
        "my_game_requests",
        "delete_game_request"
    ),
    subcommand_required,
    category = "Misc",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn suggest(_: Context<'_>) -> Result<()> {
    Ok(())
}

pub fn get_commands() -> Vec<poise::Command<crate::bot::Data, Error>> {
    vec![suggest(), admin()]
}
