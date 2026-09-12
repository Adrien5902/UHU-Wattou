use std::fmt::Display;

use crate::data::{group::GroupId, guild::GuildDataPersistent, resolve::Resolve};
use serde::{Deserialize, Serialize};

pub type StudentId = usize;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Student {
    last_name: String,
    first_name: String,
    group_id: GroupId,
}

impl Display for Student {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.first_name, self.last_name))
    }
}

impl Student {
    pub fn new(first_name: String, last_name: String, group_id: GroupId) -> Self {
        Self {
            first_name,
            last_name,
            group_id,
        }
    }

    pub fn group_id(&self) -> GroupId {
        self.group_id
    }
}

impl Resolve for Student {
    type Id = StudentId;
    type ResolvedSelf<'g, 's>
        = &'s Self
    where
        'g: 's,
        Self: 's;

    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self> {
        guild_data.students.get(*id)
    }

    fn resolve<'s, 'g: 's>(
        &'s self,
        _guild_data: &'g GuildDataPersistent,
    ) -> color_eyre::eyre::Result<Self::ResolvedSelf<'g, 's>> {
        Ok(self)
    }
}
