use crate::{
    Context, GLOBAL_DATA,
    colle::{Colle, ColleData},
    debug,
    error::{ColleParsingError, WattouError},
    group::{Group, GroupId},
    subscriber::Subscribers,
    utils::{Jour, month_to_short_fr},
};
use anyhow::Context as _;
use anyhow::Result;
use serenity::all::{ChannelId, EditMessage, GuildId, Http, Message, MessageId};
use std::{borrow::Cow, fs, num::ParseIntError, path::PathBuf, sync::Arc};
use time::{Date, Duration, OffsetDateTime, Weekday, macros::format_description};

pub type WeekId = usize;

#[derive(Debug)]
pub struct GuildData {
    pub guild_id: GuildId,
    pub groups: Vec<Group>,
    pub ghosts: Vec<GroupId>,
}

impl GuildData {
    pub const GLOBAL_DATA_FOLDER_NAME: &'static str = "data";
    pub const FILE_NAME_GHOSTS_GROUPS: &'static str = "ghosts";
    pub const FILE_NAME_COLLE_LIST: &'static str = "colles";
    pub const FILE_NAME_WEEKS_INFO: &'static str = "weeks";
    pub const FILE_NAME_COLLOSCOPE: &'static str = "colloscope";

    fn new(guild_id: GuildId) -> Result<Arc<Self>> {
        if !fs::exists(Self::folder(guild_id))? {
            Err(WattouError::NoDataForGuild(guild_id))?
        }

        debug!("Parsing data for guild {}", guild_id);

        let groups = Self::read_groups_data(guild_id)?;
        let ghosts = Self::read_ghost_groups(guild_id)?;

        let arc = Arc::new(Self {
            guild_id,
            groups,
            ghosts,
        });

        GLOBAL_DATA.lock().unwrap().guilds_data.push(arc.clone());

        debug!("Parsed data for guild {}", guild_id);

        Ok(arc)
    }

    pub fn get_from_id(id: GuildId) -> Result<Arc<Self>> {
        if let Some(arc) = GLOBAL_DATA
            .lock()
            .unwrap()
            .guilds_data
            .iter()
            .find(|d| d.guild_id == id)
        {
            return Ok(arc.clone());
        } else {
            let guild_data = Self::new(id)?;
            return Ok(guild_data);
        }
    }

    pub fn from_ctx(ctx: Context<'_>) -> Result<Arc<Self>> {
        let guild_id = ctx
            .guild_id()
            .ok_or(WattouError::CommandCanOnlyBeUsedInGuilds)?;
        Ok(Self::get_from_id(guild_id)?)
    }

    pub fn subscribers(&self) -> Result<Subscribers> {
        Subscribers::read_or_default(self.guild_id)
    }

    pub fn global_folder() -> PathBuf {
        Self::GLOBAL_DATA_FOLDER_NAME.into()
    }

    pub fn folder(id: GuildId) -> PathBuf {
        Self::global_folder().join(id.to_string())
    }

    pub fn get_file_path<'a>(id: GuildId, file: impl Into<&'a str>) -> PathBuf {
        Self::folder(id).join(file.into())
    }

    pub fn read_text_for_guild<'a>(id: GuildId, file: impl Into<&'a str>) -> Result<String> {
        let path = Self::get_file_path(id, file);
        Ok(fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.to_str().unwrap()))?)
    }

    pub fn read_groups_data(guild_id: GuildId) -> Result<Vec<Group>> {
        let colloscope = Self::parse_colloscope(guild_id)?;
        let groups = colloscope
            .into_iter()
            .enumerate()
            .map(|(i, colles)| Group {
                guild_id,
                id: i + 1,
                colles,
            })
            .collect();

        Ok(groups)
    }

    pub fn read_ghost_groups(guild_id: GuildId) -> Result<Vec<GroupId>> {
        let s = Self::read_text_for_guild(guild_id, Self::FILE_NAME_GHOSTS_GROUPS)?;
        let res: Result<Vec<usize>, ParseIntError> = s
            .lines()
            .map(|l| l.parse::<u64>().map(|u64| u64 as usize))
            .collect();
        Ok(res?)
    }

    pub fn parse_colloscope(guild_id: GuildId) -> Result<Vec<Vec<Colle>>> {
        let colloscope = Self::read_text_for_guild(guild_id, Self::FILE_NAME_COLLOSCOPE)?;
        let colle_list = Self::read_text_for_guild(guild_id, Self::FILE_NAME_COLLE_LIST)?
            .lines()
            .map(|s| Colle::parse_string(s))
            .collect::<Result<Vec<ColleData>>>()?;

        let weeks = Self::read_weeks_data(guild_id)?;

        let mut lines = colloscope.lines();

        let first_line = lines.next().unwrap();
        let week_numbers: Vec<Vec<usize>> = first_line
            .split(" ")
            .map(|weeks_tuple| weeks_tuple.split("-").map(|n| n.parse().unwrap()).collect())
            .collect();

        let groups = lines
            .map(|group_colles| {
                group_colles
                    .split(" ")
                    .enumerate()
                    .map(|(i, colles)| {
                        let weeks_n = &week_numbers[i];
                        let data = colles
                            .split("+")
                            .map(|colle_id| {
                                colle_list
                                    .iter()
                                    .find(|data| &data.0.to_string() == colle_id)
                                    .ok_or(WattouError::ColleParsingFailed(
                                        ColleParsingError::Unknown,
                                    ))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        let colles = weeks_n
                            .iter()
                            .flat_map(|week| {
                                data.iter().map(|d| {
                                    let (_, _, jour, _, _) = d;
                                    let date = Self::get_date(&weeks, *week, *jour);
                                    Colle::from_data_and_date(date, (*d).clone())
                                })
                            })
                            .collect::<Result<Vec<Colle>>>()?;

                        Ok(colles)
                    })
                    .collect::<Result<Vec<Vec<Colle>>>>()
                    .map(|inner| {
                        let mut colles = inner.into_iter().flatten().collect::<Vec<Colle>>();
                        colles.sort();
                        colles
                    })
            })
            .collect::<Result<_>>()?;

        Ok(groups)
    }

    pub fn read_weeks_data(guild_id: GuildId) -> Result<Vec<Date>> {
        let format = format_description!("[day padding:none]-[month padding:none]-[year]");
        let res = Self::read_text_for_guild(guild_id, Self::FILE_NAME_WEEKS_INFO)?
            .lines()
            .map(|line| Date::parse(line.split(" ").last().unwrap(), &format))
            .collect::<Result<Vec<Date>, _>>()?;
        Ok(res)
    }

    pub fn get_date(weeks: &[Date], week: usize, jour: Jour) -> Date {
        weeks[week - 1]
            .saturating_sub(Duration::days(7))
            .next_occurrence(jour.inner())
    }

    pub fn prochaines_colles_msg(&self) -> String {
        format!(
            "# Prochaines colles: {}",
            self.groups
                .iter()
                .map(|group| format!(
                    "\n### Groupe {}{} {}",
                    group.id,
                    if self.ghosts.contains(&group.id) {
                        " (fantôme 👻)"
                    } else {
                        ""
                    },
                    group
                        .get_next_colles(2)
                        .iter()
                        .map(|colle| format!(
                            "\n- {}",
                            colle.to_string(crate::colle::ColleStringFormat::Implicit)
                        ))
                        .collect::<String>()
                ))
                .collect::<String>()
        )
    }

    pub async fn edit_toutes_les_colles_msg(&self, http: &Http) -> Result<()> {
        let (message_id, channel_id) = ToutesLesCollesMessage::read(self.guild_id)?.0;
        let mut message = http.get_message(channel_id, message_id).await?;

        let text = self.prochaines_colles_msg();
        message.edit(http, EditMessage::new().content(text)).await?;
        debug!(
            "edited message {} in channel {} successfully",
            message.id, message.channel_id
        );

        Ok(())
    }

    pub fn semaine_tp_msg(&self) -> String {
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
            "Mercredi prochain ({} {}) le group 1 commence par {} et le group 2 par {}",
            date.day(),
            month_to_short_fr(date.month()),
            grp1,
            grp2
        )
    }

    pub async fn edit_semaine_tp_msg(&self, http: &Http) -> Result<()> {
        let (message_id, channel_id) = SemaineTPMessage::read(self.guild_id)?.0;
        let mut message = http.get_message(channel_id, message_id).await?;

        message
            .edit(http, EditMessage::new().content(self.semaine_tp_msg()))
            .await?;

        debug!(
            "edited message {} in channel {} successfully",
            message.id, message.channel_id
        );

        Ok(())
    }

    pub fn get_group(&self, group_id: usize) -> Result<&Group> {
        Ok(self
            .groups
            .iter()
            .find(|g| g.id == group_id)
            .ok_or(WattouError::GroupNotFound)?)
    }
}

pub trait SavedData: Sized {
    const FILE_NAME: &'static str;

    fn ser(&self) -> String;
    fn de(value: &str) -> Result<Self>;

    fn read<'a>(guild_id: GuildId) -> Result<Self> {
        Self::de(&GuildData::read_text_for_guild(guild_id, Self::FILE_NAME)?)
    }

    fn save<'a>(&self, guild_id: GuildId) -> Result<()> {
        let path = GuildData::get_file_path(guild_id, Self::FILE_NAME);
        fs::write(path, self.ser())?;
        Ok(())
    }

    fn exists(guild_id: GuildId) -> Result<bool> {
        Ok(fs::exists(GuildData::get_file_path(
            guild_id,
            Subscribers::FILE_NAME,
        ))?)
    }
}

pub trait SavedDataWithDefault: SavedData + Default {
    fn read_or_default(guild_id: GuildId) -> Result<Self> {
        if Self::exists(guild_id)? {
            Self::read(guild_id)
        } else {
            Ok(Self::default())
        }
    }
}

pub struct RecurrentMessage<const ID: usize>((MessageId, ChannelId));

impl<const ID: usize> RecurrentMessage<ID> {
    const IDS: [&'static str; 2] = ["message_semaine_tp", "message_colles"];
}

impl<const ID: usize> SavedData for RecurrentMessage<ID> {
    const FILE_NAME: &'static str = Self::IDS[ID];

    fn de(s: &str) -> Result<Self> {
        let mut lines = s.lines();

        let (message_id, channel_id) =
            (|| Some((lines.next()?.parse().ok()?, lines.next()?.parse().ok()?)))()
                .ok_or(WattouError::MessageParsingFailed)?;

        Ok(Self((message_id, channel_id)))
    }

    fn ser(&self) -> String {
        [self.0.0.to_string(), self.0.1.to_string()].join("\n")
    }
}

impl<const ID: usize> From<&Cow<'_, Message>> for RecurrentMessage<ID> {
    fn from(value: &Cow<'_, Message>) -> Self {
        Self((value.id, value.channel_id))
    }
}

pub type SemaineTPMessage = RecurrentMessage<0>;
pub type ToutesLesCollesMessage = RecurrentMessage<1>;
