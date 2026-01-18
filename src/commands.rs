use super::Context;
use crate::debug;
use crate::{colle::ColleStringFormat, group::GroupId, guild_data::GuildData};
use anyhow::Result;
use poise::CreateReply;
use serenity::all::CreateAttachment;

#[poise::command(slash_command, guild_only)]
pub async fn mes_colles(
    ctx: Context<'_>,
    #[description = "Groupe de colle"]
    #[rename = "groupe"]
    group_id: GroupId,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let guild_data = GuildData::from_ctx(ctx)?;
    let group = guild_data.get_group(group_id)?;

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(format!(
                "Prochaines colles pour le groupe {}: \n- {}",
                group.id,
                group
                    .get_next_colles(5)
                    .iter()
                    .map(|colle| colle.to_string(ColleStringFormat::Explicit))
                    .collect::<Vec<_>>()
                    .join("\n- ")
            ))
            .reply(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn colles_calendrier(
    ctx: Context<'_>,
    #[description = "Groupe de colle"]
    #[rename = "groupe"]
    group_id: GroupId,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let data = GuildData::from_ctx(ctx)?;

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .attachment(CreateAttachment::bytes(
                data.get_group(group_id)?.ics_calendar()?,
                format!("Calendrier de colles group {}.ics", group_id),
            ))
            .content("Importe le fichier dans ton calendrier pour y ajouter les colles !"),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn toutes_les_colles(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let data = GuildData::from_ctx(ctx)?;
    let handle = ctx.say(data.prochaines_colles_msg()).await?;

    let message = handle.message().await?;
    debug!(
        "new toutes les colles msg : {} {}",
        message.id, message.channel_id,
    );
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn semaine_tp(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let data = GuildData::from_ctx(ctx)?;

    let handle = ctx.say(data.semaine_tp_msg()).await?;

    let message = handle.message().await?;
    debug!("new semaine tp msg : {} {}", message.id, message.channel_id,);
    Ok(())
}
