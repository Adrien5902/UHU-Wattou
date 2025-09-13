#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use poise::CreateReply;
use serenity::{
    all::{ChannelId, EditMessage, MessageId, Ready},
    async_trait,
    prelude::*,
};
use std::{
    cell::LazyCell,
    fs::{self, OpenOptions},
    io::{self, Write},
};
use time::{Date, Duration, Month, OffsetDateTime, Weekday, macros::format_description};

use dotenv::dotenv;
use std::env;

pub fn write_to_log(s: &str) -> io::Result<()> {
    let st = format!(
        "{}: {}",
        OffsetDateTime::now_local().unwrap().to_string(),
        s
    );
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open("latest.log")?;
    f.write_all(&st.as_bytes())?;
    f.write_all(b"\n")?;
    f.flush()
}

macro_rules! debug {
    ($($arg:tt)*) => {
        {
            let s = format!($($arg)*);
            #[cfg(not(debug_assertions))]
            {
                // in release mode
                let _ = write_to_log(&s);
            }
            #[cfg(debug_assertions)]
            {
                println!("{}", s);
            }
        }
    }
}

const DATA: LazyCell<Data> = LazyCell::new(|| {
    let (groups, profs) = (WeekGroups::parse(), ProfData::parse());
    Data {
        week_groups: groups,
        profs,
        weeks: Data::get_weeks(),
    }
});

struct Data {
    profs: ProfData,
    week_groups: Vec<WeekGroups>,
    weeks: Vec<Date>,
}

impl Data {
    fn get_toutes_les_colles_id() -> Option<(u64, u64)> {
        let Some(file) = fs::read_to_string("message.txt").ok() else {
            return None;
        };

        let mut lines = file.lines();

        Some((lines.next()?.parse().ok()?, lines.next()?.parse().ok()?))
    }

    fn get_data_for_group(&self, group: usize) -> Vec<(Vec<usize>, (Prof, Prof))> {
        self.week_groups
            .iter()
            .map(|week_groups| {
                (
                    week_groups.weeks.clone(),
                    self.profs
                        .from_ids(week_groups.groups[(group - 1) as usize].clone()),
                )
            })
            .collect()
    }

    fn get_weeks() -> Vec<Date> {
        let format = format_description!("[day padding:none]-[month padding:none]-[year]");
        fs::read_to_string("weeks.txt")
            .unwrap()
            .lines()
            .map(|line| Date::parse(line.split(" ").last().unwrap(), &format).unwrap())
            .collect()
    }

    fn get_sorted_data_for_group(&self, groupe: usize) -> Vec<(usize, Prof)> {
        Self::sort_data(&self.get_data_for_group(groupe))
    }

    fn sort_data(data: &[(Vec<usize>, (Prof, Prof))]) -> Vec<(usize, Prof)> {
        let mut week_colles: Vec<(usize, (Prof, Prof))> = data
            .iter()
            .flat_map(|(weeks, colles)| weeks.iter().map(|week_id| (*week_id, colles.clone())))
            .collect();

        week_colles.sort_by(|a, b| a.0.cmp(&b.0));
        let days_colles = week_colles
            .into_iter()
            .flat_map(|(week, colles)| {
                let mut days = vec![(week, colles.0), (week, colles.1)];
                days.sort_by(|(_, a), (_, b)| a.jour.cmp(&b.jour));
                days
            })
            .collect();

        days_colles
    }

    fn get_date(&self, week: usize, jour: Jour) -> Date {
        self.weeks[week - 1]
            .saturating_sub(Duration::days(7))
            .next_occurrence(jour.into())
    }

    fn day_to_string(&self, colle: &Prof, date: Date, short: bool) -> String {
        format!(
            "{}: {} {} {} {} avec {} en {}",
            if short {
                colle.id.clone()
            } else {
                let explicit = explicit_colle_id(&colle.id).to_string();
                explicit
            },
            colle.jour.to_string(),
            date.day(),
            month_to_short_fr(date.month()),
            colle.horaire,
            colle.prof,
            colle.salle
        )
    }

    fn prochaines_colles_msg(&self) -> String {
        format!(
            "# Prochaines colles: {}",
            &self.week_groups[0]
                .groups
                .iter()
                .enumerate()
                .map(|(i, _)| format!(
                    "\n## Groupe {}{}",
                    i + 1,
                    self.get_sorted_data_for_group(i + 1)
                        .iter()
                        .map(|(week, colle)| (self.get_date(*week, colle.jour), colle))
                        .filter(|(date, _)| *date > OffsetDateTime::now_local().unwrap().date())
                        .collect::<Vec<_>>()[..2]
                        .iter()
                        .map(|(date, colle)| format!(
                            "\n- {}",
                            self.day_to_string(colle, *date, true)
                        ))
                        .collect::<String>()
                ))
                .collect::<String>()
        )
    }

    async fn edit_toutes_les_colles_msg(
        &self,
        ctx: serenity::prelude::Context,
    ) -> Result<(), Error> {
        if let Some((message_id, channel_id)) = Data::get_toutes_les_colles_id() {
            let channel = ChannelId::new(channel_id);
            let mut message = channel
                .message(&ctx.http, MessageId::new(message_id))
                .await?;
            message
                .edit(
                    ctx,
                    EditMessage::new().content(self.prochaines_colles_msg()),
                )
                .await?;
            debug!(
                "edited message {} in channel {} successfully",
                message_id, channel_id
            );
        }
        Ok(())
    }
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

#[poise::command(slash_command)]
async fn mes_colles(
    ctx: Context<'_>,
    #[description = "Groupe"] groupe: usize,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(format!(
                "Prochaines colles pour le groupe {}: \n- {}",
                groupe,
                Data::sort_data(&DATA.get_data_for_group(groupe))[..5]
                    .iter()
                    .map(|(week, colle)| {
                        let date = DATA.get_date(*week, colle.jour);
                        DATA.day_to_string(colle, date, false)
                    })
                    .collect::<Vec<_>>()
                    .join("\n- ")
            ))
            .reply(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn toutes_les_colles(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let handle = ctx.say(DATA.prochaines_colles_msg()).await?;

    let message = handle.message().await?;
    debug!(
        "new toutes les colles msg : {} {}",
        message.id, message.channel_id,
    );
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::prelude::Context, ready: Ready) {
        debug!("{} is connected!", ready.user.name);

        ctx.set_activity(Some(serenity::all::ActivityData {
            name: "les aventures de wattou".to_string(),
            kind: serenity::all::ActivityType::Watching,
            state: None,
            url: None,
        }));

        if let Err(err) = DATA.edit_toutes_les_colles_msg(ctx).await {
            debug!("Error : {:?}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![mes_colles(), toutes_les_colles()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    let client = serenity::Client::builder(token, GatewayIntents::GUILDS)
        .event_handler(Handler)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

#[derive(Debug, Clone)]
struct Prof {
    id: String,
    prof: String,
    horaire: String,
    jour: Jour,
    salle: String,
}

#[derive(Debug)]
struct ProfData(Vec<Prof>);

impl ProfData {
    fn parse() -> Self {
        let profs_data = fs::read_to_string("profs.txt").unwrap();

        Self(profs_data.lines().map(|d| d.into()).collect())
    }

    fn from_ids(&self, ids: (String, String)) -> (Prof, Prof) {
        let mut results = self
            .0
            .iter()
            .filter(|prof| prof.id == ids.0 || prof.id == ids.1);
        (
            results.next().unwrap().clone(),
            results.next().unwrap().clone(),
        )
    }
}

impl From<&str> for Prof {
    fn from(value: &str) -> Self {
        let mut str = value.to_string();
        let open_paren = value.find("(").unwrap();
        let salle = &str[open_paren + 1..value.len() - 1].to_string();
        str.replace_range(open_paren..value.len(), "");
        let mut words = str.split(" ");
        let id = words.next().unwrap();
        let mut words_vec: Vec<_> = words.collect();
        words_vec.pop();
        let horaire = words_vec.pop().unwrap();
        let jour_str = words_vec.pop().unwrap();
        let prof = words_vec.join(" ");

        Self {
            id: id.to_string(),
            prof,
            horaire: horaire.to_string(),
            jour: jour_str.into(),
            salle: salle.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Jour {
    Lundi,
    Mardi,
    Mercredi,
    Jeudi,
    Vendredi,
    Samedi,
    Dimanche,
}

impl Into<Weekday> for Jour {
    fn into(self) -> Weekday {
        match self {
            Self::Lundi => Weekday::Monday,
            Self::Mardi => Weekday::Tuesday,
            Self::Mercredi => Weekday::Wednesday,
            Self::Jeudi => Weekday::Thursday,
            Self::Vendredi => Weekday::Friday,
            Self::Samedi => Weekday::Saturday,
            Self::Dimanche => Weekday::Sunday,
        }
    }
}

impl From<&str> for Jour {
    fn from(value: &str) -> Self {
        match value {
            "Lu" => Self::Lundi,
            "Ma" => Self::Mardi,
            "Me" => Self::Mercredi,
            "Je" => Self::Jeudi,
            "Ve" => Self::Vendredi,
            "Sa" => Self::Samedi,
            "Di" => Self::Dimanche,
            _ => panic!(),
        }
    }
}

impl ToString for Jour {
    fn to_string(&self) -> String {
        match self {
            Self::Lundi => "Lundi",
            Self::Mardi => "Mardi",
            Self::Mercredi => "Mercredi",
            Self::Jeudi => "Jeudi",
            Self::Vendredi => "Vendredi",
            Self::Samedi => "Samedi",
            Self::Dimanche => "Dimanche",
        }
        .to_string()
    }
}

#[derive(Debug)]
struct WeekGroups {
    weeks: Vec<usize>,
    groups: Vec<(String, String)>,
}

impl WeekGroups {
    fn parse() -> Vec<Self> {
        let data = fs::read_to_string("data.txt").unwrap();
        let mut lines = data.lines();

        let first_line = lines.next().unwrap();
        let week_numbers: Vec<Vec<usize>> = first_line
            .split(" ")
            .map(|weeks_tuple| weeks_tuple.split("-").map(|n| n.parse().unwrap()).collect())
            .collect();
        let parsed_rows: Vec<Vec<(String, String)>> = lines
            .map(|group_colles| {
                group_colles
                    .split(" ")
                    .map(|colles| {
                        let mut colles_for_week = colles.split("+");
                        (
                            colles_for_week.next().unwrap().to_string(),
                            colles_for_week.next().unwrap().to_string(),
                        )
                    })
                    .collect()
            })
            .collect();

        let num_cols = parsed_rows.get(0).map_or(0, |row| row.len());
        let mut groups: Vec<Vec<(String, String)>> = vec![Vec::new(); num_cols];

        for row in parsed_rows {
            for (i, item) in row.into_iter().enumerate() {
                groups[i].push(item);
            }
        }
        let mut groups_iter = groups.into_iter();

        week_numbers
            .into_iter()
            .map(|weeks| WeekGroups {
                weeks,
                groups: groups_iter.next().unwrap(),
            })
            .collect()
    }
}

fn month_to_short_fr(month: Month) -> String {
    match month {
        Month::January => "Jan",
        Month::February => "Fev",
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

fn explicit_colle_id(id: &str) -> String {
    let mut s = match id.chars().nth(0).unwrap() {
        'M' => "Math",
        'P' => "Physique",
        'A' => "Anglais",
        _ => panic!(),
    }
    .to_string();
    s += " ";
    s.extend(id.chars().nth(1));
    s
}
