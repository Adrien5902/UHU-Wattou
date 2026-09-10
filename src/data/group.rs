use crate::data::{
    colle::{Colle, ColleId},
    guild::GuildDataPersistent,
    resolve::Resolve,
};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub type GroupId = usize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub colles: Vec<ColleId>,
}

pub struct ResolvedGroup<'g, 's> {
    pub group: &'s Group,
    pub colles: Box<[&'g Colle]>,
}

impl Resolve for Group {
    type Id = GroupId;
    type ResolvedSelf<'g: 's, 's> = ResolvedGroup<'g, 's>;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self> {
        guild_data.groups.get(*id - 1)
    }

    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g GuildDataPersistent,
    ) -> Result<Self::ResolvedSelf<'g, 's>> {
        Ok(Self::ResolvedSelf {
            group: self,
            colles: self
                .colles
                .iter()
                .map(|id| {
                    Colle::from_id(id, guild_data)
                        .ok_or_else(|| eyre!("cant resolve colle {id} for group {}", self.id))
                })
                .collect::<Result<_>>()?,
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
            .copied()
            .collect()
    }
}
