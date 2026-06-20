use crate::{
    Context,
    data::guild::{GuildData, SavedData},
    debug,
    recurrent_message::ToutesLesCollesMessage,
};
use color_eyre::Result;

#[poise::command(slash_command, guild_only)]
pub async fn toutes_les_colles(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let data = GuildData::from_ctx(ctx)?;
    let handle = ctx.say(data.prochaines_colles_msg()?).await?;

    let message = handle.message().await?;
    ToutesLesCollesMessage::from(&message).save(data.guild_id)?;
    debug!(
        "new toutes les colles msg : {} {}",
        message.id, message.channel_id,
    );
    Ok(())
}
