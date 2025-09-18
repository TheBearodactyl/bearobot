mod admin;
mod music;

use crate::error::{Context, Error, Result};

pub use admin::*;
pub use music::*;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("song", "list", "my_suggestions", "delete"),
    subcommand_required,
    category = "Misc"
)]
pub async fn suggest(_: Context<'_>) -> Result<()> {
    Ok(())
}

pub fn get_commands() -> Vec<poise::Command<crate::bot::Data, Error>> {
    vec![suggest(), admin()]
}
