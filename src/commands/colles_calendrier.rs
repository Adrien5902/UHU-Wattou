use crate::{
    bot::Context,
    data::{
        colle::{ColleId, ResolvedColle},
        group::{Group, GroupId},
        guild::GuildDataPersistent,
        prof::Prof,
        resolve::Resolve,
    },
};
use color_eyre::{
    Result,
    eyre::{self, eyre},
};
use ics::{
    Event, ICalendar,
    properties::{Categories, Description, DtEnd, DtStart, Organizer, Summary},
};
use once_cell::sync::Lazy;
use poise::{
    CreateReply,
    serenity_prelude::{CreateAttachment, GuildId},
};
use time::format_description::well_known::Iso8601;
use uuid::Uuid;

const ICS_CATEGORY: Lazy<Categories> = Lazy::new(|| Categories::new("Colles"));

impl<'g: 's, 's> ResolvedColle<'g, 's> {
    pub fn to_ics_event<'e>(
        &self,
        guild_data: &'e GuildDataPersistent,
        guild_id: GuildId,
        colle_id: ColleId,
    ) -> Result<Event<'e>> {
        let [start, end]: [String; 2] = [self.colle.start, self.colle.end]
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

        let key = format!(
            "https://github.com/Adrien5902/UHU-Wattou/calendar/v1/guild/{}/group/{}/colle/{}",
            guild_id, self.group.id, colle_id
        );
        let mut event = Event::new(
            Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes()).to_string(),
            start.clone(),
        );
        let prof =
            Prof::from_id(&self.template.prof, guild_data).ok_or_else(|| eyre!("error :("))?;

        event.push(Organizer::new(prof.name()));
        event.push(DtStart::new(start));
        event.push(DtEnd::new(end));
        event.push(ICS_CATEGORY.clone());
        event.push(Summary::new(format!(
            "Colle {} avec {}",
            self.colle.template_id.explicit(),
            prof,
        )));
        event.push(Description::new(format!(
            "Colle {} avec {} en salle {} de {}",
            self.colle.template_id.explicit(),
            prof,
            self.template.room,
            self.colle.horaire()
        )));

        Ok(event)
    }
}

impl Group {
    fn ics_calendar(&self, guild_data: &GuildDataPersistent, guild_id: GuildId) -> Result<String> {
        let mut calendar = ICalendar::new(
            "2.0",
            format!("-//Wattou//Calendrier de colle groupe {}//FR", self.id),
        );

        let resolved = self.resolve(guild_data)?;
        // create event which contains the information regarding the conference
        for (i, colle) in resolved.colles.iter().enumerate() {
            let resolved_colle = colle.resolve(guild_data)?;
            let event = resolved_colle.to_ics_event(guild_data, guild_id, self.colles[i])?;
            calendar.add_event(event);
        }

        let mut writer = Vec::new();
        calendar.write(&mut writer).unwrap();
        Ok(String::from_utf8(writer).unwrap())
    }
}

#[poise::command(slash_command, guild_only)]
pub async fn colles_calendrier(
    ctx: Context<'_>,
    #[description = "Groupe de colle"]
    #[rename = "groupe"]
    group_id: GroupId,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_from_ctx(ctx)?;

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .attachment(CreateAttachment::bytes(
                Group::from_id(&group_id, &guild_data.persistent)
                    .ok_or_else(|| eyre!("error :("))?
                    .ics_calendar(&guild_data.persistent, guild_data.guild_id)?,
                format!("Calendrier de colles du groupe {}.ics", group_id),
            ))
            .content("Importe le fichier dans ton calendrier pour y ajouter les colles !"),
    )
    .await?;
    Ok(())
}
