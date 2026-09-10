use crate::{bot::Context, data::guild::SemaineTPMessage};
use color_eyre::eyre::Result;

#[poise::command(slash_command, guild_only)]
pub async fn semaine_tp(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_mut_from_ctx(ctx)?;
    guild_data.mutable
        .semaine_tp_msg
        .set_from_ctx(&guild_data.persistent, SemaineTPMessage, ctx)
        .await?;
    guild_data.save()?;
    Ok(())
}
