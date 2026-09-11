use crate::bot::Context;
use color_eyre::Result;

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
pub async fn clear(ctx: Context<'_>, limit: u8) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let messages = ctx
        .http()
        .get_messages(ctx.channel_id(), None, Some(limit))
        .await?;

    ctx.channel_id().delete_messages(ctx.http(), messages.iter().map(|m| m.id)).await?;

    ctx.say(format!("{} messages supprimé(s)", limit)).await?;

    Ok(())
}
