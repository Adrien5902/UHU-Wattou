#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ics::{
    Event, ICalendar,
    properties::{Categories, Description, DtEnd, DtStart, Organizer, Summary},
};
use poise::CreateReply;
use serenity::{
    all::{ChannelId, CreateAttachment, EditMessage, Interaction, Message, MessageId, Ready},
    async_trait,
    prelude::*,
};
use std::{
    cell::LazyCell,
    fs::{self, OpenOptions},
    io::{self, Write},
};
use time::{
    Date, Duration, Month, OffsetDateTime, Time, UtcDateTime, Weekday,
    format_description::well_known::Iso8601, macros::format_description,
};
use uuid::Uuid;

use dotenv::dotenv;
use std::env;

type WeekId = usize;

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
    let (groups, profs) = (WeekGroups::parse(), CollesData::parse());
    Data {
        week_groups: groups,
        profs,
        weeks: Data::get_weeks(),
    }
});

struct Data {
    profs: CollesData,
    week_groups: Vec<WeekGroups>,
    weeks: Vec<Date>,
}

impl Data {
    fn get_message_id(file_name: &str) -> Option<(u64, u64)> {
        let Some(file) = fs::read_to_string(file_name.to_string() + ".txt").ok() else {
            return None;
        };

        let mut lines = file.lines();

        Some((lines.next()?.parse().ok()?, lines.next()?.parse().ok()?))
    }

    async fn get_message_from_file(
        ctx: &serenity::prelude::Context,
        file_name: &str,
    ) -> Option<Result<Message, SerenityError>> {
        let Some((message_id, channel_id)) = Data::get_message_id(file_name) else {
            return None;
        };

        let channel = ChannelId::new(channel_id);
        Some(channel.message(&ctx.http, MessageId::new(message_id)).await)
    }

    fn get_data_for_group(&self, group: usize) -> Vec<(Vec<WeekId>, (Colle, Colle))> {
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

    fn get_sorted_data_for_group(&self, groupe: usize) -> Vec<(WeekId, Colle)> {
        Self::sort_data(&self.get_data_for_group(groupe))
    }

    fn sort_data(data: &[(Vec<WeekId>, (Colle, Colle))]) -> Vec<(WeekId, Colle)> {
        let mut week_colles: Vec<(WeekId, (Colle, Colle))> = data
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
            .next_occurrence(jour.inner())
    }

    fn day_to_string(&self, colle: &Colle, date: Date, short: bool) -> String {
        format!(
            "{}: {} {} {} {} avec {} en {}",
            if short {
                colle.id.clone()
            } else {
                let explicit = colle.explicit_id();
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
            (0..self.week_groups[0].groups.len())
                .map(|i| format!(
                    "\n## Groupe {}{} {}",
                    i + 1,
                    if i == 9 { " (fantôme 👻)" } else { "" },
                    DATA.get_next_colles_for_groupe(i + 1)
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
        ctx: &serenity::prelude::Context,
    ) -> Result<(), Error> {
        if let Some(message_res) = Data::get_message_from_file(ctx, "message").await {
            let mut message = message_res?;
            message
                .edit(
                    ctx,
                    EditMessage::new().content(self.prochaines_colles_msg()),
                )
                .await?;
            debug!(
                "edited message {} in channel {} successfully",
                message.id, message.channel_id
            );
        }
        Ok(())
    }

    fn semaine_tp_msg() -> String {
        let date = OffsetDateTime::now_local()
            .unwrap()
            .date()
            .next_occurrence(Weekday::Wednesday);

        let grp = date.iso_week() % 2 == 1;

        let math = "td maths";
        let physique = "tp physique";

        let (grp1, grp2);
        if grp {
            grp1 = math;
            grp2 = physique;
        } else {
            grp1 = physique;
            grp2 = math;
        }

        format!(
            "Mercredi prochain ({} {}) le groupe 1 commence par {} et le groupe 2 par {}",
            date.day(),
            month_to_short_fr(date.month()),
            grp1,
            grp2
        )
    }

    async fn edit_semaine_tp_msg(&self, ctx: &serenity::prelude::Context) -> Result<(), Error> {
        if let Some(message_res) = Data::get_message_from_file(ctx, "semaine_tp_message").await {
            let mut message = message_res?;
            message
                .edit(ctx, EditMessage::new().content(Data::semaine_tp_msg()))
                .await?;
            debug!(
                "edited message {} in channel {} successfully",
                message.id, message.channel_id
            );
        }
        Ok(())
    }

    fn get_next_colles_for_groupe(&self, groupe: usize) -> Vec<(Date, Colle)> {
        self.get_sorted_data_for_group(groupe)
            .into_iter()
            .filter_map(|(week, colle)| {
                let date = self.get_date(week, colle.jour);
                (date > OffsetDateTime::now_local().unwrap().date()).then_some((date, colle))
            })
            .collect::<Vec<_>>()
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
                DATA.get_next_colles_for_groupe(groupe)[..5]
                    .iter()
                    .map(|(date, colle)| { DATA.day_to_string(colle, *date, false) })
                    .collect::<Vec<_>>()
                    .join("\n- ")
            ))
            .reply(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn colles_calendrier(
    ctx: Context<'_>,
    #[description = "Groupe"] groupe: usize,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .attachment(CreateAttachment::bytes(
                ics_file(groupe)?,
                format!("Calendrier de colles groupe {}.ics", groupe),
            ))
            .content("Importe le fichier dans ton calendrier pour y ajouter les colles !"),
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

#[poise::command(slash_command)]
async fn semaine_tp(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let handle = ctx.say(Data::semaine_tp_msg()).await?;

    let message = handle.message().await?;
    debug!("new semaine tp msg : {} {}", message.id, message.channel_id,);
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

        if let Err(err) = DATA.edit_toutes_les_colles_msg(&ctx).await {
            debug!("Error : {:?}", err);
        }

        if let Err(err) = DATA.edit_semaine_tp_msg(&ctx).await {
            debug!("Error : {:?}", err);
        }
    }

    async fn interaction_create(&self, _ctx: serenity::prelude::Context, interaction: Interaction) {
        let command = interaction.as_command().unwrap();
        debug!("{} executed command {}", command.user.id, command.data.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                mes_colles(),
                toutes_les_colles(),
                semaine_tp(),
                colles_calendrier(),
            ],
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
struct Colle {
    id: String,
    prof: String,
    horaire: String,
    jour: Jour,
    salle: String,
}

impl Colle {
    fn get_start_end_time(&self) -> (u8, u8) {
        let [start, end] = self
            .horaire
            .split("-")
            .map(|s| s[..s.len() - 1].parse().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        (start, end)
    }

    fn explicit_id(&self) -> String {
        let mut s = match self.id.chars().nth(0).unwrap() {
            'M' => "Math",
            'P' => "Physique",
            'A' => "Anglais",
            _ => panic!(),
        }
        .to_string();
        s += " ";
        s.extend(self.id.chars().nth(1));
        s
    }
}

#[derive(Debug)]
struct CollesData(Vec<Colle>);

impl CollesData {
    fn parse() -> Self {
        let profs_data = fs::read_to_string("profs.txt").unwrap();

        Self(profs_data.lines().map(|d| d.into()).collect())
    }

    fn from_ids(&self, ids: (String, String)) -> (Colle, Colle) {
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

impl From<&str> for Colle {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Jour(Weekday);

impl Jour {
    fn inner(&self) -> Weekday {
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

fn get_ics_date_time(date: Date, hour: u8) -> Result<String, Error> {
    let date_time = UtcDateTime::new(date, Time::from_hms(hour, 0, 0)?);
    let s = date_time
        .format(&Iso8601::DATE_TIME)?
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    Ok(s)
}

fn ics_file<'a>(groupe: usize) -> Result<String, Error> {
    let mut calendar = ICalendar::new(
        "2.0",
        format!("-//Wattou//Calendrier de colle groupe {}//FR", groupe),
    );

    let colles = DATA.get_sorted_data_for_group(groupe);
    let category = Categories::new("Colles");

    // create event which contains the information regarding the conference
    for (week_id, colle) in colles {
        let date = DATA.get_date(week_id, colle.jour);
        let (start, end) = colle.get_start_end_time();
        let start_date = get_ics_date_time(date, start)?;
        let end_date = get_ics_date_time(date, end)?;
        let mut event = Event::new(Uuid::new_v4().to_string(), start_date.clone());

        event.push(Organizer::new(colle.prof.clone()));
        event.push(DtStart::new(start_date));
        event.push(DtEnd::new(end_date));
        event.push(category.clone());
        event.push(Summary::new(format!(
            "Colle {} avec {}",
            &colle.explicit_id(),
            &colle.prof
        )));
        event.push(Description::new(format!(
            "Colle {} avec {} en salle {} de {}",
            &colle.explicit_id(),
            &colle.prof,
            &colle.salle,
            &colle.horaire
        )));
        calendar.add_event(event);
    }

    let mut writer = Vec::new();
    // write calendar to file
    calendar.write(&mut writer).unwrap();
    Ok(String::from_utf8(writer).unwrap())
}
