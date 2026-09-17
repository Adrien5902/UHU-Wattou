use crate::{
    bot::Context,
    data::guild::PressReviewMessage,
    periodic_message::{OptionalPeriodicMessage, PeriodicMessage},
};
use color_eyre::eyre::Result;
use poise::CreateReply;

#[poise::command(slash_command, guild_only)]
pub async fn press_reviews(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_from_ctx(ctx)?;
    {
        let mut guild_data_mut = data.guild_mut_from_ctx(ctx).await?;
        guild_data_mut.press_review_msg = OptionalPeriodicMessage(Some(
            PeriodicMessage::from_ctx(&guild_data, ctx, PressReviewMessage).await?,
        ));
    }
    guild_data.save().await?;

    ctx.send(
        CreateReply::new()
            .ephemeral(true)
            .reply(true)
            .content("Ce salon recevra les infos des press reviews"),
    )
    .await?;

    Ok(())
}
