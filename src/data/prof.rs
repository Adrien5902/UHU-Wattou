use crate::data::{guild::GuildDataPersistent, resolve::Resolve};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

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

impl<T> From<T> for Prof where T: Into<Box<str>>{
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
    // pub fn get_next_colles(
    //     &self,
    //     guild_data: Arc<GuildData>,
    //     limit: usize,
    // ) -> Vec<(GroupId, Colle)> {
    //     let now = OffsetDateTime::now_utc();
    //     let mut colles = guild_data
    //         .persistent
    //         .groups
    //         .iter()
    //         .flat_map(|groupe| {
    //             groupe.colles.iter().filter_map(|colle| {
    //                 (*self == *colle.get_template(&guild_data).prof && colle.end > now)
    //                     .then_some((groupe.id, colle.clone()))
    //             })
    //         })
    //         .collect::<Vec<_>>();
    //     colles.sort_by(|(_, a), (_, b)| a.cmp(b));
    //
    //     colles[..limit].to_vec()
    // }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type ProfId = usize;
