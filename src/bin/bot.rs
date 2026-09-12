#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::Result;
use dotenv::dotenv;
use poise::serenity_prelude::{
    self as serenity, ActivityData, ActivityType, Client, FullEvent, GatewayIntents, GuildId,
};
use std::env;
use tokio::sync::Mutex;
use wattou_bot::{bot::GlobalData, commands, debug, error::WattouError};

async fn event_handler(
    framework: poise::FrameworkContext<'_, Mutex<GlobalData>, color_eyre::Report>,
    event: &FullEvent,
) -> Result<()> {
    let data = framework.user_data;
    let ctx = framework.serenity_context;

    match event {
        FullEvent::Ready {
            data_about_bot: ready,
            ..
        } => {
            debug!("{} is connected!", ready.user.name);

            ctx.set_activity(Some(ActivityData {
                name: "les aventures de wattou".to_string(),
                kind: ActivityType::Watching,
                state: None,
                url: None,
            }));

            if let Err(e) = refresh_messages(ctx, data).await {
                debug!("Error : {:?}", e);
            }
        }
        FullEvent::InteractionCreate { interaction } => {
            if let Some(command) = interaction.as_command() {
                debug!("{} executed command {}", command.user.id, command.data.name);
            }
        }
        _ => {}
    }
    Ok(())
}

async fn refresh_messages(
    ctx: &serenity::Context,
    data: &Mutex<GlobalData>,
) -> color_eyre::Result<()> {
    let http = &*ctx.http;

    let guild_ids: Vec<GuildId> = http
        .get_guilds(None, None)
        .await?
        .iter()
        .map(|g| g.id)
        .collect();

    for id in guild_ids {
        match data.lock().await.get_guild(id) {
            Ok(guild_data) => {
                guild_data.refresh_messages(http).await?;
            }
            Err(e) => {
                if !e.is::<WattouError>()
                    || *e.downcast_ref::<WattouError>().unwrap() != WattouError::NoDataForGuild(id)
                {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |framework, event| Box::pin(event_handler(framework, event)),
            commands: vec![
                commands::clear(),
                commands::colles_de_prof(),
                // commands::rappel(),
                commands::mes_colles(),
                commands::toutes_les_colles(),
                commands::semaine_tp(),
                commands::colles_calendrier(),
                commands::link(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Mutex::new(GlobalData::default()))
            })
        })
        .build();

    let mut client: Client = Client::builder(token, GatewayIntents::GUILDS)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
