use crate::data::guild::GuildData;

pub trait Resolve {
    type ResolvedSelf<'g, 's>;
    type Id;
    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g GuildData,
    ) -> Option<Self::ResolvedSelf<'g, 's>>;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildData) -> Option<&'g Self>;
}
