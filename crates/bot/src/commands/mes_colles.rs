use crate::{
    Context,
    data::{colle::ColleStringFormat, group::GroupId, guild::GuildData},
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
    let guild_data = GuildData::from_ctx(ctx)?;
    let group = guild_data.get_group(group_id)?;

    let mut content = String::new();
    content.write_fmt(format_args!(
        "Prochaines colles pour le groupe {}:",
        group.id
    ))?;
    for colle in group.get_next_colles(5) {
        content.push_str("\n- ");
        colle.format(&mut content, ColleStringFormat::Explicit)?;
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
