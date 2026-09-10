use crate::{data::guild::GuildData, error::WattouError};
use color_eyre::eyre::{Report, Result};
use poise::serenity_prelude::GuildId;
use std::collections::{HashMap, hash_map::Entry};
use tokio::sync::Mutex;

pub type Context<'a> = poise::Context<'a, Mutex<GlobalData>, Report>;

#[derive(Default)]
pub struct GlobalData {
    pub guilds: HashMap<GuildId, GuildData>,
}

impl GlobalData {
    pub fn get_guild(&mut self, id: GuildId) -> Result<&GuildData> {
        match self.guilds.entry(id) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let data = GuildData::new(id)?;
                Ok(entry.insert(data))
            }
        }
    }

    pub fn guild_mut_from_ctx(&mut self, ctx: Context<'_>) -> Result<&mut GuildData> {
        let guild_id = ctx
            .guild_id()
            .ok_or(WattouError::CommandCanOnlyBeUsedInGuilds)?;
        // This is to make sure guild is initialized
        self.get_guild(guild_id)?;
        Ok(self.guilds.get_mut(&guild_id).unwrap())
    }

    pub fn guild_from_ctx(&mut self, ctx: Context<'_>) -> Result<&GuildData> {
        let guild_id = ctx
            .guild_id()
            .ok_or(WattouError::CommandCanOnlyBeUsedInGuilds)?;
        Ok(self.get_guild(guild_id)?)
    }
}
