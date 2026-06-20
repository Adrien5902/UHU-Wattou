use crate::{
    Context,
    data::guild::{GuildData, SavedData},
    debug,
    recurrent_message::SemaineTPMessage,
};
use color_eyre::Result;

#[poise::command(slash_command, guild_only)]
pub async fn semaine_tp(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let data = GuildData::from_ctx(ctx)?;

    let handle = ctx.say(data.semaine_tp_msg()).await?;

    let message = handle.message().await?;
    SemaineTPMessage::from(&message).save(data.guild_id)?;
    debug!("new semaine tp msg : {} {}", message.id, message.channel_id);
    Ok(())
}
