use time::{Month, OffsetDateTime, Weekday};

use std::{
    fs::OpenOptions,
    io::{self, Write},
};

pub fn write_to_log(s: &str) -> io::Result<()> {
    let st = format!("{}: {}", OffsetDateTime::now_local().unwrap(), s);
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open("latest.log")?;
    f.write_all(&st.as_bytes())?;
    f.write_all(b"\n")?;
    f.flush()
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        {
            let s = format!($($arg)*);
            #[cfg(not(debug_assertions))]
            {
                // in release mode
                let _ = crate::utils::write_to_log(&s);
            }
            #[cfg(debug_assertions)]
            {
                println!("{}", s);
            }
        }
    }
}

pub fn month_to_short_fr(month: Month) -> String {
    match month {
        Month::January => "Jan",
        Month::February => "Fév",
        Month::March => "Mars",
        Month::April => "Avril",
        Month::May => "Mai",
        Month::June => "Juin",
        Month::July => "Juil",
        Month::August => "Août",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
    .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jour(Weekday);

impl From<Weekday> for Jour {
    fn from(value: Weekday) -> Self {
        Jour(value)
    }
}

impl Jour {
    pub fn inner(&self) -> Weekday {
        self.0
    }
}

impl Ord for Jour {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .number_days_from_monday()
            .cmp(&other.0.number_days_from_monday())
    }
}

impl PartialOrd for Jour {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for Jour {
    fn from(value: &str) -> Self {
        Self(match value {
            "Lu" => Weekday::Monday,
            "Ma" => Weekday::Tuesday,
            "Me" => Weekday::Wednesday,
            "Je" => Weekday::Thursday,
            "Ve" => Weekday::Friday,
            "Sa" => Weekday::Saturday,
            "Di" => Weekday::Sunday,
            _ => panic!(),
        })
    }
}

impl ToString for Jour {
    fn to_string(&self) -> String {
        match self.0 {
            Weekday::Monday => "Lundi",
            Weekday::Tuesday => "Mardi",
            Weekday::Wednesday => "Mercredi",
            Weekday::Thursday => "Jeudi",
            Weekday::Friday => "Vendredi",
            Weekday::Saturday => "Samedi",
            Weekday::Sunday => "Dimanche",
        }
        .to_string()
    }
}
