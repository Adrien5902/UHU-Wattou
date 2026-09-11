use std::fmt::Debug;

use crate::{bot::Context, data::guild::GuildDataPersistent, debug};
use color_eyre::eyre::{Ok, Result, eyre};
use poise::serenity_prelude::{self, ChannelId, EditMessage, Http, Message, MessageId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshableMessage<T: RefreshableMessageKind> {
    kind: T,
    message_id: MessageId,
    channel_id: ChannelId,
}

impl<T: RefreshableMessageKind> RefreshableMessage<T> {
    /// Returns true if message was found, false otherwise
    async fn refresh(&self, http: &Http, guild_data: &GuildDataPersistent) -> Result<bool> {
        let content = self.kind.content(guild_data)?;
        if let Err(e) = self
            .channel_id
            .edit_message(http, self.message_id, EditMessage::new().content(content))
            .await
        {
            if let serenity_prelude::Error::Http(http_error) = &e
                && http_error
                    .status_code()
                    .ok_or(eyre!("no status code for request"))?
                    == 404
            {
                debug!(
                    "message not found {:?} for guild TODO INSERT GUILD_ID, removing...",
                    self.kind
                );
                return Ok(false);
            } else {
                Err(e)?
            }
        }

        println!(
            "refreshed message {:?} for guild TODO INSERT GUILD_ID",
            self.kind
        );
        Ok(true)
    }

    pub async fn from_ctx(
        guild_data: &GuildDataPersistent,
        ctx: Context<'_>,
        kind: T,
    ) -> Result<Self> {
        let content = kind.content(guild_data)?;
        let handle = ctx.say(content).await?;

        let message = handle.message().await?;
        debug!(
            "new refreshable message : {:?} {} {}",
            kind, message.id, message.channel_id,
        );
        Ok(Self::from_message(&message, kind))
    }

    pub fn from_message(message: &Message, kind: T) -> Self {
        Self {
            channel_id: message.channel_id,
            message_id: message.id,
            kind,
        }
    }
}

pub trait RefreshableMessageKind: Debug {
    fn content(&self, guild_data: &GuildDataPersistent) -> Result<String>;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OptionalRefreshableMessage<T: RefreshableMessageKind>(pub Option<RefreshableMessage<T>>);
impl<T: RefreshableMessageKind> OptionalRefreshableMessage<T> {
    pub async fn refresh_if_some(
        &mut self,
        http: &Http,
        guild_data: &GuildDataPersistent,
    ) -> Result<()> {
        if let Some(message) = &self.0 {
            if !message.refresh(http, guild_data).await?{
                self.0 = None
            }
        }
        Ok(())
    }

    pub async fn set_from_ctx(
        &mut self,
        guild_data: &GuildDataPersistent,
        kind: T,
        ctx: Context<'_>,
    ) -> Result<()> {
        if let Some(old_message) = &self.0 {
            old_message
                .channel_id
                .delete_message(ctx.http(), old_message.message_id)
                .await?;
        }

        self.0 = Some(RefreshableMessage::<T>::from_ctx(guild_data, ctx, kind).await?);
        Ok(())
    }
}
