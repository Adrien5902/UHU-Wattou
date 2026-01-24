use crate::{debug, error::WattouError, guild_data::SavedData};
use color_eyre::Result;
use serenity::all::{ChannelId, EditMessage, Http, Message, MessageId};
use std::borrow::Cow;

pub struct RecurrentMessage<const ID: usize>((MessageId, ChannelId));

impl<const ID: usize> RecurrentMessage<ID> {
    const IDS: [&'static str; 2] = ["message_semaine_tp", "message_colles"];

    pub fn inner(&self) -> (MessageId, ChannelId) {
        self.0
    }

    pub async fn edit(&self, http: &Http, content: impl Into<String>) -> Result<()> {
        let (message_id, channel_id) = self.inner();
        let mut message = http.get_message(channel_id, message_id).await?;

        message
            .edit(http, EditMessage::new().content(content.into()))
            .await?;

        debug!(
            "edited message {} in channel {} successfully",
            message.id, message.channel_id
        );

        Ok(())
    }
}

impl<const ID: usize> SavedData for RecurrentMessage<ID> {
    const FILE_NAME: &'static str = Self::IDS[ID];

    fn de(s: &str) -> Result<Self> {
        let mut lines = s.lines();

        let (message_id, channel_id) =
            (|| Some((lines.next()?.parse().ok()?, lines.next()?.parse().ok()?)))()
                .ok_or(WattouError::MessageParsingFailed)?;

        Ok(Self((message_id, channel_id)))
    }

    fn ser(&self) -> String {
        [self.0.0.to_string(), self.0.1.to_string()].join("\n")
    }
}

impl<const ID: usize> From<&Cow<'_, Message>> for RecurrentMessage<ID> {
    fn from(value: &Cow<'_, Message>) -> Self {
        Self((value.id, value.channel_id))
    }
}

pub type SemaineTPMessage = RecurrentMessage<0>;
pub type ToutesLesCollesMessage = RecurrentMessage<1>;
