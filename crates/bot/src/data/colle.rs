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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
pub type ColleData = (ColleId, (u8, u8), Jour, RoomNumber, Arc<Prof>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Colle {
    pub id: ColleId,
    pub prof: Arc<Prof>,
    pub room: RoomNumber,

    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
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
                ColleStringFormat::Explicit => self.id.explicit(),
                ColleStringFormat::Implicit | ColleStringFormat::ForProf(_) => self.id.to_string(),
            },
            Jour::from(self.start.weekday()).to_string(),
            self.start.day().to_string(),
            month_to_short_fr(self.start.month()),
            self.horaire(),
            match format {
                ColleStringFormat::ForProf(group) => format!("le groupe {} ", group),
                _ => self.prof.to_string(),
            },
            &self.room,
        ))?;
        Ok(())
    }

    pub fn parse_string(s: impl Into<String>) -> Result<ColleData> {
        let mut string = s.into();
        let open_paren = string
            .find("(")
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;
        let room_number = &string[open_paren + 1..string.len() - 1].to_string();
        string.replace_range(open_paren - 1..string.len(), "");
        let mut words = string.split(" ");
        let id = ColleId::from_str(
            words
                .next()
                .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?,
        )?;

        let mut words_vec: Vec<_> = words.collect();

        let horaire = words_vec
            .pop()
            .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;

        let [start, end]: [u8; 2] = horaire
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
        let jour = Jour::from(jour_str);

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

        Ok((id, (start, end), jour, room_number.clone(), prof))
    }

    pub fn from_data_and_date(date: Date, data: ColleData) -> Result<Self> {
        let (id, (start, end), _, room, prof) = data;

        Ok(Self {
            id,
            room,
            start: date.with_hms(start, 0, 0)?.assume_utc(),
            end: date.with_hms(end, 0, 0)?.assume_utc(),
            prof,
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
