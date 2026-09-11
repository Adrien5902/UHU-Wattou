use crate::{
    data::guild::{GuildData, GuildDataMutable},
    error::WattouError,
};
use color_eyre::eyre::{Report, Result};
use poise::serenity_prelude::GuildId;
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub type Context<'a> = poise::Context<'a, Mutex<GlobalData>, Report>;

#[derive(Default)]
pub struct GlobalData {
    pub guilds: HashMap<GuildId, Arc<GuildData>>,
}

impl GlobalData {
    pub fn get_guild(&mut self, id: GuildId) -> Result<Arc<GuildData>> {
        match self.guilds.entry(id) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let data = GuildData::new(id)?;
                Ok(entry.insert(Arc::new(data)).clone())
            }
        }
    }

    pub async fn guild_mut_from_ctx(
        &mut self,
        ctx: Context<'_>,
    ) -> Result<OwnedMutexGuard<GuildDataMutable>> {
        let guild_id = ctx
            .guild_id()
            .ok_or(WattouError::CommandCanOnlyBeUsedInGuilds)?;
        let guild = self.get_guild(guild_id)?;
        Ok(guild.mutable.clone().lock_owned().await)
    }

    pub fn guild_from_ctx(&mut self, ctx: Context<'_>) -> Result<Arc<GuildData>> {
        let guild_id = ctx
            .guild_id()
            .ok_or(WattouError::CommandCanOnlyBeUsedInGuilds)?;
        self.get_guild(guild_id)
    }
}
