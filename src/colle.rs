use crate::GLOBAL_DATA;
use crate::error::{ColleParsingError, WattouError};
use crate::prof::Prof;
use crate::utils::{Jour, month_to_short_fr};
use color_eyre::{Result, eyre};
use ics::Event;
use ics::properties::{Categories, Description, DtEnd, DtStart, Organizer, Summary};
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::str::FromStr;
use std::sync::Arc;
use time::format_description::well_known::Iso8601;
use time::macros::format_description;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// e.g. : M4 (Maths n°4)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ColleId(pub char, pub u8);

impl FromStr for ColleId {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
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

const ICS_CATEGORY: Lazy<Categories> = Lazy::new(|| Categories::new("Colles"));

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Colle {
    pub id: ColleId,
    pub prof: Arc<Prof>,
    pub room: RoomNumber,

    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl ToString for Colle {
    fn to_string(&self) -> String {
        self.format(ColleStringFormat::Explicit)
    }
}

impl Colle {
    pub fn horaire(&self) -> String {
        let format = format_description!("[hour]h");
        [self.start, self.end]
            .map(|date| date.format(format).unwrap())
            .join("-")
    }

    pub fn format(&self, format: ColleStringFormat) -> String {
        format!(
            "{}: {} {} {} {} avec {} en {}",
            match format {
                ColleStringFormat::Explicit => self.id.explicit(),
                ColleStringFormat::Implicit => self.id.to_string(),
            },
            Jour::from(self.start.weekday()).to_string(),
            self.start.day(),
            month_to_short_fr(self.start.month()),
            self.horaire(),
            self.prof.to_string(),
            self.room
        )
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

        let prof = if let Some(arc) = GLOBAL_DATA
            .lock()
            .unwrap()
            .profs
            .iter()
            .find(|p| p.name() == prof_str)
        {
            arc.clone()
        } else {
            let arc = Arc::new(Prof::new(prof_str));
            GLOBAL_DATA.lock().unwrap().profs.push(arc.clone());
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

    pub fn to_ics_event(&self) -> Result<Event> {
        let [start, end]: [String; 2] = [self.start, self.end]
            .iter()
            .map(|date| {
                Ok(date
                    .format(&Iso8601::DATE_TIME)?
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>())
            })
            .collect::<Result<Vec<String>, eyre::Report>>()?
            .try_into()
            .unwrap();

        let mut event = Event::new(Uuid::new_v4().to_string(), start.clone());

        event.push(Organizer::new(self.prof.to_string()));
        event.push(DtStart::new(start));
        event.push(DtEnd::new(end));
        event.push(ICS_CATEGORY.clone());
        event.push(Summary::new(format!(
            "Colle {} avec {}",
            &self.id.explicit(),
            &self.prof.to_string()
        )));
        event.push(Description::new(format!(
            "Colle {} avec {} en salle {} de {}",
            &self.id.explicit(),
            &self.prof.to_string(),
            &self.room,
            self.horaire()
        )));

        Ok(event)
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
    Implicit,
    Explicit,
}
