use poise::CreateReply;

use crate::{
    Context,
    data::{colle::ColleStringFormat, guild::GuildData},
};

use std::{fmt::Write, sync::Arc};

use color_eyre::Result;

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
    let data = GuildData::from_ctx(ctx)?;

    let Some(prof) = ctx
        .data()
        .lock()
        .unwrap()
        .profs
        .get(&Arc::from(prof_str))
        .map(|p| p.clone())
    else {
        todo!("impl error");
    };

    let mut content = String::new();
    content.write_fmt(format_args!("Prochaines colles pour {}:", prof.name()))?;

    for (group_id, colle) in prof.get_next_colles_in_guild(data, limit) {
        content.push_str("\n- ");
        colle.format(&mut content, ColleStringFormat::ForProf(group_id))?;
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

pub async fn autocomplete_prof(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let input = easy_comp_string(partial);
    ctx.data()
        .lock()
        .unwrap()
        .profs
        .iter()
        .filter_map(|(_, p)| {
            let name = p.name();
            easy_comp_string(name)
                .contains(&input)
                .then_some(name.to_owned())
        })
        .collect::<Vec<_>>()
}

fn easy_comp_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}
