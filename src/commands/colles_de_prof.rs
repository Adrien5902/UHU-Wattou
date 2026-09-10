use crate::{
    bot::Context,
    data::{colle::ColleStringFormat, prof::Prof},
};
use color_eyre::{Result, eyre::eyre};
use poise::{
    CreateReply,
    serenity_prelude::{AutocompleteChoice, CreateAutocompleteResponse},
};
use std::fmt::Write;

#[poise::command(slash_command)]
pub async fn colles_de_prof(
    ctx: Context<'_>,
    #[rename = "prof"]
    #[autocomplete = "autocomplete_prof"]
    prof_str: String,
    #[rename = "limite"] limit: Option<usize>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let limit = limit.and_then(|l| (l < 100).then_some(l)).unwrap_or(5);
    let mut data = ctx.data().lock().await;
    let guild_data = data.guild_from_ctx(ctx)?;

    let Some((id, _prof)) = guild_data
        .persistent
        .profs
        .iter()
        .enumerate()
        .find(|(_, p)| p.name() == prof_str)
    else {
        Err(eyre!("Can't find prof with name {}", prof_str))?
    };

    let mut content = String::new();
    content.write_fmt(format_args!("Prochaines colles pour {}:", prof_str))?;

    for colle in Prof::get_next_colles(id, &guild_data.persistent, limit) {
        content.push_str("\n- ");
        colle.format(
            &mut content,
            ColleStringFormat::Implicit,
            &guild_data.persistent,
        )?;
    }

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(content)
            .reply(true),
    )
    .await?;

    Ok(())
}

pub async fn autocomplete_prof(ctx: Context<'_>, partial: &str) -> CreateAutocompleteResponse {
    let opt = (async || {
        let input = easy_comp_string(partial);
        let mut data = ctx.data().lock().await;
        let guild_data = data.guild_from_ctx(ctx).ok()?;
        let completions = guild_data
            .persistent
            .profs
            .iter()
            .filter_map(|p| {
                let name = p.name();
                easy_comp_string(name)
                    .contains(&input)
                    .then_some(name.to_owned())
            })
            .collect::<Vec<_>>();
        Some(completions)
    })()
    .await;
    CreateAutocompleteResponse::new().set_choices(
        opt.unwrap_or_default()
            .into_iter()
            .map(|c| AutocompleteChoice::new(c.clone(), c))
            .collect(),
    )
}

fn easy_comp_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}
