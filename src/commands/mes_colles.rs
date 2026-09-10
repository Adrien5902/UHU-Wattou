use crate::{
    bot::Context,
    data::{colle::ColleStringFormat, group::GroupId, resolve::Resolve},
};
use color_eyre::Result;
use poise::CreateReply;
use std::fmt::Write;

#[poise::command(slash_command, guild_only)]
pub async fn mes_colles(
    ctx: Context<'_>,
    #[description = "Groupe de colle"]
    #[rename = "groupe"]
    group_id: GroupId,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_from_ctx(ctx)?;
    let group = guild_data.get_group(group_id)?;

    let mut content = String::new();
    content.write_fmt(format_args!(
        "Prochaines colles pour le groupe {}:",
        group.id
    ))?;

    let resolved = group.resolve(&guild_data.persistent)?;
    for colle in resolved.get_next_colles(5) {
        let resolved_colle = colle.resolve(&guild_data.persistent)?;
        content.push_str("\n- ");
        resolved_colle.format(
            &mut content,
            ColleStringFormat::Explicit,
            &guild_data.persistent,
        )?;
    }

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(content)
            .reply(true),
    )
    .await?;
    Ok(())
}
