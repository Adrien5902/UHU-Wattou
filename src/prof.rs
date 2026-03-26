use std::sync::Arc;

use time::OffsetDateTime;

use crate::{colle::Colle, group::GroupId, guild_data::GuildData};

#[derive(Debug, Clone)]
pub struct Prof {
    name: Arc<str>,
}

impl PartialEq for Prof {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Prof {}

impl ToString for Prof {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl Prof {
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn new(name: Arc<str>) -> Self {
        Self { name }
    }

    pub fn get_next_colles_in_guild(
        &self,
        guild_data: Arc<GuildData>,
        limit: usize,
    ) -> Vec<(GroupId, Colle)> {
        let now = OffsetDateTime::now_utc();
        let mut colles = guild_data
            .groups
            .iter()
            .flat_map(|groupe| {
                groupe.colles.iter().filter_map(|colle| {
                    (*self == *colle.prof && colle.end > now).then_some((groupe.id, colle.clone()))
                })
            })
            .collect::<Vec<_>>();
        colles.sort_by(|(_, a), (_, b)| a.cmp(b));

        colles[..limit].to_vec()
    }
}
