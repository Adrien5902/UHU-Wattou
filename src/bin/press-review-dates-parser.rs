use std::{env::args, fs, path::PathBuf};

use color_eyre::eyre::{Result, eyre};
use time::{Date, Month, OffsetDateTime, SignedDuration};
use wattou_bot::data::guild::GuildDataPersistent;

pub fn parse(s: &str) -> Result<Vec<Date>> {
    let now = OffsetDateTime::now_utc().date();
    let year_start = next_occurence(
        now.saturating_sub(SignedDuration::days(365)),
        Month::August,
        15,
    )?;

    s.split(" ")
        .map(|date_str| {
            let mut iter = date_str.split("-");
            let month_int: u8 = iter
                .next()
                .ok_or_else(|| eyre!("month not found in press review date"))?
                .parse()?;
            let month = month_int.try_into()?;
            let day = iter
                .next()
                .ok_or_else(|| eyre!("day not found in press review date"))?
                .parse()?;

            Ok(next_occurence(year_start, month, day)?)
        })
        .collect::<Result<_>>()
}

fn next_occurence(date: Date, month: Month, day: u8) -> Result<Date> {
    let year = if (date.month(), date.day()) > (month, day) {
        date.year() + 1
    } else {
        date.year()
    };

    Ok(Date::from_calendar_date(year, month, day)?)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut args = args().skip(1);
    let path_str = args.next().expect("Path argument missing");
    let path = PathBuf::from(path_str);
    let s = fs::read_to_string(path.join("press_review_dates.txt"))?;
    let out_path = path.join("persistent_data.ron");
    let press_review_day = args
        .next()
        .ok_or_else(|| eyre!("press review day argument missing"))?
        .parse()?;

    if fs::exists(&out_path)?
        && let Some(mut data) =
            ron::from_str::<GuildDataPersistent>(&fs::read_to_string(&out_path)?).ok()
    {
        data.press_review_dates = parse(s.trim())?
            .into_iter()
            .map(|d| {
                d.saturating_sub(SignedDuration::days(1))
                    .next_occurrence(press_review_day)
            })
            .collect();
        fs::write(out_path, ron::to_string(&data)?)?;
        return Ok(());
    } else {
        Err(eyre!("no current data in path"))
    }
}
