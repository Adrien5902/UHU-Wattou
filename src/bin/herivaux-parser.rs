use color_eyre::eyre::Result;
use regex::regex;
use std::{collections::HashMap, env::args, fs, path::PathBuf};
use time::{Date, SignedDuration, macros::format_description};
use wattou_bot::{
    data::{
        colle::{Colle, ColleTemplate, ColleTemplateId},
        group::Group,
        guild::GuildDataPersistent,
        prof::Prof,
        student::Student,
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
    let mut students = Vec::new();

    for group_id in 1..=16 {
        let url =
            format!("https://www.normalesup.org/~heriveau/MP2526/0Groupes/groupe_{group_id}.html");
        let content = reqwest::get(url).await?.text().await?;
        let mut content_iter = content.split("class=\"week-block\"");

        let header_str = content_iter.next().unwrap();
        let inner_members_str = header_str
            .split("<span class=\"label\">Membres :</span> ")
            .skip(1)
            .next()
            .unwrap()
            .split("</div>")
            .next()
            .unwrap();

        for member_str in inner_members_str.split(" — ") {
            let mut memeber_str_split = member_str.split(" ");
            let first_name = memeber_str_split.next().unwrap();
            let last_name = memeber_str_split.collect();
            students.push(Student::new(first_name.to_owned(), last_name, group_id));
        }

        for split in content_iter {
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
            students: Vec::new(),
            colles: Vec::new(),
        });
    }

    colles.sort();
    students.sort();

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
        students: students
            .into_iter()
            .enumerate()
            .map(|(i, student)| {
                groups[student.group_id() - 1].students.push(i);
                student
            })
            .collect(),
        groups,
        ghosts: Vec::new(),
        colle_templates,
    };

    fs::write(out_path, ron::to_string(&data)?)?;

    Ok(())
}
