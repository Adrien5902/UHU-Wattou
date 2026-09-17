use crate::{bot::Context, data::guild::{GuildData, GuildDataPersistent}, debug};
use color_eyre::eyre::Result;
use poise::serenity_prelude::{ChannelId, CreateMessage, GetMessages, Http};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct PeriodicMessage<K: PeriodicMessageKind> {
    kind: K,
    defined_channel: ChannelId,
}

pub trait PeriodicMessageKind: Debug {
    async fn content(&self, guild_data: &GuildDataPersistent) -> Result<Option<String>>;
}

impl<K: PeriodicMessageKind> PeriodicMessage<K> {
    pub async fn update(&mut self, http: &Http, guild_data: &GuildData) -> Result<()> {
        let Some(content) = self.kind.content(&guild_data.persistent).await? else {
            return Ok(());
        };

        let last_message = self
            .defined_channel
            .messages(http, GetMessages::new().limit(1))
            .await?
            .into_iter()
            .next();

        if let Some(old_message) = last_message
            && old_message.content == content
        {
            // No need to update
            return Ok(());
        }

        self.defined_channel
            .send_message(http, CreateMessage::new().content(content))
            .await?;

        Ok(())
    }

    pub async fn from_ctx(guild_data: &GuildData, ctx: Context<'_>, kind: K) -> Result<Self> {
        let defined_channel = ctx.channel_id();
        debug!(
            "set channel {} to receive updates for {:?}",
            defined_channel, kind
        );

        let mut message = PeriodicMessage {
            kind,
            defined_channel,
        };
        message.update(ctx.http(), guild_data).await?;

        Ok(message)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OptionalPeriodicMessage<K: PeriodicMessageKind>(pub Option<PeriodicMessage<K>>);
impl<K: PeriodicMessageKind> OptionalPeriodicMessage<K> {
    #[inline]
    pub async fn update_if_some(&mut self, http: &Http, guild_data: &GuildData) -> Result<()> {
        if let Some(inner) = &mut self.0 {
            return inner.update(http, guild_data).await;
        }
        Ok(())
    }
}
