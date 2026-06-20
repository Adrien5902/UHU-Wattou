use crate::{
    data::colle::{Colle, ColleStringFormat},
    data::group::GroupId,
    data::guild::{GuildData, SavedData},
    debug,
};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, GetMessages, GuildId, Http, Mention, PrivateChannel, UserId};
use std::{
    collections::{self, HashMap},
    fmt::{Debug, Write},
    time::Duration,
    u8,
};
use time::OffsetDateTime;

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
    type PredicateData: Debug;

    fn get_predicate<'a>(&self, guild_data: &'a GuildData) -> Option<&'a Self::PredicateData>;
    fn create_message(&self, user_id: UserId, predicate: &Self::PredicateData) -> Result<String>;
    fn should_make_message(&self, predicate: &Self::PredicateData) -> bool;

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

    async fn try_send<'a>(
        &'a self,
        user_id: UserId,
        http: &Http,
        guild_data: &'a GuildData,
    ) -> Result<()> {
        if let Some(predicate_data) = self.get_predicate(guild_data) {
            if self.should_make_message(&predicate_data) {
                let user = http.get_user(user_id).await?;
                let channel = user.create_dm_channel(http).await?;
                let content = self.create_message(user_id, &predicate_data)?;

                if !Self::check_already_sent(&channel, http, &content).await? {
                    channel
                        .send_message(http, CreateMessage::new().content(content))
                        .await?;
                    debug!(
                        "sent subscriber message for {} with {:?}",
                        user_id, predicate_data
                    )
                } else {
                    debug!(
                        "already sent subscriber message for {} with {:?} skipped sending",
                        user_id, predicate_data
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
    type PredicateData = Colle;

    fn get_predicate<'a>(&self, guild_data: &'a GuildData) -> Option<&'a Self::PredicateData> {
        guild_data
            .get_group(self.group_id)
            .ok()?
            .get_next_colles(4)
            .into_iter()
            .find(|colle| colle.template.id.0 == 'A')
    }

    fn should_make_message(&self, colle: &Self::PredicateData) -> bool {
        return colle.start - OffsetDateTime::now_local().unwrap()
            < Duration::from_secs(60 * 60 * Self::MIN_HOUR_DIFF);
    }

    fn create_message(&self, user_id: UserId, predicate: &Self::PredicateData) -> Result<String> {
        let mut content = String::new();
        content.write_fmt(format_args!(
            "{}, n'oublie pas ton carnet de colle pour ta colle",
            Mention::from(user_id),
        ))?;
        predicate.format(&mut content, ColleStringFormat::Explicit)?;
        Ok(content)
    }
}
