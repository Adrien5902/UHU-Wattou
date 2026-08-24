use crate::{data::colle::Colle, data::group::GroupId, data::guild::GuildData};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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

impl ToString for Prof {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl Prof {
    pub fn get_next_colles_in_guild(
        &self,
        guild_data: Arc<GuildData>,
        limit: usize,
    ) -> Vec<(GroupId, Colle)> {
        let now = OffsetDateTime::now_utc();
        let mut colles = guild_data
            .persistent
            .groups
            .iter()
            .flat_map(|groupe| {
                groupe.colles.iter().filter_map(|colle| {
                    (*self == *colle.get_template(&guild_data).prof && colle.end > now)
                        .then_some((groupe.id, colle.clone()))
                })
            })
            .collect::<Vec<_>>();
        colles.sort_by(|(_, a), (_, b)| a.cmp(b));

        colles[..limit].to_vec()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type ProfId = usize;
