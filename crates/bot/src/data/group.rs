use crate::data::colle::Colle;
use serenity::all::GuildId;
use time::OffsetDateTime;

pub type GroupId = usize;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Group {
    pub guild_id: GuildId,
    pub id: GroupId,
    pub colles: Vec<Colle>,
}

impl Group {
    pub fn get_next_colles<'a>(&'a self, limit: usize) -> Vec<&'a Colle> {
        let now = OffsetDateTime::now_utc();
        self.colles
            .iter()
            .filter(|colle| colle.end > now)
            .take(limit)
            .collect()
    }
}
