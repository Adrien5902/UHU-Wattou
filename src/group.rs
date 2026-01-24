use crate::colle::Colle;
use color_eyre::Result;
use ics::ICalendar;
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
            .collect::<Vec<_>>()[..limit]
            .into()
    }

    pub fn ics_calendar<'a>(&self) -> Result<String> {
        let mut calendar = ICalendar::new(
            "2.0",
            format!("-//Wattou//Calendrier de colle groupe {}//FR", self.id),
        );

        // create event which contains the information regarding the conference
        for colle in self.colles.iter() {
            calendar.add_event(colle.to_ics_event()?);
        }

        let mut writer = Vec::new();
        calendar.write(&mut writer).unwrap();
        Ok(String::from_utf8(writer).unwrap())
    }
}
