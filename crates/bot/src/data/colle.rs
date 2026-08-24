use crate::{
    GLOBAL_DATA,
    data::{
        group::{Group, GroupId},
        guild::GuildData,
        prof::{Prof, ProfId},
        resolve::Resolve,
    },
    error::{ColleParsingError, WattouError},
    utils::{Jour, month_to_short_fr},
};
use color_eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Write, str::FromStr, sync::Arc};
use time::{OffsetDateTime, macros::format_description};

/// e.g. : M4 (Maths n°4)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ColleTemplateId(pub char, pub u8);
pub type ColleId = usize;

impl FromStr for ColleTemplateId {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars();
        let mut chars = chars;
        let c = chars.next().ok_or(WattouError::ColleParsingFailed(
            ColleParsingError::IdParsingFailed,
        ))?;
        let n = chars.collect::<String>().parse()?;
        Ok(Self(c, n))
    }
}

impl ColleTemplateId {
    pub fn explicit(&self) -> String {
        let mut s = match &self.0 {
            'M' => "Maths",
            'P' => "Physique",
            'A' => "Anglais",
            _ => panic!(),
        }
        .to_string();
        s.push(' ');
        s += &self.1.to_string();
        s
    }
}

impl ToString for ColleTemplateId {
    fn to_string(&self) -> String {
        self.0.to_string() + &self.1.to_string()
    }
}

/// Room number, e.g. : 207
pub type RoomNumber = String;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Colle {
    template_id: ColleTemplateId,
    group_id: GroupId,
    pub(crate) start: OffsetDateTime,
    pub(crate) end: OffsetDateTime,
}

pub struct ResolvedColle<'g, 's> {
    template: &'g ColleTemplate,
    group: &'g Group,
    colle: &'s Colle,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ColleTemplate {
    pub id: ColleTemplateId,
    pub prof: ProfId,
    pub room: RoomNumber,

    pub day: Jour,
    pub hour_start: u8,
    pub hour_end: u8,
}

impl Colle {
    pub fn horaire(&self) -> String {
        let format = format_description!("[hour]h");
        [self.start, self.end]
            .map(|date| date.format(format).unwrap())
            .join("-")
    }

    // pub fn from_template(template: &ColleTemplate, date: Date, group_id: GroupId) -> Result<Self> {
    //     Ok(Self {
    //         group_id,
    //         start: date.with_hms(template.hour_start, 0, 0)?.assume_utc(),
    //         end: date.with_hms(template.hour_end, 0, 0)?.assume_utc(),
    //         template,
    //     })
    // }
}

impl Resolve for Colle {
    type Id = ColleId;
    type ResolvedSelf<'g, 's> = ResolvedColle<'g, 's>;
    fn resolve<'s, 'g: 's>(&'s self, guild_data: &'g GuildData) -> Option<Self::ResolvedSelf<'g, 's>> {
        Some(Self::ResolvedSelf{
            group: Group::from_id(&self.group_id, guild_data)?,
            template: ColleTemplate::from_id(&self.template_id, guild_data)?,
            colle: self
        })
    }

    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildData) -> Option<&'g Self> {
        guild_data.persistent.colles.get(*id)
    }
}

impl<'g, 's> ResolvedColle<'g, 's> {
    pub fn format<F>(&self, mut f: F, format: ColleStringFormat) -> Result<()>
    where
        F: Write,
    {
        f.write_fmt(format_args!(
            "{}: {} {} {} {} avec {} en {}",
            match format {
                ColleStringFormat::Explicit => self.template.id.explicit(),
                ColleStringFormat::Implicit | ColleStringFormat::ForProf(_) =>
                    self.template.id.to_string(),
            },
            Jour::from(self.colle.start.weekday()).as_str(),
            self.colle.start.day().to_string(),
            month_to_short_fr(self.colle.start.month()),
            self.colle.horaire(),
            match format {
                ColleStringFormat::ForProf(group) => format!("le groupe {} ", group),
                _ => self.template.prof.to_string(),
            },
            &self.template.room,
        ))?;
        Ok(())
    }
}

impl FromStr for ColleTemplate {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        let open_paren = s
            .find("(")
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;

        let room_number = &s[open_paren + 1..s.len() - 1];

        let mut words = s[..open_paren - 1].split(" ");
        let id = ColleTemplateId::from_str(
            words
                .next()
                .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?,
        )?;

        let mut words_vec: Vec<_> = words.collect();

        let horaire = words_vec
            .pop()
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;

        let [hour_start, hour_end]: [u8; 2] = horaire
            .split("-")
            .map(|p| p[..p.len() - 1].parse().ok())
            .collect::<Option<Vec<_>>>()
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?
            .try_into()
            .ok()
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;

        let jour_str = words_vec
            .pop()
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;
        let day = Jour::from_str(jour_str)?;

        let prof_str = words_vec.join(" ");
        let arc_str = Arc::from(prof_str);

        let prof = if let Some(arc) = GLOBAL_DATA.lock().unwrap().profs.get(&arc_str) {
            arc.clone()
        } else {
            let arc = Arc::new(Prof::new(arc_str.clone()));
            GLOBAL_DATA
                .lock()
                .unwrap()
                .profs
                .insert(arc_str, arc.clone());
            arc
        };

        Ok(ColleTemplate {
            id,
            prof,
            room: room_number.into(),
            day,
            hour_start,
            hour_end,
        })
    }
}

impl PartialOrd for Colle {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Colle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

pub enum ColleStringFormat {
    ForProf(GroupId),
    Implicit,
    Explicit,
}
