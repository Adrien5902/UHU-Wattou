#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod colle;
pub mod commands;
pub mod error;
pub mod group;
pub mod guild_data;
pub mod prof;
pub mod subscriber;
pub mod utils;

use crate::{
    commands::{colles_calendrier, mes_colles, semaine_tp, toutes_les_colles},
    error::WattouError,
    guild_data::GuildData,
    prof::Prof,
};
use anyhow::{Error, Result};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serenity::{
    all::{Http, Interaction, Ready},
    async_trait,
    prelude::*,
};
use std::{
    env,
    sync::{Arc, Mutex},
};

type Context<'a> = poise::Context<'a, Arc<Mutex<GlobalData>>, Error>;

#[derive(Default)]
pub struct GlobalData {
    pub guilds_data: Vec<Arc<GuildData>>,
    pub profs: Vec<Arc<Prof>>,
}

const GLOBAL_DATA: Lazy<Arc<Mutex<GlobalData>>> =
    Lazy::new(|| Arc::new(Mutex::new(GlobalData::default())));

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

        if let Err(e) = refresh_messages(&ctx.http).await {
            debug!("Error : {:?}", e);
        }
    }

    async fn interaction_create(&self, _ctx: serenity::prelude::Context, interaction: Interaction) {
        let command = interaction.as_command().unwrap();
        debug!("{} executed command {}", command.user.id, command.data.name);
    }
}

async fn refresh_messages(http: &Http) -> anyhow::Result<()> {
    let guilds = http.get_guilds(None, None).await?;
    for guild in guilds {
        match GuildData::get_from_id(guild.id) {
            Ok(guild_data) => {
                guild_data.edit_toutes_les_colles_msg(&http).await?;
                guild_data.edit_semaine_tp_msg(&http).await?;
            }
            Err(e) => {
                if !e.is::<WattouError>()
                    || *e.downcast_ref::<WattouError>().unwrap()
                        != WattouError::NoDataForGuild(guild.id)
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
                Ok(GLOBAL_DATA.clone())
            })
        })
        .build();

    let mut client: Client = serenity::Client::builder(token, GatewayIntents::GUILDS)
        .event_handler(Handler)
        .framework(framework)
        .await?;
    client.start().await?;

    Ok(())
}
