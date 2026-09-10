use crate::{
    data::{
        group::{Group, GroupId},
        guild::GuildDataPersistent,
        prof::{Prof, ProfId},
        resolve::Resolve,
    },
    error::{ColleParsingError, WattouError},
    utils::{Jour, month_to_short_fr},
};
use color_eyre::{
    Result,
    eyre::{self, eyre},
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt::{Display, Write},
    str::FromStr,
};
use time::{Date, OffsetDateTime, macros::format_description};

/// e.g. : M4 (Maths n°4)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

impl Display for ColleTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0)?;
        f.write_str(&self.1.to_string())
    }
}

impl Serialize for ColleTemplateId {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ColleTemplateId {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ColleTemplateId::from_str(&s)
            .map_err(|_| serde::de::Error::custom("failed to deser ColleTemplateId"))
    }
}

/// Room number, e.g. : 207
pub type RoomNumber = String;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Colle {
    pub template_id: ColleTemplateId,
    pub group_id: GroupId,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedColle<'g, 's> {
    pub template: &'g ColleTemplate,
    pub group: &'g Group,
    pub colle: &'s Colle,
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

pub struct ResolvedColleTemplate<'g, 's> {
    pub template: &'s ColleTemplate,
    pub prof: &'g Prof,
}

impl Resolve for ColleTemplate {
    type Id = ColleTemplateId;
    type ResolvedSelf<'g, 's>
        = ResolvedColleTemplate<'g, 's>
    where
        'g: 's,
        Self: 's;
    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self> {
        guild_data.colle_templates.get(id)
    }

    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g GuildDataPersistent,
    ) -> Result<Self::ResolvedSelf<'g, 's>> {
        Ok(ResolvedColleTemplate {
            template: self,
            prof: Prof::from_id(&self.prof, guild_data)
                .ok_or_else(|| eyre!("can't resolve prof {} for colle template", self.prof))?,
        })
    }
}

impl Colle {
    pub fn horaire(&self) -> String {
        let format = format_description!("[hour]h");
        [self.start, self.end]
            .map(|date| date.format(format).unwrap())
            .join("-")
    }

    pub fn from_template(template: &ColleTemplate, date: Date, group_id: GroupId) -> Result<Self> {
        Ok(Self {
            group_id,
            start: date.with_hms(template.hour_start, 0, 0)?.assume_utc(),
            end: date.with_hms(template.hour_end, 0, 0)?.assume_utc(),
            template_id: template.id,
        })
    }
}

impl Resolve for Colle {
    type Id = ColleId;
    type ResolvedSelf<'g: 's, 's> = ResolvedColle<'g, 's>;
    fn resolve<'s, 'g: 's>(
        &'s self,
        guild_data: &'g GuildDataPersistent,
    ) -> Result<Self::ResolvedSelf<'g, 's>> {
        Ok(Self::ResolvedSelf {
            group: Group::from_id(&self.group_id, guild_data)
                .ok_or_else(|| eyre!("can't resolve group {} for colle", self.group_id))?,
            template: ColleTemplate::from_id(&self.template_id, guild_data)
                .ok_or_else(|| eyre!("can't resolve template for colle"))?,
            colle: self,
        })
    }

    fn from_id<'g>(id: &Self::Id, guild_data: &'g GuildDataPersistent) -> Option<&'g Self> {
        guild_data.colles.get(*id)
    }
}

impl<'g, 's> ResolvedColle<'g, 's> {
    pub fn format<F>(
        &self,
        mut f: F,
        format: ColleStringFormat,
        guild_data: &GuildDataPersistent,
    ) -> Result<()>
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
            self.colle.start.day(),
            month_to_short_fr(self.colle.start.month()),
            self.colle.horaire(),
            match format {
                ColleStringFormat::ForProf(group) => Cow::Owned(format!("le groupe {} ", group)),
                _ => Cow::Borrowed(
                    Prof::from_id(&self.template.prof, guild_data)
                        .ok_or_else(|| eyre!("error colle :("))?
                        .name()
                ),
            },
            self.template.room,
        ))?;
        Ok(())
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
