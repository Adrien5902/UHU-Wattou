use std::fmt::Display;

use crate::data::{colle::ResolvedColle, guild::GuildDataPersistent, resolve::Resolve};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prof {
    name: Box<str>,
}

impl PartialEq for Prof {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Prof {}
impl Display for Prof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl<T> From<T> for Prof
where
    T: Into<Box<str>>,
{
    fn from(value: T) -> Self {
        Prof { name: value.into() }
    }
}

impl Resolve for Prof {
    type ResolvedSelf<'g, 's>
        = &'s Prof
    where
        'g: 's,
        Self: 's;

    type Id = ProfId;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self> {
        guild_data.profs.get(*id)
    }

    fn resolve<'s, 'g: 's>(
        &'s self,
        _guild_data: &'g GuildDataPersistent,
    ) -> Result<Self::ResolvedSelf<'g, 's>> {
        Ok(self)
    }
}

impl Prof {
    pub fn get_next_colles<'g: 's, 's>(
        id: ProfId,
        guild_data: &'g GuildDataPersistent,
        limit: usize,
    ) -> Box<[ResolvedColle<'g, 's>]> {
        let now = OffsetDateTime::now_utc();
        guild_data
            .colles
            .iter()
            .filter_map(|colle| {
                let resolved = colle.resolve(guild_data).ok()?;
                (id == resolved.template.prof && colle.end > now).then_some(resolved)
            })
            .take(limit)
            .collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type ProfId = usize;
