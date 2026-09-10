use color_eyre::eyre::Result;

use crate::data::guild::{GuildDataPersistent};

pub trait Resolve {
    type ResolvedSelf<'g, 's>
    where
        'g: 's,
        Self: 's;

    type Id;
    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g GuildDataPersistent,
    ) -> Result<Self::ResolvedSelf<'g, 's>>;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self>;
}
