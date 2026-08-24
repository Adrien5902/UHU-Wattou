use crate::data::{
    colle::{Colle, ColleId},
    resolve::Resolve,
};
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use time::OffsetDateTime;

pub type GroupId = usize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Group {
    guild_id: GuildId,
    id: GroupId,
    colles: Box<[ColleId]>,
}

pub struct ResolvedGroup<'g, 's> {
    group: &'s Group,
    colles: Box<[&'g Colle]>,
}

impl Resolve for Group {
    type Id = GroupId;
    type ResolvedSelf<'g, 's> = ResolvedGroup<'g, 's>;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g super::guild::GuildData) -> Option<&'g Self> {
        guild_data.persistent.groups.get(*id)
    }

    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g super::guild::GuildData,
    ) -> Option<Self::ResolvedSelf<'g, 's>> {
        Some(Self::ResolvedSelf {
            group: self,
            colles: self
                .colles
                .iter()
                .map(|id| Colle::from_id(id, guild_data))
                .collect::<Option<_>>()?,
        })
    }
}

impl<'g, 's> ResolvedGroup<'g, 's> {
    pub fn get_next_colles(&self, limit: usize) -> Vec<&Colle> {
        let now = OffsetDateTime::now_utc();
        self.colles
            .iter()
            .filter(|colle| colle.end > now)
            .take(limit)
            .map(|c| *c)
            .collect()
    }
}
