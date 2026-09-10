use crate::{
    data::{
        colle::{Colle, ColleStringFormat, ColleTemplate, ColleTemplateId},
        group::{Group, GroupId},
        prof::Prof,
        resolve::Resolve,
    },
    debug,
    error::WattouError,
    refreshable_message::{OptionalRefreshableMessage, RefreshableMessageKind},
    utils::month_to_short_fr,
};
use color_eyre::Result;
use poise::serenity_prelude::{GuildId, Http};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use time::{OffsetDateTime, Weekday};

pub type WeekId = usize;

#[derive(Debug)]
pub struct GuildData {
    pub persistent: GuildDataPersistent,
    pub mutable: GuildDataMutable,
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildDataPersistent {
    pub colle_templates: HashMap<ColleTemplateId, ColleTemplate>,
    pub profs: Vec<Prof>,
    // This is sorted
    pub colles: Vec<Colle>,
    pub groups: Vec<Group>,
    pub ghosts: Vec<GroupId>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GuildDataMutable {
    pub toutes_les_colles_msg: OptionalRefreshableMessage<ToutesLesCollesMessage>,
    pub semaine_tp_msg: OptionalRefreshableMessage<SemaineTPMessage>,
}

impl GuildData {
    const PERSISTENT_DATA_FILE: &'static str = "persistent_data.ron";
    const MUTABLE_DATA_FILE: &'static str = "mutable_data.ron";
    pub fn new(guild_id: GuildId) -> Result<Self> {
        let path = Self::folder(guild_id);
        if !fs::exists(&path)? {
            Err(WattouError::NoDataForGuild(guild_id))?
        }

        debug!("Parsing data for guild {}", guild_id);

        let persistent_data_string = fs::read_to_string(path.join(Self::PERSISTENT_DATA_FILE))?;
        let persistent = ron::from_str(&persistent_data_string)?;

        let mutable = fs::read_to_string(path.join(Self::MUTABLE_DATA_FILE))
            .ok()
            .map(|s| ron::from_str(&s))
            .unwrap_or_else(|| Ok(GuildDataMutable::default()))?;

        let data = Self {
            guild_id,
            mutable,
            persistent,
        };

        debug!("Parsed data for guild {}", guild_id);

        Ok(data)
    }

    fn folder(guild_id: GuildId) -> PathBuf {
        PathBuf::from("data").join(guild_id.to_string())
    }

    // pub fn subscribers(&self) -> Result<Subscribers> {
    //     Subscribers::read_or_default(self.guild_id)
    // }

    // pub async fn edit_semaine_tp_msg(&self, http: &Http) -> Result<()> {
    //     if let Some(message) = SemaineTPMessage::read(self.guild_id) {
    //         message?.edit(http, self.semaine_tp_msg()).await?;
    //     }
    //     Ok(())
    // }

    pub fn get_group(&self, group_id: usize) -> Result<&Group> {
        Ok(self
            .persistent
            .groups
            .iter()
            .find(|g| g.id == group_id)
            .ok_or(WattouError::GroupNotFound)?)
    }

    // pub async fn refresh_subscribers_message(&self, http: &Http) -> Result<()> {
    //     let subs = self.subscribers()?;
    //
    //     for (user_id, data) in subs.iter() {
    //         data.try_send(*user_id, http, &self).await?;
    //     }
    //
    //     Ok(())
    // }

    pub async fn refresh_messages(&self, http: &Http) -> Result<()> {
        self.mutable
            .toutes_les_colles_msg
            .refresh_if_some(http, &self.persistent)
            .await?;
        self.mutable
            .semaine_tp_msg
            .refresh_if_some(http, &self.persistent)
            .await?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        Ok(fs::write(
            Self::folder(self.guild_id).join(Self::MUTABLE_DATA_FILE),
            ron::to_string(&self.mutable)?,
        )?)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToutesLesCollesMessage;
impl RefreshableMessageKind for ToutesLesCollesMessage {
    fn content(&self, guild_data: &GuildDataPersistent) -> Result<String> {
        let mut final_message = String::new();

        for group in &guild_data.groups {
            let resolved = group.resolve(guild_data)?;
            let next_colles = resolved.get_next_colles(2);
            final_message.push_str("\n## Groupe ");
            final_message.push_str(&group.id.to_string());

            if guild_data.ghosts.contains(&group.id) {
                final_message.push_str(" 👻");
            }

            if !next_colles.is_empty() {
                for colle in next_colles {
                    final_message.push_str("\n- ");
                    colle.resolve(guild_data)?.format(
                        &mut final_message,
                        ColleStringFormat::Implicit,
                        guild_data,
                    )?;
                }
            } else {
                final_message.push_str("\nAucune colle prochainement");
            }
        }

        Ok(final_message)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SemaineTPMessage;
impl RefreshableMessageKind for SemaineTPMessage {
    fn content(&self, _guild_data: &GuildDataPersistent) -> Result<String> {
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

        Ok(format!(
            "Mercredi prochain ({} {}) le groupe 1 commence par {} et le groupe 2 par {}",
            date.day(),
            month_to_short_fr(date.month()),
            grp1,
            grp2
        ))
    }
}
