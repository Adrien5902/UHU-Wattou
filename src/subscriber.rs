use std::{collections::HashMap, u8};

use anyhow::Result;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};

use crate::{
    group::GroupId,
    guild_data::{SavedData, SavedDataWithDefault},
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
    fn de(value: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(value)?)
    }
}

impl SavedDataWithDefault for Subscribers {}

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
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SubscriberData {
    pub group_id: GroupId,
    pub plan: SubscribePlan,
}

impl SubscriberData {
    pub fn new_default(group_id: GroupId) -> Self {
        Self {
            group_id,
            plan: SubscribePlan::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SubscribePlan(u8);
bitflags! {
    impl SubscribePlan: u8 {
        const LivretColleAnglais = 0b00000001;
        const All = u8::MAX;
    }
}

impl Default for SubscribePlan {
    fn default() -> Self {
        SubscribePlan::All
    }
}
