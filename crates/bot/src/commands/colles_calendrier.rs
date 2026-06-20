use crate::{
    Context,
    data::{
        colle::Colle,
        group::{Group, GroupId},
        guild::GuildData,
    },
};
use color_eyre::{Result, eyre};
use ics::{
    Event, ICalendar,
    properties::{Categories, Description, DtEnd, DtStart, Organizer, Summary},
};
use once_cell::sync::Lazy;
use poise::CreateReply;
use serenity::all::CreateAttachment;
use time::format_description::well_known::Iso8601;
use uuid::Uuid;

const ICS_CATEGORY: Lazy<Categories> = Lazy::new(|| Categories::new("Colles"));

impl Colle {
    fn to_ics_event(&self) -> Result<Event<'_>> {
        let [start, end]: [String; 2] = [self.start, self.end]
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

        let mut event = Event::new(Uuid::new_v4().to_string(), start.clone());

        event.push(Organizer::new(self.prof.to_string()));
        event.push(DtStart::new(start));
        event.push(DtEnd::new(end));
        event.push(ICS_CATEGORY.clone());
        event.push(Summary::new(format!(
            "Colle {} avec {}",
            &self.id.explicit(),
            &self.prof.to_string()
        )));
        event.push(Description::new(format!(
            "Colle {} avec {} en salle {} de {}",
            &self.id.explicit(),
            &self.prof.to_string(),
            &self.room,
            self.horaire()
        )));

        Ok(event)
    }
}

impl Group {
    fn ics_calendar<'a>(&self) -> Result<String> {
        let mut calendar = ICalendar::new(
            "2.0",
            format!("-//Wattou//Calendrier de colle groupe {}//FR", self.id),
        );

        // create event which contains the information regarding the conference
        for colle in self.colles.iter() {
            calendar.add_event(colle.to_ics_event()?);
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
    let data = GuildData::from_ctx(ctx)?;

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .attachment(CreateAttachment::bytes(
                data.get_group(group_id)?.ics_calendar()?,
                format!("Calendrier de colles group {}.ics", group_id),
            ))
            .content("Importe le fichier dans ton calendrier pour y ajouter les colles !"),
    )
    .await?;
    Ok(())
}
