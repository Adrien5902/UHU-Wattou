use crate::Context;
use color_eyre::Result;

#[poise::command(slash_command)]
pub async fn clear(ctx: Context<'_>, limit: u8) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let messages = ctx
        .http()
        .get_messages(ctx.channel_id(), None, Some(limit))
        .await?;

    for message in messages {
        message.delete(ctx).await?;
    }

    ctx.say(format!("{} messages supprimé(s)", limit)).await?;

    Ok(())
}
