use crate::{
    bot::Context,
    data::{guild::GuildData, subscriber::SubscriberData},
    debug,
};
use color_eyre::Result;

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
