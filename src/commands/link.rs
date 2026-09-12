use crate::{bot::Context, utils::easy_comp_string};
use color_eyre::{Result, eyre::eyre};
use poise::{
    CreateReply,
    serenity_prelude::{AutocompleteChoice, CreateAutocompleteResponse},
};

#[poise::command(slash_command)]
pub async fn link(
    ctx: Context<'_>,
    #[rename = "eleve"]
    #[autocomplete = "autocomplete_student"]
    student_str_opt: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_from_ctx(ctx)?;

    let reply_content = match student_str_opt {
        Some(student_str) => {
            let Some((id, student)) = guild_data
                .persistent
                .students
                .iter()
                .enumerate()
                .find(|(_, s)| s.to_string() == student_str)
            else {
                Err(eyre!("Can't find student with name {}", student_str))?
            };

            {
                let mut mutable = data.guild_mut_from_ctx(ctx).await?;
                mutable.students_link.insert(ctx.author().id, id);
            }
            guild_data.save().await?;

            format!("Compte discord lié avec {}", student)
        }
        None => {
            format!("Compte discord délié")
        }
    };

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(reply_content)
            .reply(true),
    )
    .await?;

    Ok(())
}

pub async fn autocomplete_student(ctx: Context<'_>, partial: &str) -> CreateAutocompleteResponse {
    let opt = (async || {
        let input = easy_comp_string(partial);
        let mut data = ctx.data().lock().await;
        let guild_data = data.guild_from_ctx(ctx).ok()?;
        let completions = guild_data
            .persistent
            .students
            .iter()
            .filter_map(|s| {
                let name = s.to_string();
                easy_comp_string(&name).contains(&input).then_some(name)
            })
            .collect::<Vec<_>>();
        Some(completions)
    })()
    .await;

    CreateAutocompleteResponse::new().set_choices(
        opt.unwrap_or_default()
            .into_iter()
            .take(20)
            .map(|c| AutocompleteChoice::new(c.clone(), c))
            .collect(),
    )
}
