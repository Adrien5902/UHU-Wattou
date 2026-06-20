use crate::{
    GLOBAL_DATA,
    data::group::GroupId,
    data::prof::Prof,
    error::{ColleParsingError, WattouError},
    utils::{Jour, month_to_short_fr},
};
use color_eyre::{Result, eyre};
use std::{cmp::Ordering, fmt::Write, str::FromStr, sync::Arc};
use time::{Date, OffsetDateTime, macros::format_description};

/// e.g. : M4 (Maths n°4)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ColleId(pub char, pub u8);

impl FromStr for ColleId {
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

impl ColleId {
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

impl ToString for ColleId {
    fn to_string(&self) -> String {
        self.0.to_string() + &self.1.to_string()
    }
}

/// Room number, e.g. : 207
pub type RoomNumber = String;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Colle {
    pub template: Arc<ColleTemplate>,
    pub group_id: GroupId,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ColleTemplate {
    pub id: ColleId,
    pub prof: Arc<Prof>,
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
            Jour::from(self.start.weekday()).as_str(),
            self.start.day().to_string(),
            month_to_short_fr(self.start.month()),
            self.horaire(),
            match format {
                ColleStringFormat::ForProf(group) => format!("le groupe {} ", group),
                _ => self.template.prof.to_string(),
            },
            &self.template.room,
        ))?;
        Ok(())
    }

    pub fn from_template(
        template: Arc<ColleTemplate>,
        date: Date,
        group_id: GroupId,
    ) -> Result<Self> {
        Ok(Self {
            group_id,
            start: date.with_hms(template.hour_start, 0, 0)?.assume_utc(),
            end: date.with_hms(template.hour_end, 0, 0)?.assume_utc(),
            template,
        })
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
        let id = ColleId::from_str(
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
