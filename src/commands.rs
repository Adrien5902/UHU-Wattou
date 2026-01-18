use super::Context;
use crate::{
    colle::ColleStringFormat,
    debug,
    group::GroupId,
    guild_data::{GuildData, SavedData, SemaineTPMessage, ToutesLesCollesMessage},
    subscriber::SubscriberData,
};
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
    ToutesLesCollesMessage::from(&message).save(data.guild_id)?;
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
    SemaineTPMessage::from(&message).save(data.guild_id)?;
    debug!("new semaine tp msg : {} {}", message.id, message.channel_id);
    Ok(())
}

#[poise::command(slash_command)]
pub async fn rappel(
    ctx: Context<'_>,
    #[description = "Groupe de colle"]
    #[rename = "groupe"]
    group: usize,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let data = GuildData::from_ctx(ctx)?;

    let user_id = ctx.author().id;
    let mut subscribers = data.subscribers()?;
    if let Some(current) = subscribers.get(&user_id) {
        if current.group_id == group {
            let c = current.clone();
            subscribers.remove(data.guild_id, &user_id)?;

            ctx.say(format!("Rappel désactivé")).await?;

            debug!(
                "{} unsubscribed from group reminders {:?}",
                ctx.author().id,
                c
            );
            return Ok(());
        }

        ctx.say(format!("Rappel désactivé pour le groupe {}", group))
            .await?;
    }

    ctx.say(format!("Tu auras désormais un rappel de prendre ton carnet de colle à chaque fois que le groupe {} a colle d'anglais !\nRefais la commande pour désactiver", group)).await?;

    subscribers.set(data.guild_id, user_id, SubscriberData::new_default(group))?;

    debug!(
        "{} subscribed to group reminders {}",
        ctx.author().id,
        group
    );

    Ok(())
}
