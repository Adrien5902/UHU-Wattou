use std::{collections::HashMap, env::args, fs, path::PathBuf, str::FromStr};

use color_eyre::eyre::Result;
use time::{Date, Duration, macros::format_description};
use wattou_bot::{
    data::{
        colle::{Colle, ColleTemplate, ColleTemplateId},
        group::Group,
        guild::GuildDataPersistent,
        prof::Prof,
    },
    error::{ColleParsingError, WattouError},
    utils::Jour,
};

pub struct WattiezDataDir {
    path: PathBuf,
}

impl WattiezDataDir {
    pub const GLOBAL_DATA_FOLDER_NAME: &'static str = "data";
    pub const FILE_NAME_COLLE_TEMPLATES_LIST: &'static str = "templates";
    pub const FILE_NAME_WEEKS_INFO: &'static str = "weeks";
    pub const FILE_NAME_COLLOSCOPE: &'static str = "colloscope";

    pub fn parse_colle_templates(
        &self,
    ) -> Result<(Vec<Prof>, HashMap<ColleTemplateId, ColleTemplate>)> {
        let mut profs: Vec<Prof> = Vec::new();
        let mut templates = HashMap::new();
        for line in
            fs::read_to_string(self.path.join(Self::FILE_NAME_COLLE_TEMPLATES_LIST))?.lines()
        {
            let open_paren = line
                .find("(")
                .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?;

            let room_number = &line[open_paren + 1..line.len() - 1];

            let mut words = line[..open_paren - 1].split(" ");
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

            let prof_id = profs
                .iter()
                .enumerate()
                .find(|(_, p)| p.name() == prof_str)
                .map(|(i, _)| i)
                .unwrap_or_else(|| {
                    profs.push(Prof::from(prof_str));
                    profs.len() - 1
                });

            templates.insert(
                id,
                ColleTemplate {
                    id,
                    prof: prof_id,
                    room: room_number.to_owned(),
                    day,
                    hour_start,
                    hour_end,
                },
            );
        }

        Ok((profs, templates))
    }

    pub fn read_weeks_data(&self) -> Result<Vec<Date>> {
        let format = format_description!("[day padding:none]-[month padding:none]-[year]");
        let res = fs::read_to_string(self.path.join(Self::FILE_NAME_WEEKS_INFO))?
            .lines()
            .map(|line| Date::parse(line.split(" ").last().unwrap(), &format))
            .collect::<Result<Vec<Date>, _>>()?;
        Ok(res)
    }

    pub fn parse_colloscope(&self) -> Result<GuildDataPersistent> {
        let mut groups = Vec::new();
        let (profs, colle_templates) = self.parse_colle_templates()?;
        let weeks_dates = self.read_weeks_data()?;
        let colloscope = fs::read_to_string(self.path.join(Self::FILE_NAME_COLLOSCOPE))?;
        let mut colles = Vec::new();
        let mut lines = colloscope.lines();

        let first_line = lines.next().unwrap();
        let week_numbers: Vec<Vec<usize>> = first_line
            .split(" ")
            .map(|weeks_tuple| weeks_tuple.split("-").map(|n| n.parse().unwrap()).collect())
            .collect();

        for (i, line) in lines.enumerate() {
            let group_id = i + 1;
            let week_templates: Vec<Vec<ColleTemplate>> = line
                .split(" ")
                .map(|s| {
                    s.split("+")
                        .map(|colle_id| {
                            Ok(colle_templates
                                .get(&ColleTemplateId::from_str(colle_id)?)
                                .ok_or(WattouError::ColleParsingFailed(ColleParsingError::Unknown))?
                                .clone())
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<Vec<_>>>>()?;

            for (j, weeks) in week_numbers.iter().enumerate() {
                let templates_for_this_week = &week_templates[j];

                for week_number in weeks {
                    for template in templates_for_this_week {
                        let date = Self::get_date(&weeks_dates, *week_number, template.day);
                        let colle = Colle::from_template(template, date, group_id)?;

                        colles.push(colle);
                    }
                }
            }

            groups.push(Group {
                id: group_id,
                colles: Vec::new(),
            });
        }

        // Colles needs to be sorted
        colles.sort();
        for (i, colle) in colles.iter().enumerate() {
            groups[colle.group_id - 1].colles.push(i);
        }

        Ok(GuildDataPersistent {
            profs,
            colle_templates,
            ghosts: Vec::new(),
            groups,
            colles,
        })
    }

    pub fn get_date(weeks: &[Date], week: usize, day: Jour) -> Date {
        weeks[week - 1]
            .saturating_sub(Duration::days(7))
            .next_occurrence(day.inner())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = args();
    args.next();
    let path_str = args.next().expect("Path argument missing");
    let path = PathBuf::from(path_str);
    let out_path = path.join("out.ron");
    let dir = WattiezDataDir { path };

    let data = dir.parse_colloscope()?;
    fs::write(out_path, ron::to_string(&data)?)?;

    Ok(())
}
