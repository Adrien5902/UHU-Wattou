use color_eyre::eyre::Result;
use regex::regex;
use reqwest;
use std::{collections::HashMap, env::args, fs, path::PathBuf};
use time::{Date, SignedDuration, macros::format_description};
use wattou_bot::{
    data::{
        colle::{Colle, ColleTemplate, ColleTemplateId},
        group::Group,
        guild::GuildDataPersistent,
        prof::Prof,
    },
    utils::Jour,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut args = args();
    args.next();
    let path_str = args.next().expect("Path argument missing");
    let path = PathBuf::from(path_str);
    fs::create_dir_all(&path)?;
    let out_path = path.join("persistent_data.ron");

    let mut groups = Vec::new();
    let mut profs: Vec<Prof> = Vec::new();
    let mut colles = Vec::new();
    let mut colle_templates: HashMap<ColleTemplateId, ColleTemplate> = HashMap::new();

    for group_id in 1..=16 {
        let url =
            format!("https://www.normalesup.org/~heriveau/MP2526/0Groupes/groupe_{group_id}.html");
        let content = reqwest::get(url).await?.text().await?;
        for split in content.split("class=\"week-block\"").skip(1) {
            let inner = split.split("class=\"dates\"").skip(1).next().expect("oups");
            let mut iter = inner.split("<td>");

            let week_info_str = iter.next().expect("semaine introuvable");
            let week_first_day_str = regex!(r"du (\d+\/\d+\/\d+)")
                .captures(week_info_str)
                .unwrap()
                .iter()
                .skip(1)
                .next()
                .unwrap()
                .unwrap()
                .as_str();
            let week_parse_format = format_description!("[day]/[month]/[year]");
            let week_first_day = Date::parse(week_first_day_str, week_parse_format)?;

            while let Some(day_hour_str) = iter.next() {
                let [day_str, hour] = regex!(r"(\w+) (\d+)h")
                    .captures(day_hour_str)
                    .unwrap()
                    .iter()
                    .skip(1)
                    .map(|m| m.unwrap().as_str())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                let day = Jour::from_str_long(day_str)?;
                let hour_start = hour.parse()?;
                let hour_end = hour_start + 1;
                let matiere = iter.next().unwrap().split("</td>").next().unwrap();
                let prof_name = iter.next().unwrap().split("</td>").next().unwrap();
                let room = iter
                    .next()
                    .unwrap()
                    .split("</td>")
                    .next()
                    .unwrap()
                    .to_owned();
                let prof = profs
                    .iter()
                    .enumerate()
                    .find(|(_, p)| p.name() == prof_name)
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| {
                        profs.push(Prof::from(prof_name));
                        profs.len() - 1
                    });

                let template_id = colle_templates
                    .iter()
                    .find(|(_, t)| {
                        t.prof == prof
                            && t.hour_start == hour_start
                            && t.room == room
                            && t.day == day
                    })
                    .map(|(id, _)| id)
                    .copied()
                    .unwrap_or_else(|| {
                        let id = ColleTemplateId(
                            matiere.chars().next().unwrap(),
                            colle_templates.len() as u8,
                        );
                        let template = ColleTemplate {
                            hour_start,
                            hour_end,
                            day,
                            prof,
                            room,
                        };
                        colle_templates.insert(id, template);
                        id
                    });

                let template = colle_templates.get(&template_id).unwrap();

                let colle = Colle::from_template(
                    template_id,
                    template,
                    week_first_day
                        .saturating_sub(SignedDuration::days(1))
                        .next_occurrence(day.inner()),
                    group_id,
                )?;

                colles.push((colle, group_id));
            }
        }

        groups.push(Group {
            id: group_id,
            colles: Vec::new(),
        });
    }

    colles.sort();

    let data = GuildDataPersistent {
        profs,
        colles: colles
            .into_iter()
            .enumerate()
            .map(|(i, (colle, group_id))| {
                groups[group_id - 1].colles.push(i);
                colle
            })
            .collect(),
        groups,
        ghosts: Vec::new(),
        colle_templates,
    };

    fs::write(out_path, ron::to_string(&data)?)?;

    Ok(())
}
