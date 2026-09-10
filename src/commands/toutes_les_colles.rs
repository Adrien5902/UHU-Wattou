use crate::{bot::Context, data::guild::ToutesLesCollesMessage};
use color_eyre::eyre::Result;

#[poise::command(slash_command, guild_only)]
pub async fn toutes_les_colles(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_mut_from_ctx(ctx)?;
    guild_data.mutable
        .toutes_les_colles_msg
        .set_from_ctx(&guild_data.persistent, ToutesLesCollesMessage, ctx)
        .await?;
    guild_data.save()?;
    Ok(())
}
