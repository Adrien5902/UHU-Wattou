use std::{
    collections::{self, HashMap},
    fmt::Debug,
    time::Duration,
    u8,
};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, GetMessages, GuildId, Http, Mention, PrivateChannel, UserId};
use time::OffsetDateTime;

use crate::{
    colle::Colle,
    debug,
    group::GroupId,
    guild_data::{GuildData, SavedData},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Subscribers {
    map: HashMap<UserId, SubscriberData>,
}

impl SavedData for Subscribers {
    const FILE_NAME: &'static str = "subscribers.json";
    fn ser(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn de(value: &str) -> color_eyre::Result<Self> {
        Ok(serde_json::from_str(value)?)
    }
}

impl Subscribers {
    pub fn get(&self, user_id: &UserId) -> Option<&SubscriberData> {
        self.map.get(user_id)
    }

    pub fn remove(
        &mut self,
        guild_id: GuildId,
        user_id: &UserId,
    ) -> Result<Option<SubscriberData>> {
        let data = self.map.remove(user_id);
        self.save(guild_id)?;
        Ok(data)
    }

    pub fn set(&mut self, guild_id: GuildId, user_id: UserId, data: SubscriberData) -> Result<()> {
        self.map.insert(user_id, data);
        self.save(guild_id)?;
        Ok(())
    }

    pub fn iter<'a>(&'a self) -> collections::hash_map::Iter<'a, UserId, SubscriberData> {
        self.map.iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SubscriberData {
    pub group_id: GroupId,
}

impl SubscriberData {
    pub fn new_default(group_id: GroupId) -> Self {
        Self { group_id }
    }
}

pub trait SubscribePlan: Debug {
    type Predicate: ToString;

    fn get_predicate(&self, guild_data: &GuildData) -> Option<Self::Predicate>;
    fn create_message(&self, user_id: UserId, predicate: &Self::Predicate) -> Result<String>;
    fn should_make_message(&self, predicate: &Self::Predicate) -> bool;

    async fn check_already_sent(
        channel: &PrivateChannel,
        http: &Http,
        content: &str,
    ) -> Result<bool> {
        let last_messages = channel
            .messages(http, GetMessages::default().limit(7))
            .await?;

        Ok(last_messages
            .iter()
            .find(|message| message.content == content)
            .is_some())
    }

    async fn try_send(&self, user_id: UserId, http: &Http, guild_data: &GuildData) -> Result<()> {
        if let Some(predicate) = self.get_predicate(guild_data) {
            if self.should_make_message(&predicate) {
                let user = http.get_user(user_id).await?;
                let channel = user.create_dm_channel(http).await?;
                let content = self.create_message(user_id, &predicate)?;

                if !Self::check_already_sent(&channel, http, &content).await? {
                    channel
                        .send_message(http, CreateMessage::new().content(content))
                        .await?;
                    debug!(
                        "sent subscriber message for {} with {}",
                        user_id,
                        predicate.to_string()
                    )
                } else {
                    debug!(
                        "already sent subscriber message for {} with {} skipped sending",
                        user_id,
                        predicate.to_string()
                    )
                }
            }
        }

        Ok(())
    }
}

impl SubscriberData {
    const MIN_HOUR_DIFF: u64 = 30;
}

impl SubscribePlan for SubscriberData {
    type Predicate = Colle;

    fn get_predicate(&self, guild_data: &GuildData) -> Option<Self::Predicate> {
        guild_data
            .get_group(self.group_id)
            .ok()?
            .get_next_colles(4)
            .into_iter()
            .find(|colle| colle.id.0 == 'A')
            .cloned()
    }

    fn should_make_message(&self, colle: &Self::Predicate) -> bool {
        return colle.start - OffsetDateTime::now_local().unwrap()
            < Duration::from_secs(60 * 60 * Self::MIN_HOUR_DIFF);
    }

    fn create_message(&self, user_id: UserId, predicate: &Self::Predicate) -> Result<String> {
        Ok(format!(
            "{}, n'oublie pas ton carnet de colle pour ta colle {}",
            Mention::from(user_id),
            predicate.format(crate::colle::ColleStringFormat::Explicit)
        ))
    }
}
